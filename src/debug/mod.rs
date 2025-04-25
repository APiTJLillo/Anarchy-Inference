// Debug Agent - Main Module
// This module provides the central Debug Manager that coordinates all debugging activities

use crate::ast::AstNode;
use crate::error::Error;
use crate::value::Value;
use std::fmt;

use crate::debug::ast_stepper::{AstStepper, BreakpointId, PauseReason, SourceLocation, StepMode, WatchId};
use crate::debug::variable_tracker::{VariableTracker, ScopeId};
use crate::debug::error_analyzer::{ErrorAnalyzer, ErrorInfo, ErrorAnalysis};
use crate::debug::fix_suggester::{FixSuggester, FixSuggestion, FixError};

/// Debug configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub max_history_size: usize,
    pub enable_ast_stepping: bool,
    pub enable_variable_tracking: bool,
    pub enable_error_analysis: bool,
    pub enable_fix_suggestions: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            enable_ast_stepping: true,
            enable_variable_tracking: true,
            enable_error_analysis: true,
            enable_fix_suggestions: true,
        }
    }
}

/// Debug state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugState {
    Inactive,
    Active,
    Paused,
}

/// Debug event
#[derive(Debug, Clone)]
pub enum DebugEvent {
    // Execution events
    ExecutionPaused { location: SourceLocation, reason: PauseReason },
    ExecutionResumed,
    ExecutionStepped { location: SourceLocation },
    
    // Variable events
    VariableChanged { name: String, old_value: Option<Value>, new_value: Value },
    WatchTriggered { id: WatchId, value: Value },
    
    // Error events
    ErrorOccurred { error: Error, details: ErrorAnalysis },
    FixSuggested { error: Error, suggestions: Vec<FixSuggestion> },
    FixApplied { suggestion: FixSuggestion, result: Result<(), FixError> },
}

/// Debug event listener
pub type DebugEventListener = Box<dyn Fn(&DebugEvent)>;

/// Debug Manager component
pub struct DebugManager {
    /// Configuration settings
    config: DebugConfig,
    /// Current debugging state
    state: DebugState,
    /// AST stepper component
    ast_stepper: AstStepper,
    /// Variable tracker component
    variable_tracker: VariableTracker,
    /// Error analyzer component
    error_analyzer: ErrorAnalyzer,
    /// Fix suggester component
    fix_suggester: FixSuggester,
    /// Event listeners
    event_listeners: Vec<DebugEventListener>,
}

