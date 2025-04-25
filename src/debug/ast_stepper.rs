// Debug Agent - AST Stepping and Inspection Module
// This module provides functionality for stepping through AST nodes during execution

use crate::ast::{AstNode, AstNodeRef};
use crate::error::Error;
use crate::value::Value;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Source location in code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Breakpoint information
#[derive(Debug, Clone)]
pub struct BreakpointInfo {
    pub id: BreakpointId,
    pub location: SourceLocation,
    pub enabled: bool,
    pub condition: Option<String>,
    pub hit_count: usize,
    pub hit_condition: Option<HitCondition>,
}

/// Unique identifier for breakpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BreakpointId(pub usize);

/// Hit condition for conditional breakpoints
#[derive(Debug, Clone)]
pub enum HitCondition {
    Equal(usize),
    GreaterThan(usize),
    GreaterThanOrEqual(usize),
    LessThan(usize),
    LessThanOrEqual(usize),
    Multiple(usize),
}

impl HitCondition {
    /// Check if the hit condition is satisfied
    pub fn is_satisfied(&self, hit_count: usize) -> bool {
        match self {
            HitCondition::Equal(n) => hit_count == *n,
            HitCondition::GreaterThan(n) => hit_count > *n,
            HitCondition::GreaterThanOrEqual(n) => hit_count >= *n,
            HitCondition::LessThan(n) => hit_count < *n,
            HitCondition::LessThanOrEqual(n) => hit_count <= *n,
            HitCondition::Multiple(n) => *n != 0 && hit_count % *n == 0,
        }
    }
}

/// Reason for execution pause
#[derive(Debug, Clone)]
pub enum PauseReason {
    Breakpoint(BreakpointId),
    Step,
    Exception(Error),
    UserRequest,
    WatchTriggered(WatchId),
}

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
}

/// Execution mode for stepping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// Execute until the next node in the current scope
    StepOver,
    /// Execute until the next node, including entering function calls
    StepInto,
    /// Execute until returning from the current function
    StepOut,
    /// Execute until a breakpoint is hit
    Continue,
}

/// AST Stepper component
pub struct AstStepper {
    /// Current position in AST
    current_node: Option<AstNodeRef>,
    /// Current step mode
    step_mode: StepMode,
    /// Current scope depth
    scope_depth: usize,
    /// Target scope depth for StepOut
    target_scope_depth: Option<usize>,
    /// Execution history
    execution_history: Vec<AstNodeRef>,
    /// Maximum history size
    max_history_size: usize,
    /// Breakpoints
    breakpoints: HashMap<SourceLocation, BreakpointInfo>,
    /// Next breakpoint ID
    next_breakpoint_id: usize,
    /// Whether execution is currently paused
    paused: bool,
    /// Reason for the current pause
    pause_reason: Option<PauseReason>,
}

impl AstStepper {
    /// Create a new AST stepper
    pub fn new(max_history_size: usize) -> Self {
        Self {
            current_node: None,
            step_mode: StepMode::Continue,
            scope_depth: 0,
            target_scope_depth: None,
            execution_history: Vec::with_capacity(max_history_size),
            max_history_size,
            breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
            paused: false,
            pause_reason: None,
        }
    }

    /// Set the current step mode
    pub fn set_step_mode(&mut self, mode: StepMode) {
        self.step_mode = mode;
        
        // If stepping out, set the target scope depth
        if mode == StepMode::StepOut {
            self.target_scope_depth = Some(self.scope_depth.saturating_sub(1));
        } else {
            self.target_scope_depth = None;
        }
        
        // Resume execution
        self.paused = false;
        self.pause_reason = None;
    }

    /// Called before executing an AST node
    pub fn before_node_execution(&mut self, node: &AstNode) -> bool {
        // Update current node
        self.current_node = Some(Rc::new(node.clone()));
        
        // Check if we should pause at this node
        let should_pause = self.should_pause_at_node(node);
        
        // If we should pause, set the pause state
        if should_pause {
            self.paused = true;
            
            // Set the pause reason based on the current step mode
            if self.pause_reason.is_none() {
                self.pause_reason = Some(PauseReason::Step);
            }
        }
        
        // Add to execution history
        self.add_to_history(Rc::new(node.clone()));
        
        // Return whether execution should pause
        should_pause
    }
    
