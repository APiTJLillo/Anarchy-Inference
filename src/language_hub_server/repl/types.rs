// Types module for Advanced REPL Service
//
// This module defines common types used throughout the Advanced REPL Service.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Result value
    pub result: serde_json::Value,
    
    /// Execution output
    pub output: Option<String>,
    
    /// Execution duration in milliseconds
    pub duration: u64,
    
    /// Execution status
    pub status: String,
}

/// Execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOptions {
    /// Execution timeout in milliseconds
    pub timeout: u64,
    
    /// Whether to capture output
    pub capture_output: bool,
    
    /// Whether to execute asynchronously
    pub async_execution: bool,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        ExecutionOptions {
            timeout: 5000, // 5 seconds
            capture_output: true,
            async_execution: false,
        }
    }
}

/// Variable type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    /// Number
    #[serde(rename = "number")]
    Number,
    
    /// String
    #[serde(rename = "string")]
    String,
    
    /// Boolean
    #[serde(rename = "boolean")]
    Boolean,
    
    /// Array
    #[serde(rename = "array")]
    Array,
    
    /// Object
    #[serde(rename = "object")]
    Object,
    
    /// Function
    #[serde(rename = "function")]
    Function,
    
    /// Null
    #[serde(rename = "null")]
    Null,
    
    /// Undefined
    #[serde(rename = "undefined")]
    Undefined,
}

/// Variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    
    /// Variable type
    pub type_: VariableType,
    
    /// Variable value
    pub value: serde_json::Value,
    
    /// Variable size in bytes (approximate)
    pub size: usize,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Success
    #[serde(rename = "success")]
    Success,
    
    /// Error
    #[serde(rename = "error")]
    Error,
    
    /// Timeout
    #[serde(rename = "timeout")]
    Timeout,
    
    /// Cancelled
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl ToString for ExecutionStatus {
    fn to_string(&self) -> String {
        match self {
            ExecutionStatus::Success => "success".to_string(),
            ExecutionStatus::Error => "error".to_string(),
            ExecutionStatus::Timeout => "timeout".to_string(),
            ExecutionStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

/// Error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    /// Syntax error
    #[serde(rename = "syntax")]
    Syntax,
    
    /// Runtime error
    #[serde(rename = "runtime")]
    Runtime,
    
    /// Type error
    #[serde(rename = "type")]
    Type,
    
    /// Reference error
    #[serde(rename = "reference")]
    Reference,
    
    /// Range error
    #[serde(rename = "range")]
    Range,
    
    /// Internal error
    #[serde(rename = "internal")]
    Internal,
}

impl ToString for ErrorType {
    fn to_string(&self) -> String {
        match self {
            ErrorType::Syntax => "syntax".to_string(),
            ErrorType::Runtime => "runtime".to_string(),
            ErrorType::Type => "type".to_string(),
            ErrorType::Reference => "reference".to_string(),
            ErrorType::Range => "range".to_string(),
            ErrorType::Internal => "internal".to_string(),
        }
    }
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error type
    pub type_: ErrorType,
    
    /// Error message
    pub message: String,
    
    /// Error location
    pub location: Option<ErrorLocation>,
    
    /// Stack trace
    pub stack_trace: Option<Vec<StackFrame>>,
}

/// Error location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// Line number
    pub line: usize,
    
    /// Column number
    pub column: usize,
    
    /// File name
    pub file: Option<String>,
}

/// Stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name
    pub function: String,
    
    /// File name
    pub file: Option<String>,
    
    /// Line number
    pub line: Option<usize>,
    
    /// Column number
    pub column: Option<usize>,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    /// Session ID
    pub id: String,
    
    /// Session name
    pub name: String,
    
    /// Creation time
    pub created: DateTime<Utc>,
    
    /// Last accessed time
    pub last_accessed: DateTime<Utc>,
    
    /// Number of variables
    pub variable_count: usize,
    
    /// Number of history entries
    pub history_count: usize,
    
    /// Total execution time in milliseconds
    pub total_execution_time: u64,
    
    /// Number of successful executions
    pub successful_executions: usize,
    
    /// Number of failed executions
    pub failed_executions: usize,
    
    /// Memory usage in bytes (approximate)
    pub memory_usage: usize,
}

/// Service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatistics {
    /// Number of active sessions
    pub active_sessions: usize,
    
    /// Maximum number of sessions
    pub max_sessions: usize,
    
    /// Total number of variables across all sessions
    pub total_variables: usize,
    
    /// Total number of history entries across all sessions
    pub total_history_entries: usize,
    
    /// Total execution time in milliseconds
    pub total_execution_time: u64,
    
    /// Number of successful executions
    pub successful_executions: usize,
    
    /// Number of failed executions
    pub failed_executions: usize,
    
    /// Memory usage in bytes (approximate)
    pub memory_usage: usize,
    
    /// Uptime in seconds
    pub uptime: u64,
    
    /// Start time
    pub start_time: DateTime<Utc>,
}