impl DebugManager {
    /// Create a new debug manager
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config: config.clone(),
            state: DebugState::Inactive,
            ast_stepper: AstStepper::new(config.max_history_size),
            variable_tracker: VariableTracker::new(config.max_history_size),
            error_analyzer: ErrorAnalyzer::new(config.max_history_size),
            fix_suggester: FixSuggester::new(),
            event_listeners: Vec::new(),
        }
    }

    /// Start debugging
    pub fn start_debugging(&mut self) {
        self.state = DebugState::Active;
    }
    
    /// Stop debugging
    pub fn stop_debugging(&mut self) {
        self.state = DebugState::Inactive;
    }
    
    /// Check if debugging is active
    pub fn is_debugging_active(&self) -> bool {
        self.state != DebugState::Inactive
    }
    
    /// Check if execution is paused
    pub fn is_execution_paused(&self) -> bool {
        self.state == DebugState::Paused
    }
    
    /// Get the current debug state
    pub fn get_debug_state(&self) -> DebugState {
        self.state
    }
    
    /// Called before executing an AST node
    pub fn before_node_execution(&mut self, node: &AstNode) {
        if !self.is_debugging_active() || !self.config.enable_ast_stepping {
            return;
        }
        
        let should_pause = self.ast_stepper.before_node_execution(node);
        
        if should_pause {
            self.state = DebugState::Paused;
            
            if let Some(location) = self.ast_stepper.get_current_node().and_then(|n| {
                // In a real implementation, we would extract the location from the node
                // For now, use a dummy location
                Some(SourceLocation {
                    file: "main.ai".to_string(),
                    line: 1,
                    column: 1,
                })
            }) {
                let reason = self.ast_stepper.get_pause_reason().cloned().unwrap_or(PauseReason::UserRequest);
                
                self.emit_event(DebugEvent::ExecutionPaused {
                    location,
                    reason,
                });
            }
        }
    }
    
    /// Called after executing an AST node
    pub fn after_node_execution(&mut self, node: &AstNode, result: &Result<Value, Error>) {
        if !self.is_debugging_active() {
            return;
        }
        
        if self.config.enable_ast_stepping {
            self.ast_stepper.after_node_execution(node, result);
        }
        
        if let Err(error) = result {
            if self.config.enable_error_analysis {
                let error_info = self.error_analyzer.on_error(error, Some(node));
                
                let error_analysis = self.error_analyzer.analyze_error(&error_info);
                
                self.emit_event(DebugEvent::ErrorOccurred {
                    error: error.clone(),
                    details: error_analysis.clone(),
                });
                
                if self.config.enable_fix_suggestions {
                    let suggestions = self.fix_suggester.suggest_fixes(&error_analysis);
                    
                    if !suggestions.is_empty() {
                        self.emit_event(DebugEvent::FixSuggested {
                            error: error.clone(),
                            suggestions: suggestions.clone(),
                        });
                    }
                }
            }
        }
    }
    
    /// Called when an error occurs
    pub fn on_error(&mut self, error: &Error) {
        if !self.is_debugging_active() || !self.config.enable_error_analysis {
            return;
        }
        
        let error_info = self.error_analyzer.on_error(error, None);
        
        let error_analysis = self.error_analyzer.analyze_error(&error_info);
        
        self.emit_event(DebugEvent::ErrorOccurred {
            error: error.clone(),
            details: error_analysis.clone(),
        });
        
        if self.config.enable_fix_suggestions {
            let suggestions = self.fix_suggester.suggest_fixes(&error_analysis);
            
            if !suggestions.is_empty() {
                self.emit_event(DebugEvent::FixSuggested {
                    error: error.clone(),
                    suggestions: suggestions.clone(),
                });
            }
        }
    }
    
    /// Called when a variable changes
    pub fn on_variable_change(&mut self, name: &str, value: Value) {
        if !self.is_debugging_active() || !self.config.enable_variable_tracking {
            return;
        }
        
        let old_value = self.variable_tracker.get_variable(name);
        self.variable_tracker.set_variable(name, value.clone());
        
        self.emit_event(DebugEvent::VariableChanged {
            name: name.to_string(),
            old_value,
            new_value: value,
        });
    }
    
    /// Set a breakpoint
    pub fn set_breakpoint(&mut self, location: SourceLocation) -> BreakpointId {
        self.ast_stepper.set_breakpoint(location)
    }
    
    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        self.ast_stepper.remove_breakpoint(id)
    }
    
    /// Enable a breakpoint
    pub fn enable_breakpoint(&mut self, id: BreakpointId) -> bool {
        self.ast_stepper.enable_breakpoint(id)
    }
    
    /// Disable a breakpoint
    pub fn disable_breakpoint(&mut self, id: BreakpointId) -> bool {
        self.ast_stepper.disable_breakpoint(id)
    }
    
    /// Step into the next node
    pub fn step_into(&mut self) {
        if self.state == DebugState::Paused {
            self.ast_stepper.step_into();
            self.state = DebugState::Active;
            
            self.emit_event(DebugEvent::ExecutionResumed);
        }
    }
    
    /// Step over the next node
    pub fn step_over(&mut self) {
        if self.state == DebugState::Paused {
            self.ast_stepper.step_over();
            self.state = DebugState::Active;
            
            self.emit_event(DebugEvent::ExecutionResumed);
        }
    }
    
    /// Step out of the current function
    pub fn step_out(&mut self) {
        if self.state == DebugState::Paused {
            self.ast_stepper.step_out();
            self.state = DebugState::Active;
            
            self.emit_event(DebugEvent::ExecutionResumed);
        }
    }
    
    /// Continue execution
    pub fn continue_execution(&mut self) {
        if self.state == DebugState::Paused {
            self.ast_stepper.continue_execution();
            self.state = DebugState::Active;
            
            self.emit_event(DebugEvent::ExecutionResumed);
        }
    }
    
    /// Get the value of a variable
    pub fn get_variable_value(&self, name: &str) -> Option<Value> {
        if !self.is_debugging_active() || !self.config.enable_variable_tracking {
            return None;
        }
        
        self.variable_tracker.get_variable(name)
    }
    
    /// Add a watch expression
    pub fn add_watch(&mut self, expression: &str) -> WatchId {
        self.variable_tracker.add_watch(expression)
    }
    
    /// Remove a watch expression
    pub fn remove_watch(&mut self, id: WatchId) -> bool {
        self.variable_tracker.remove_watch(id)
    }
    
    /// Create a new scope
    pub fn create_scope(&mut self, name: &str, parent_id: Option<ScopeId>) -> ScopeId {
        self.variable_tracker.create_scope(name, parent_id)
    }
    
    /// Enter an existing scope
    pub fn enter_scope(&mut self, id: ScopeId) -> bool {
        self.variable_tracker.enter_scope(id)
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Option<ScopeId> {
        self.variable_tracker.exit_scope()
    }
    
    /// Get error details
    pub fn get_error_details(&self, error: &Error) -> Option<ErrorAnalysis> {
        if !self.is_debugging_active() || !self.config.enable_error_analysis {
            return None;
        }
        
        // Find the error info in the history
        let error_info = self.error_analyzer.get_error_history()
            .iter()
            .find(|info| format!("{:?}", info.error) == format!("{:?}", error))
            .cloned();
        
        error_info.map(|info| self.error_analyzer.analyze_error(&info))
    }
    
    /// Get fix suggestions for an error
    pub fn get_fix_suggestions(&mut self, error: &Error) -> Vec<FixSuggestion> {
        if !self.is_debugging_active() || !self.config.enable_fix_suggestions || !self.config.enable_error_analysis {
            return Vec::new();
        }
        
        // Find the error info in the history
        let error_info = self.error_analyzer.get_error_history()
            .iter()
            .find(|info| format!("{:?}", info.error) == format!("{:?}", error))
            .cloned();
        
        if let Some(info) = error_info {
            let analysis = self.error_analyzer.analyze_error(&info);
            self.fix_suggester.suggest_fixes(&analysis)
        } else {
            Vec::new()
        }
    }
    
    /// Apply a fix suggestion
    pub fn apply_fix(&mut self, suggestion: &FixSuggestion) -> Result<(), FixError> {
        if !self.is_debugging_active() || !self.config.enable_fix_suggestions {
            return Err(FixError::CannotGenerateFix);
        }
        
        let result = self.fix_suggester.apply_fix(suggestion);
        
        self.emit_event(DebugEvent::FixApplied {
            suggestion: suggestion.clone(),
            result: result.clone(),
        });
        
        result
    }
    
    /// Add an event listener
    pub fn add_event_listener<F>(&mut self, listener: F)
    where
        F: Fn(&DebugEvent) + 'static,
    {
        self.event_listeners.push(Box::new(listener));
    }
    
    /// Emit an event to all listeners
    fn emit_event(&self, event: DebugEvent) {
        for listener in &self.event_listeners {
            listener(&event);
        }
    }
    
    /// Get the AST stepper
    pub fn get_ast_stepper(&self) -> &AstStepper {
        &self.ast_stepper
    }
    
    /// Get the variable tracker
    pub fn get_variable_tracker(&self) -> &VariableTracker {
        &self.variable_tracker
    }
    
    /// Get the error analyzer
    pub fn get_error_analyzer(&self) -> &ErrorAnalyzer {
        &self.error_analyzer
    }
    
    /// Get the fix suggester
    pub fn get_fix_suggester(&self) -> &FixSuggester {
        &self.fix_suggester
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Add tests for debug manager
}
