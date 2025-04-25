// Debug Agent - Error Trace Analysis Module
// This module provides functionality for analyzing errors and providing detailed diagnostics

use crate::error::Error;
use crate::ast::AstNode;
use std::collections::{HashMap, VecDeque};
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

/// Error type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    Syntax,
    Type,
    Reference,
    Runtime,
    Logic,
    Custom(String),
}

/// Detailed error information
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error: Error,
    pub error_type: ErrorType,
    pub location: Option<SourceLocation>,
    pub timestamp: u64,
    pub stack_trace: StackTrace,
    pub context: ErrorContext,
}

/// Stack trace information
#[derive(Debug, Clone)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub location: Option<SourceLocation>,
    pub locals: HashMap<String, String>, // Variable name -> string representation of value
}

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub code_snippet: Option<String>,
    pub relevant_variables: HashMap<String, String>, // Variable name -> string representation of value
    pub previous_operations: Vec<String>,
}

/// Error pattern for recognition
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub pattern_id: String,
    pub error_type: ErrorType,
    pub message_pattern: String,
    pub code_pattern: Option<String>,
    pub description: String,
    pub common_causes: Vec<String>,
}

/// Error Analyzer component
pub struct ErrorAnalyzer {
    /// Error history
    error_history: VecDeque<ErrorInfo>,
    /// Maximum history size
    max_history_size: usize,
    /// Error patterns database
    error_patterns: HashMap<String, ErrorPattern>,
    /// Current timestamp
    timestamp: u64,
    /// Current stack trace
    current_stack_trace: StackTrace,
}

impl ErrorAnalyzer {
    /// Create a new error analyzer
    pub fn new(max_history_size: usize) -> Self {
        Self {
            error_history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            error_patterns: Self::initialize_error_patterns(),
            timestamp: 0,
            current_stack_trace: StackTrace { frames: Vec::new() },
        }
    }

    /// Initialize the error patterns database
    fn initialize_error_patterns() -> HashMap<String, ErrorPattern> {
        let mut patterns = HashMap::new();
        
        // Add some common error patterns
        
        // Undefined variable
        patterns.insert(
            "undefined_variable".to_string(),
            ErrorPattern {
                pattern_id: "undefined_variable".to_string(),
                error_type: ErrorType::Reference,
                message_pattern: "undefined variable".to_string(),
                code_pattern: None,
                description: "Attempted to use a variable that has not been defined".to_string(),
                common_causes: vec![
                    "Typo in variable name".to_string(),
                    "Using a variable before it's defined".to_string(),
                    "Variable defined in a different scope".to_string(),
                ],
            },
        );
        
        // Type error
        patterns.insert(
            "type_error".to_string(),
            ErrorPattern {
                pattern_id: "type_error".to_string(),
                error_type: ErrorType::Type,
                message_pattern: "expected type".to_string(),
                code_pattern: None,
                description: "Operation performed on a value of the wrong type".to_string(),
                common_causes: vec![
                    "Using a string where a number is expected".to_string(),
                    "Using a number where a string is expected".to_string(),
                    "Using a collection where a scalar is expected".to_string(),
                ],
            },
        );
        
        // Syntax error
        patterns.insert(
            "syntax_error".to_string(),
            ErrorPattern {
                pattern_id: "syntax_error".to_string(),
                error_type: ErrorType::Syntax,
                message_pattern: "syntax error".to_string(),
                code_pattern: None,
                description: "Code contains invalid syntax".to_string(),
                common_causes: vec![
                    "Missing closing parenthesis or bracket".to_string(),
                    "Missing semicolon".to_string(),
                    "Invalid operator usage".to_string(),
                ],
            },
        );
        
        patterns
    }

    /// Called when an error occurs
    pub fn on_error(&mut self, error: &Error, node: Option<&AstNode>) -> ErrorInfo {
        self.timestamp += 1;
        
        // Classify the error
        let error_type = self.classify_error(error);
        
        // Get the location
        let location = match node {
            Some(node) => self.get_node_location(node),
            None => None,
        };
        
        // Create error context
        let context = self.create_error_context(node);
        
        // Create error info
        let error_info = ErrorInfo {
            error: error.clone(),
            error_type,
            location,
            timestamp: self.timestamp,
            stack_trace: self.current_stack_trace.clone(),
            context,
        };
        
        // Add to history
        self.error_history.push_back(error_info.clone());
        
        // Trim history if it exceeds the maximum size
        while self.error_history.len() > self.max_history_size {
            self.error_history.pop_front();
        }
        
        error_info
    }
    
