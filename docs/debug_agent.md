# Debug Agent Documentation

## Overview

The Debug Agent is a comprehensive debugging tool for Anarchy Inference that provides advanced debugging capabilities including AST stepping and inspection, variable state tracking, error trace analysis, and automated fix suggestions. This document provides detailed information about the Debug Agent's architecture, components, and usage.

## Architecture

The Debug Agent follows a modular architecture with five core components:

1. **Debug Manager**: The central component that coordinates all debugging activities and provides a unified interface for the Debug Agent.
2. **AST Stepper**: Responsible for stepping through AST nodes during execution, managing breakpoints, and tracking execution history.
3. **Variable Tracker**: Tracks variable states throughout program execution, manages scopes, and supports watch expressions.
4. **Error Analyzer**: Analyzes errors and provides detailed diagnostics, including error classification, stack trace management, and pattern matching.
5. **Fix Suggester**: Suggests fixes for common errors based on error patterns and context analysis.

## Components

### Debug Manager

The Debug Manager is the central component that coordinates all debugging activities. It provides a unified interface for the Debug Agent and manages the state of the debugging session.

```rust
// Create a debug manager with default configuration
let config = DebugConfig::default();
let mut debug_manager = DebugManager::new(config);

// Start debugging
debug_manager.start_debugging();

// Stop debugging
debug_manager.stop_debugging();

// Check if debugging is active
if debug_manager.is_debugging_active() {
    // Debugging is active
}

// Check if execution is paused
if debug_manager.is_execution_paused() {
    // Execution is paused
}

// Get the current debug state
let state = debug_manager.get_debug_state();
```

### AST Stepper

The AST Stepper is responsible for stepping through AST nodes during execution. It manages breakpoints, tracks execution history, and provides different stepping modes.

```rust
// Set a breakpoint
let location = SourceLocation {
    file: "main.ai".to_string(),
    line: 10,
    column: 5,
};
let breakpoint_id = debug_manager.set_breakpoint(location);

// Remove a breakpoint
debug_manager.remove_breakpoint(breakpoint_id);

// Enable a breakpoint
debug_manager.enable_breakpoint(breakpoint_id);

// Disable a breakpoint
debug_manager.disable_breakpoint(breakpoint_id);

// Step into the next node
debug_manager.step_into();

// Step over the next node
debug_manager.step_over();

// Step out of the current function
debug_manager.step_out();

// Continue execution
debug_manager.continue_execution();
```

### Variable Tracker

The Variable Tracker tracks variable states throughout program execution. It manages scopes, supports watch expressions, and provides variable history.

```rust
// Create a scope
let scope_id = debug_manager.create_scope("main", None);

// Create a child scope
let child_scope_id = debug_manager.create_scope("function", Some(scope_id));

// Enter a scope
debug_manager.enter_scope(scope_id);

// Exit the current scope
let exited_scope_id = debug_manager.exit_scope();

// Get the value of a variable
let value = debug_manager.get_variable_value("x");

// Add a watch expression
let watch_id = debug_manager.add_watch("x > 10");

// Remove a watch expression
debug_manager.remove_watch(watch_id);
```

### Error Analyzer

The Error Analyzer analyzes errors and provides detailed diagnostics. It classifies errors, manages stack traces, and matches errors against known patterns.

```rust
// Get error details
let error_details = debug_manager.get_error_details(&error);

// Access error information
if let Some(details) = error_details {
    println!("Error type: {:?}", details.error_info.error_type);
    println!("Error location: {:?}", details.error_info.location);
    println!("Stack trace: {:?}", details.error_info.stack_trace);
    println!("Description: {}", details.description);
    println!("Common causes:");
    for cause in &details.common_causes {
        println!("- {}", cause);
    }
}
```

### Fix Suggester

The Fix Suggester suggests fixes for common errors. It matches errors against known patterns and generates fix suggestions based on the error context.

```rust
// Get fix suggestions for an error
let suggestions = debug_manager.get_fix_suggestions(&error);

// Apply a fix suggestion
if let Some(suggestion) = suggestions.first() {
    match debug_manager.apply_fix(suggestion) {
        Ok(()) => println!("Fix applied successfully"),
        Err(e) => println!("Failed to apply fix: {:?}", e),
    }
}

// Access suggestion information
for suggestion in &suggestions {
    println!("Description: {}", suggestion.description);
    println!("Explanation: {}", suggestion.explanation);
    println!("Confidence: {:?}", suggestion.confidence);
    println!("Code change: {:?}", suggestion.code_change);
}
```

## Event System

The Debug Agent provides an event system that allows clients to subscribe to debugging events. This is useful for building user interfaces or integrating with other tools.

```rust
// Add an event listener
debug_manager.add_event_listener(|event| {
    match event {
        DebugEvent::ExecutionPaused { location, reason } => {
            println!("Execution paused at {:?} due to {:?}", location, reason);
        },
        DebugEvent::ExecutionResumed => {
            println!("Execution resumed");
        },
        DebugEvent::VariableChanged { name, old_value, new_value } => {
            println!("Variable {} changed from {:?} to {:?}", name, old_value, new_value);
        },
        DebugEvent::ErrorOccurred { error, details } => {
            println!("Error occurred: {:?}", error);
        },
        DebugEvent::FixSuggested { error, suggestions } => {
            println!("Fix suggested for error: {:?}", error);
        },
        _ => {},
    }
});
```

