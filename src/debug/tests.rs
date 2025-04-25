# Debug Agent Tests

use crate::ast::{AstNode, AstNodeRef};
use crate::error::Error;
use crate::value::Value;
use crate::debug::{
    DebugManager, DebugConfig, DebugState, DebugEvent,
    ast_stepper::{SourceLocation, PauseReason, BreakpointId, StepMode},
    variable_tracker::{ScopeId, WatchId},
    error_analyzer::{ErrorType},
    fix_suggester::{FixSuggestion, CodeChange, FixConfidence}
};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

// Mock AST node for testing
fn create_mock_ast_node() -> AstNode {
    AstNode::Identifier { name: "test_var".to_string() }
}

// Mock error for testing
fn create_mock_error() -> Error {
    Error::UndefinedVariable("test_var".to_string())
}

// Mock value for testing
fn create_mock_value() -> Value {
    Value::Number(42.0)
}

#[test]
fn test_debug_manager_creation() {
    let config = DebugConfig::default();
    let debug_manager = DebugManager::new(config);
    
    assert_eq!(debug_manager.get_debug_state(), DebugState::Inactive);
    assert!(!debug_manager.is_debugging_active());
    assert!(!debug_manager.is_execution_paused());
}

#[test]
fn test_debug_manager_start_stop() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    assert_eq!(debug_manager.get_debug_state(), DebugState::Active);
    assert!(debug_manager.is_debugging_active());
    
    debug_manager.stop_debugging();
    assert_eq!(debug_manager.get_debug_state(), DebugState::Inactive);
    assert!(!debug_manager.is_debugging_active());
}

#[test]
fn test_ast_stepping() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Set a breakpoint
    let location = SourceLocation {
        file: "main.ai".to_string(),
        line: 1,
        column: 1,
    };
    let breakpoint_id = debug_manager.set_breakpoint(location.clone());
    
    // Create a mock node at the breakpoint location
    let node = create_mock_ast_node();
    
    // This should pause execution
    debug_manager.before_node_execution(&node);
    
    // Check that execution is paused
    assert_eq!(debug_manager.get_debug_state(), DebugState::Paused);
    
    // Continue execution
    debug_manager.continue_execution();
    
    // Check that execution is resumed
    assert_eq!(debug_manager.get_debug_state(), DebugState::Active);
    
    // Disable the breakpoint
    assert!(debug_manager.disable_breakpoint(breakpoint_id));
    
    // This should not pause execution now
    debug_manager.before_node_execution(&node);
    
    // Check that execution is still active
    assert_eq!(debug_manager.get_debug_state(), DebugState::Active);
}

#[test]
fn test_variable_tracking() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Create a scope
    let scope_id = debug_manager.create_scope("test_scope", None);
    
    // Set a variable
    let var_name = "test_var";
    let var_value = create_mock_value();
    debug_manager.on_variable_change(var_name, var_value.clone());
    
    // Get the variable value
    let retrieved_value = debug_manager.get_variable_value(var_name);
    assert!(retrieved_value.is_some());
    assert_eq!(format!("{:?}", retrieved_value.unwrap()), format!("{:?}", var_value));
    
    // Create a child scope
    let child_scope_id = debug_manager.create_scope("child_scope", Some(scope_id));
    
    // Set a variable in the child scope
    let child_var_name = "child_var";
    let child_var_value = Value::String("child value".to_string());
    debug_manager.on_variable_change(child_var_name, child_var_value.clone());
    
    // Get the variable value from the child scope
    let retrieved_child_value = debug_manager.get_variable_value(child_var_name);
    assert!(retrieved_child_value.is_some());
    assert_eq!(format!("{:?}", retrieved_child_value.unwrap()), format!("{:?}", child_var_value));
    
    // Exit the child scope
    let exited_scope_id = debug_manager.exit_scope();
    assert_eq!(exited_scope_id, Some(child_scope_id));
    
    // The child variable should no longer be accessible
    let retrieved_child_value = debug_manager.get_variable_value(child_var_name);
    assert!(retrieved_child_value.is_none());
    
    // But the parent variable should still be accessible
    let retrieved_value = debug_manager.get_variable_value(var_name);
    assert!(retrieved_value.is_some());
    assert_eq!(format!("{:?}", retrieved_value.unwrap()), format!("{:?}", var_value));
}

#[test]
fn test_error_analysis() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Create a mock error
    let error = create_mock_error();
    
    // This should trigger error analysis
    debug_manager.on_error(&error);
    
    // Get error details
    let error_details = debug_manager.get_error_details(&error);
    assert!(error_details.is_some());
    
    let details = error_details.unwrap();
    assert_eq!(format!("{:?}", details.error_info.error), format!("{:?}", error));
    
    // The error should be classified as a reference error
    assert_eq!(details.error_info.error_type, ErrorType::Reference);
}