    /// Classify an error
    fn classify_error(&self, error: &Error) -> ErrorType {
        // TODO: Implement proper error classification based on error type and message
        // For now, use a simple heuristic based on the error message
        
        let error_message = format!("{:?}", error).to_lowercase();
        
        if error_message.contains("syntax") {
            ErrorType::Syntax
        } else if error_message.contains("type") {
            ErrorType::Type
        } else if error_message.contains("undefined") || error_message.contains("not found") {
            ErrorType::Reference
        } else {
            ErrorType::Runtime
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
    
    /// Create error context
    fn create_error_context(&self, node: Option<&AstNode>) -> ErrorContext {
        // TODO: Extract context from node and current state
        // For now, return an empty context
        ErrorContext {
            code_snippet: node.map(|n| format!("{:?}", n)),
            relevant_variables: HashMap::new(),
            previous_operations: Vec::new(),
        }
    }
    
    /// Enter a function
    pub fn enter_function(&mut self, function_name: &str, location: Option<SourceLocation>) {
        let frame = StackFrame {
            function_name: function_name.to_string(),
            location,
            locals: HashMap::new(),
        };
        
        self.current_stack_trace.frames.push(frame);
    }
    
    /// Exit a function
    pub fn exit_function(&mut self) {
        if !self.current_stack_trace.frames.is_empty() {
            self.current_stack_trace.frames.pop();
        }
    }
    
    /// Add a local variable to the current stack frame
    pub fn add_local_variable(&mut self, name: &str, value: &str) {
        if let Some(frame) = self.current_stack_trace.frames.last_mut() {
            frame.locals.insert(name.to_string(), value.to_string());
        }
    }
    
    /// Get the error history
    pub fn get_error_history(&self) -> &VecDeque<ErrorInfo> {
        &self.error_history
    }
    
    /// Get a specific error from history
    pub fn get_error(&self, timestamp: u64) -> Option<&ErrorInfo> {
        self.error_history.iter().find(|e| e.timestamp == timestamp)
    }
    
    /// Get the current stack trace
    pub fn get_current_stack_trace(&self) -> &StackTrace {
        &self.current_stack_trace
    }
    
    /// Match an error against known patterns
    pub fn match_error_patterns(&self, error_info: &ErrorInfo) -> Vec<&ErrorPattern> {
        let mut matches = Vec::new();
        
        let error_message = format!("{:?}", error_info.error).to_lowercase();
        
        for pattern in self.error_patterns.values() {
            if error_info.error_type == pattern.error_type && error_message.contains(&pattern.message_pattern.to_lowercase()) {
                matches.push(pattern);
            }
        }
        
        matches
    }
    
    /// Get detailed error analysis
    pub fn analyze_error(&self, error_info: &ErrorInfo) -> ErrorAnalysis {
        let matched_patterns = self.match_error_patterns(error_info);
        
        let description = if let Some(pattern) = matched_patterns.first() {
            pattern.description.clone()
        } else {
            "Unknown error".to_string()
        };
        
        let common_causes = matched_patterns
            .iter()
            .flat_map(|p| p.common_causes.clone())
            .collect();
        
        ErrorAnalysis {
            error_info: error_info.clone(),
            matched_patterns: matched_patterns.iter().map(|p| p.pattern_id.clone()).collect(),
            description,
            common_causes,
            suggested_fixes: Vec::new(), // Will be filled by the FixSuggester
        }
    }
    
    /// Clear the error history
    pub fn clear_history(&mut self) {
        self.error_history.clear();
    }
    
    /// Reset the stack trace
    pub fn reset_stack_trace(&mut self) {
        self.current_stack_trace.frames.clear();
    }
    
    /// Add a custom error pattern
    pub fn add_error_pattern(&mut self, pattern: ErrorPattern) {
        self.error_patterns.insert(pattern.pattern_id.clone(), pattern);
    }
    
    /// Remove an error pattern
    pub fn remove_error_pattern(&mut self, pattern_id: &str) -> bool {
        self.error_patterns.remove(pattern_id).is_some()
    }
    
    /// Get all error patterns
    pub fn get_error_patterns(&self) -> Vec<&ErrorPattern> {
        self.error_patterns.values().collect()
    }
}

/// Error analysis result
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub error_info: ErrorInfo,
    pub matched_patterns: Vec<String>,
    pub description: String,
    pub common_causes: Vec<String>,
    pub suggested_fixes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Add tests for error analysis
}