    /// Called after executing an AST node
    pub fn after_node_execution(&mut self, node: &AstNode, result: &Result<Value, Error>) {
        // If there was an error, we might want to pause
        if let Err(error) = result {
            self.paused = true;
            self.pause_reason = Some(PauseReason::Exception(error.clone()));
        }
        
        // Update scope depth based on node type
        match node {
            AstNode::FunctionDeclaration { .. } | AstNode::Block { .. } => {
                // Exiting a scope
                if self.scope_depth > 0 {
                    self.scope_depth -= 1;
                }
                
                // Check if we've reached the target scope depth for StepOut
                if let Some(target_depth) = self.target_scope_depth {
                    if self.scope_depth <= target_depth {
                        self.paused = true;
                        self.pause_reason = Some(PauseReason::Step);
                        self.target_scope_depth = None;
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Called when entering a new scope
    pub fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }
    
    /// Called when exiting a scope
    pub fn exit_scope(&mut self) {
        if self.scope_depth > 0 {
            self.scope_depth -= 1;
        }
        
        // Check if we've reached the target scope depth for StepOut
        if let Some(target_depth) = self.target_scope_depth {
            if self.scope_depth <= target_depth {
                self.paused = true;
                self.pause_reason = Some(PauseReason::Step);
                self.target_scope_depth = None;
            }
        }
    }
    
    /// Add a node to the execution history
    fn add_to_history(&mut self, node: AstNodeRef) {
        self.execution_history.push(node);
        
        // Trim history if it exceeds the maximum size
        if self.execution_history.len() > self.max_history_size {
            self.execution_history.remove(0);
        }
    }
    
    /// Check if execution should pause at the given node
    fn should_pause_at_node(&mut self, node: &AstNode) -> bool {
        // If already paused, stay paused
        if self.paused {
            return true;
        }
        
        // Check if there's a breakpoint at this location
        if let Some(location) = self.get_node_location(node) {
            if let Some(breakpoint) = self.breakpoints.get_mut(&location) {
                if breakpoint.enabled {
                    // Increment hit count
                    breakpoint.hit_count += 1;
                    
                    // Check hit condition if present
                    let hit_condition_satisfied = match &breakpoint.hit_condition {
                        Some(condition) => condition.is_satisfied(breakpoint.hit_count),
                        None => true,
                    };
                    
                    // Check condition if present
                    let condition_satisfied = match &breakpoint.condition {
                        Some(_condition) => {
                            // TODO: Evaluate condition expression
                            // For now, assume conditions are always satisfied
                            true
                        }
                        None => true,
                    };
                    
                    if hit_condition_satisfied && condition_satisfied {
                        self.pause_reason = Some(PauseReason::Breakpoint(breakpoint.id));
                        return true;
                    }
                }
            }
        }
        
        // Check step mode
        match self.step_mode {
            StepMode::StepInto => true,
            StepMode::StepOver => {
                // Only pause if we're at the same scope depth or lower
                let node_increases_depth = match node {
                    AstNode::FunctionDeclaration { .. } | AstNode::Block { .. } => true,
                    _ => false,
                };
                
                !node_increases_depth
            }
            StepMode::StepOut => false, // Handled in after_node_execution
            StepMode::Continue => false,
        }
    }
    
    /// Get the source location of a node
    fn get_node_location(&self, node: &AstNode) -> Option<SourceLocation> {
        // TODO: Extract location from node
        // For now, return a dummy location
        Some(SourceLocation {
            file: "main.ai".to_string(),
            line: 1,
            column: 1,
        })
    }
    
    /// Set a breakpoint at the given location
    pub fn set_breakpoint(&mut self, location: SourceLocation) -> BreakpointId {
        let id = BreakpointId(self.next_breakpoint_id);
        self.next_breakpoint_id += 1;
        
        let breakpoint = BreakpointInfo {
            id,
            location: location.clone(),
            enabled: true,
            condition: None,
            hit_count: 0,
            hit_condition: None,
        };
        
        self.breakpoints.insert(location, breakpoint);
        
        id
    }
    
    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        let mut location_to_remove = None;
        
        for (loc, bp) in &self.breakpoints {
            if bp.id == id {
                location_to_remove = Some(loc.clone());
                break;
            }
        }
        
        if let Some(location) = location_to_remove {
            self.breakpoints.remove(&location);
            true
        } else {
            false
        }
    }
    
    /// Enable a breakpoint
    pub fn enable_breakpoint(&mut self, id: BreakpointId) -> bool {
        for bp in self.breakpoints.values_mut() {
            if bp.id == id {
                bp.enabled = true;
                return true;
            }
        }
        
        false
    }
    
    /// Disable a breakpoint
    pub fn disable_breakpoint(&mut self, id: BreakpointId) -> bool {
        for bp in self.breakpoints.values_mut() {
            if bp.id == id {
                bp.enabled = false;
                return true;
            }
        }
        
        false
    }
    
    /// Set a condition on a breakpoint
    pub fn set_breakpoint_condition(&mut self, id: BreakpointId, condition: Option<String>) -> bool {
        for bp in self.breakpoints.values_mut() {
            if bp.id == id {
                bp.condition = condition;
                return true;
            }
        }
        
        false
    }
    
    /// Set a hit condition on a breakpoint
    pub fn set_breakpoint_hit_condition(&mut self, id: BreakpointId, condition: Option<HitCondition>) -> bool {
        for bp in self.breakpoints.values_mut() {
            if bp.id == id {
                bp.hit_condition = condition;
                return true;
            }
        }
        
        false
    }
    
    /// Get all breakpoints
    pub fn get_breakpoints(&self) -> Vec<&BreakpointInfo> {
        self.breakpoints.values().collect()
    }
    
    /// Get a specific breakpoint
    pub fn get_breakpoint(&self, id: BreakpointId) -> Option<&BreakpointInfo> {
        for bp in self.breakpoints.values() {
            if bp.id == id {
                return Some(bp);
            }
        }
        
        None
    }
    
    /// Get the current node
    pub fn get_current_node(&self) -> Option<AstNodeRef> {
        self.current_node.clone()
    }
    
    /// Get the execution history
    pub fn get_execution_history(&self) -> &[AstNodeRef] {
        &self.execution_history
    }
    
    /// Check if execution is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    /// Get the reason for the current pause
    pub fn get_pause_reason(&self) -> Option<&PauseReason> {
        self.pause_reason.as_ref()
    }
    
    /// Resume execution
    pub fn resume(&mut self) {
        self.paused = false;
        self.pause_reason = None;
        self.step_mode = StepMode::Continue;
    }
    
    /// Step into the next node
    pub fn step_into(&mut self) {
        self.set_step_mode(StepMode::StepInto);
    }
    
    /// Step over the next node
    pub fn step_over(&mut self) {
        self.set_step_mode(StepMode::StepOver);
    }
    
    /// Step out of the current function
    pub fn step_out(&mut self) {
        self.set_step_mode(StepMode::StepOut);
    }
    
    /// Continue execution until the next breakpoint
    pub fn continue_execution(&mut self) {
        self.set_step_mode(StepMode::Continue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Add tests for AST stepping
}