## Integration with Interpreter

The Debug Agent integrates with the Anarchy Inference interpreter through a set of hooks at key execution points. These hooks are called by the interpreter during execution and allow the Debug Agent to monitor and control the execution process.

```rust
// In interpreter.rs
pub struct Interpreter {
    // Existing fields...
    
    // Debug manager (optional, only created when debugging is enabled)
    debug_manager: Option<DebugManager>,
}

impl Interpreter {
    // Hook before executing an AST node
    fn before_execute_node(&mut self, node: &AstNode) {
        if let Some(debug) = &mut self.debug_manager {
            debug.before_node_execution(node);
        }
    }
    
    // Hook after executing an AST node
    fn after_execute_node(&mut self, node: &AstNode, result: &Result<Value, Error>) {
        if let Some(debug) = &mut self.debug_manager {
            debug.after_node_execution(node, result);
        }
    }
    
    // Hook when an error occurs
    fn on_error(&mut self, error: &Error) {
        if let Some(debug) = &mut self.debug_manager {
            debug.on_error(error);
        }
    }
}
```

## Configuration

The Debug Agent can be configured through the `DebugConfig` struct. This allows you to customize the behavior of the Debug Agent to suit your needs.

```rust
// Create a custom configuration
let config = DebugConfig {
    max_history_size: 200,
    enable_ast_stepping: true,
    enable_variable_tracking: true,
    enable_error_analysis: true,
    enable_fix_suggestions: true,
};

// Create a debug manager with the custom configuration
let mut debug_manager = DebugManager::new(config);
```

## Usage Examples

### Basic Debugging Session

```rust
// Create a debug manager
let config = DebugConfig::default();
let mut debug_manager = DebugManager::new(config);

// Start debugging
debug_manager.start_debugging();

// Set a breakpoint
let location = SourceLocation {
    file: "main.ai".to_string(),
    line: 10,
    column: 5,
};
let breakpoint_id = debug_manager.set_breakpoint(location);

// Run the program
// When the breakpoint is hit, execution will pause

// Step through the code
debug_manager.step_over();
debug_manager.step_over();
debug_manager.step_into();

// Inspect variables
let x_value = debug_manager.get_variable_value("x");
println!("x = {:?}", x_value);

// Continue execution
debug_manager.continue_execution();

// Stop debugging
debug_manager.stop_debugging();
```

### Error Analysis and Fix Suggestions

```rust
// Create a debug manager
let config = DebugConfig::default();
let mut debug_manager = DebugManager::new(config);

// Start debugging
debug_manager.start_debugging();

// Run the program
// When an error occurs, it will be analyzed

// Get error details
let error_details = debug_manager.get_error_details(&error);
if let Some(details) = error_details {
    println!("Error type: {:?}", details.error_info.error_type);
    println!("Description: {}", details.description);
}

// Get fix suggestions
let suggestions = debug_manager.get_fix_suggestions(&error);
for suggestion in &suggestions {
    println!("Suggestion: {}", suggestion.description);
    println!("Explanation: {}", suggestion.explanation);
}

// Apply a fix
if let Some(suggestion) = suggestions.first() {
    match debug_manager.apply_fix(suggestion) {
        Ok(()) => println!("Fix applied successfully"),
        Err(e) => println!("Failed to apply fix: {:?}", e),
    }
}
```

### Watch Expressions

```rust
// Create a debug manager
let config = DebugConfig::default();
let mut debug_manager = DebugManager::new(config);

// Start debugging
debug_manager.start_debugging();

// Add watch expressions
let watch1_id = debug_manager.add_watch("x");
let watch2_id = debug_manager.add_watch("y > 10");

// Add an event listener for watch triggers
debug_manager.add_event_listener(|event| {
    if let DebugEvent::WatchTriggered { id, value } = event {
        println!("Watch triggered: {:?} = {:?}", id, value);
    }
});

// Run the program
// When a watched variable changes, the event listener will be called
```

## Best Practices

1. **Start with Breakpoints**: Set breakpoints at key points in your code to pause execution and inspect the state.
2. **Use Step Over for Most Cases**: Use step over to navigate through your code without diving into function calls.
3. **Use Step Into for Debugging Functions**: Use step into when you want to debug a function call.
4. **Use Watch Expressions**: Add watch expressions for variables you want to monitor.
5. **Check Error Details**: When an error occurs, check the error details for a better understanding of the issue.
6. **Consider Fix Suggestions**: Review fix suggestions carefully before applying them.
7. **Use Event Listeners**: Add event listeners to respond to debugging events in your UI.

## Conclusion

The Debug Agent provides a powerful set of tools for debugging Anarchy Inference programs. By using the Debug Agent, you can step through code, inspect variables, analyze errors, and get fix suggestions, making the debugging process more efficient and effective.
