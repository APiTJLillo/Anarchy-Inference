// Debug Agent - Variable State Tracking Module
// This module provides functionality for tracking variable states during execution

use crate::value::Value;
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Unique identifier for watch expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WatchId(pub usize);

/// Watch expression
#[derive(Debug, Clone)]
pub struct WatchExpression {
    pub id: WatchId,
    pub expression: String,
    pub condition: Option<String>,
    pub enabled: bool,
    pub last_value: Option<Value>,
}

/// Variable change event
#[derive(Debug, Clone)]
pub struct VariableChangeEvent {
    pub name: String,
    pub old_value: Option<Value>,
    pub new_value: Value,
    pub scope_id: ScopeId,
    pub timestamp: u64,
}

/// Scope identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

/// Scope information
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub id: ScopeId,
    pub name: String,
    pub parent_id: Option<ScopeId>,
    pub depth: usize,
    pub variables: HashMap<String, Value>,
}

/// Variable state snapshot
#[derive(Debug, Clone)]
pub struct VariableStateSnapshot {
    pub timestamp: u64,
    pub scopes: HashMap<ScopeId, ScopeInfo>,
    pub global_variables: HashMap<String, Value>,
}

/// Variable Tracker component
pub struct VariableTracker {
    /// Current variable states by scope
    scopes: HashMap<ScopeId, ScopeInfo>,
    /// Global variables
    global_variables: HashMap<String, Value>,
    /// Current active scope
    current_scope_id: Option<ScopeId>,
    /// Variable history
    state_history: VecDeque<VariableStateSnapshot>,
    /// Maximum history size
    max_history_size: usize,
    /// Watch expressions
    watches: HashMap<WatchId, WatchExpression>,
    /// Next watch ID
    next_watch_id: usize,
    /// Next scope ID
    next_scope_id: usize,
    /// Current timestamp
    timestamp: u64,
    /// Variable change listeners
    change_listeners: Vec<Box<dyn Fn(&VariableChangeEvent)>>,
}

impl VariableTracker {
    /// Create a new variable tracker
    pub fn new(max_history_size: usize) -> Self {
        Self {
            scopes: HashMap::new(),
            global_variables: HashMap::new(),
            current_scope_id: None,
            state_history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            watches: HashMap::new(),
            next_watch_id: 1,
            next_scope_id: 1,
            timestamp: 0,
            change_listeners: Vec::new(),
        }
    }

    /// Create a new scope
    pub fn create_scope(&mut self, name: &str, parent_id: Option<ScopeId>) -> ScopeId {
        let id = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        
        let depth = match parent_id {
            Some(parent_id) => {
                if let Some(parent) = self.scopes.get(&parent_id) {
                    parent.depth + 1
                } else {
                    0
                }
            }
            None => 0,
        };
        
        let scope = ScopeInfo {
            id,
            name: name.to_string(),
            parent_id,
            depth,
            variables: HashMap::new(),
        };
        
        self.scopes.insert(id, scope);
        self.current_scope_id = Some(id);
        
        id
    }
    
    /// Enter an existing scope
    pub fn enter_scope(&mut self, id: ScopeId) -> bool {
        if self.scopes.contains_key(&id) {
            self.current_scope_id = Some(id);
            true
        } else {
            false
        }
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Option<ScopeId> {
        if let Some(current_id) = self.current_scope_id {
            if let Some(scope) = self.scopes.get(&current_id) {
                self.current_scope_id = scope.parent_id;
                return Some(current_id);
            }
        }
        
        None
    }
    
    /// Get the current scope
    pub fn get_current_scope(&self) -> Option<&ScopeInfo> {
        self.current_scope_id.and_then(|id| self.scopes.get(&id))
    }
    
    /// Get a specific scope
    pub fn get_scope(&self, id: ScopeId) -> Option<&ScopeInfo> {
        self.scopes.get(&id)
    }
    
    /// Get all scopes
    pub fn get_all_scopes(&self) -> Vec<&ScopeInfo> {
        self.scopes.values().collect()
    }
    
    /// Set a variable in the current scope
    pub fn set_variable(&mut self, name: &str, value: Value) -> Option<Value> {
        self.timestamp += 1;
        
        if let Some(scope_id) = self.current_scope_id {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                let old_value = scope.variables.get(name).cloned();
                let result = scope.variables.insert(name.to_string(), value.clone());
                
                // Notify listeners
                let event = VariableChangeEvent {
                    name: name.to_string(),
                    old_value,
                    new_value: value,
                    scope_id,
                    timestamp: self.timestamp,
                };
                
                self.notify_variable_change(&event);
                
                // Check watches
                self.check_watches(name);
                
                // Take a snapshot
                self.take_snapshot();
                
                return result;
            }
        }
        
        // If no scope is active, set in global variables
        let old_value = self.global_variables.get(name).cloned();
        let result = self.global_variables.insert(name.to_string(), value.clone());
        
        // Notify listeners
        let event = VariableChangeEvent {
            name: name.to_string(),
            old_value,
            new_value: value,
            scope_id: ScopeId(0), // Global scope
            timestamp: self.timestamp,
        };
        
        self.notify_variable_change(&event);
        
        // Check watches
        self.check_watches(name);
        
        // Take a snapshot
        self.take_snapshot();
        
        result
    }
    
    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // First check current scope and its parents
        if let Some(scope_id) = self.current_scope_id {
            let mut current_id = Some(scope_id);
            
            while let Some(id) = current_id {
                if let Some(scope) = self.scopes.get(&id) {
                    if let Some(value) = scope.variables.get(name) {
                        return Some(value.clone());
                    }
                    
                    current_id = scope.parent_id;
                } else {
                    break;
                }
            }
        }
        