#[test]
fn test_fix_suggestions() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Create a mock error
    let error = create_mock_error();
    
    // Create a scope with a similar variable
    let scope_id = debug_manager.create_scope("test_scope", None);
    debug_manager.on_variable_change("test_var2", create_mock_value());
    
    // Update the fix suggester's context
    let mut variables = HashMap::new();
    variables.insert("test_var2".to_string(), "Number".to_string());
    debug_manager.get_fix_suggester().update_available_variables(variables);
    
    // This should trigger error analysis and fix suggestions
    debug_manager.on_error(&error);
    
    // Get fix suggestions
    let suggestions = debug_manager.get_fix_suggestions(&error);
    
    // There should be at least one suggestion
    assert!(!suggestions.is_empty());
    
    // The suggestion should be to replace "test_var" with "test_var2"
    let suggestion = &suggestions[0];
    match &suggestion.code_change {
        CodeChange::Replace { old_code, new_code, .. } => {
            assert_eq!(old_code, "test_var");
            assert_eq!(new_code, "test_var2");
        },
        _ => panic!("Expected Replace code change"),
    }
}

#[test]
fn test_event_listeners() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    // Create a counter to track events
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();
    
    // Add an event listener
    debug_manager.add_event_listener(move |event| {
        match event {
            DebugEvent::ExecutionPaused { .. } => {
                *counter_clone.borrow_mut() += 1;
            },
            _ => {},
        }
    });
    
    debug_manager.start_debugging();
    
    // Set a breakpoint
    let location = SourceLocation {
        file: "main.ai".to_string(),
        line: 1,
        column: 1,
    };
    let breakpoint_id = debug_manager.set_breakpoint(location.clone());
    
    // Create a mock node at the breakpoint location
    let node = create_mock_ast_node();
    
    // This should pause execution and trigger an event
    debug_manager.before_node_execution(&node);
    
    // Check that the event was triggered
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn test_stepping_modes() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Set a breakpoint
    let location = SourceLocation {
        file: "main.ai".to_string(),
        line: 1,
        column: 1,
    };
    let breakpoint_id = debug_manager.set_breakpoint(location.clone());
    
    // Create a mock node at the breakpoint location
    let node = create_mock_ast_node();
    
    // This should pause execution
    debug_manager.before_node_execution(&node);
    
    // Check that execution is paused
    assert_eq!(debug_manager.get_debug_state(), DebugState::Paused);
    
    // Step into
    debug_manager.step_into();
    
    // Check that execution is resumed
    assert_eq!(debug_manager.get_debug_state(), DebugState::Active);
    
    // The step mode should be StepInto
    assert_eq!(debug_manager.get_ast_stepper().get_step_mode(), StepMode::StepInto);
    
    // Pause again
    debug_manager.before_node_execution(&node);
    
    // Step over
    debug_manager.step_over();
    
    // The step mode should be StepOver
    assert_eq!(debug_manager.get_ast_stepper().get_step_mode(), StepMode::StepOver);
    
    // Pause again
    debug_manager.before_node_execution(&node);
    
    // Step out
    debug_manager.step_out();
    
    // The step mode should be StepOut
    assert_eq!(debug_manager.get_ast_stepper().get_step_mode(), StepMode::StepOut);
}

#[test]
fn test_watch_expressions() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Add a watch expression
    let watch_id = debug_manager.add_watch("test_var");
    
    // Set the variable
    let var_name = "test_var";
    let var_value = create_mock_value();
    debug_manager.on_variable_change(var_name, var_value.clone());
    
    // The watch should be triggered
    // In a real implementation, this would emit an event
    // For this test, we just check that the watch exists
    let watches = debug_manager.get_variable_tracker().get_watches();
    assert!(!watches.is_empty());
    
    let watch = watches.iter().find(|w| w.id == watch_id);
    assert!(watch.is_some());
    assert_eq!(watch.unwrap().expression, "test_var");
}

#[test]
fn test_error_handling_during_execution() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Create a mock node
    let node = create_mock_ast_node();
    
    // Create a mock error
    let error = create_mock_error();
    
    // This should trigger error analysis
    debug_manager.after_node_execution(&node, &Err(error.clone()));
    
    // Get error details
    let error_details = debug_manager.get_error_details(&error);
    assert!(error_details.is_some());
}

#[test]
fn test_applying_fix() {
    let config = DebugConfig::default();
    let mut debug_manager = DebugManager::new(config);
    
    debug_manager.start_debugging();
    
    // Create a mock error
    let error = create_mock_error();
    
    // Create a scope with a similar variable
    let scope_id = debug_manager.create_scope("test_scope", None);
    debug_manager.on_variable_change("test_var2", create_mock_value());
    
    // Update the fix suggester's context
    let mut variables = HashMap::new();
    variables.insert("test_var2".to_string(), "Number".to_string());
    debug_manager.get_fix_suggester().update_available_variables(variables);
    
    // This should trigger error analysis and fix suggestions
    debug_manager.on_error(&error);
    
    // Get fix suggestions
    let suggestions = debug_manager.get_fix_suggestions(&error);
    
    // There should be at least one suggestion
    assert!(!suggestions.is_empty());
    
    // Apply the first suggestion
    let result = debug_manager.apply_fix(&suggestions[0]);
    
    // The fix should be applied successfully
    assert!(result.is_ok());
}