        // Then check global variables
        self.global_variables.get(name).cloned()
    }
    
    /// Take a snapshot of the current variable state
    fn take_snapshot(&mut self) {
        let snapshot = VariableStateSnapshot {
            timestamp: self.timestamp,
            scopes: self.scopes.clone(),
            global_variables: self.global_variables.clone(),
        };
        
        self.state_history.push_back(snapshot);
        
        // Trim history if it exceeds the maximum size
        while self.state_history.len() > self.max_history_size {
            self.state_history.pop_front();
        }
    }
    
    /// Get the variable state history
    pub fn get_state_history(&self) -> &VecDeque<VariableStateSnapshot> {
        &self.state_history
    }
    
    /// Get a specific snapshot from history
    pub fn get_snapshot(&self, timestamp: u64) -> Option<&VariableStateSnapshot> {
        self.state_history.iter().find(|s| s.timestamp == timestamp)
    }
    
    /// Add a watch expression
    pub fn add_watch(&mut self, expression: &str) -> WatchId {
        let id = WatchId(self.next_watch_id);
        self.next_watch_id += 1;
        
        let watch = WatchExpression {
            id,
            expression: expression.to_string(),
            condition: None,
            enabled: true,
            last_value: None,
        };
        
        self.watches.insert(id, watch);
        
        id
    }
    
    /// Remove a watch expression
    pub fn remove_watch(&mut self, id: WatchId) -> bool {
        self.watches.remove(&id).is_some()
    }
    
    /// Enable a watch expression
    pub fn enable_watch(&mut self, id: WatchId) -> bool {
        if let Some(watch) = self.watches.get_mut(&id) {
            watch.enabled = true;
            true
        } else {
            false
        }
    }
    
    /// Disable a watch expression
    pub fn disable_watch(&mut self, id: WatchId) -> bool {
        if let Some(watch) = self.watches.get_mut(&id) {
            watch.enabled = false;
            true
        } else {
            false
        }
    }
    
    /// Set a condition on a watch expression
    pub fn set_watch_condition(&mut self, id: WatchId, condition: Option<String>) -> bool {
        if let Some(watch) = self.watches.get_mut(&id) {
            watch.condition = condition;
            true
        } else {
            false
        }
    }
    
    /// Get all watch expressions
    pub fn get_watches(&self) -> Vec<&WatchExpression> {
        self.watches.values().collect()
    }
    
    /// Get a specific watch expression
    pub fn get_watch(&self, id: WatchId) -> Option<&WatchExpression> {
        self.watches.get(&id)
    }
    
    /// Check if any watches match the given variable
    fn check_watches(&mut self, variable_name: &str) {
        for watch in self.watches.values_mut() {
            if !watch.enabled {
                continue;
            }
            
            // For now, just check if the watch expression exactly matches the variable name
            // In a real implementation, we would evaluate the expression
            if watch.expression == variable_name {
                if let Some(value) = self.get_variable(variable_name) {
                    // Check if the value has changed
                    if watch.last_value.as_ref() != Some(&value) {
                        // Update the last value
                        watch.last_value = Some(value.clone());
                        
                        // TODO: Notify about watch triggered
                    }
                }
            }
        }
    }
    
    /// Add a variable change listener
    pub fn add_change_listener<F>(&mut self, listener: F)
    where
        F: Fn(&VariableChangeEvent) + 'static,
    {
        self.change_listeners.push(Box::new(listener));
    }
    
    /// Notify all listeners about a variable change
    fn notify_variable_change(&self, event: &VariableChangeEvent) {
        for listener in &self.change_listeners {
            listener(event);
        }
    }
    
    /// Clear all variable state
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.global_variables.clear();
        self.current_scope_id = None;
        self.state_history.clear();
        self.timestamp = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Add tests for variable tracking
}
