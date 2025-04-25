// src/external_tools/common.rs - Common types and traits for external tools

use std::collections::HashMap;
use crate::value::Value;
use std::fmt;

/// Common trait for all external tools
pub trait ExternalTool: Send + Sync {
    /// Get the name of the tool
    fn name(&self) -> &str;
    
    /// Get the description of the tool
    fn description(&self) -> &str;
    
    /// Check if the tool is available
    fn is_available(&self) -> bool;
    
    /// Execute the tool with the given parameters
    fn execute(&self, params: &ToolParams) -> Result<ToolResult, ToolError>;
}

/// Parameters for tool execution
#[derive(Debug, Clone)]
pub struct ToolParams {
    /// Command to execute
    pub command: String,
    
    /// Arguments for the command
    pub args: HashMap<String, Value>,
    
    /// Context for the execution
    pub context: Option<ToolContext>,
}

impl ToolParams {
    /// Create a new set of tool parameters
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: HashMap::new(),
            context: None,
        }
    }
    
    /// Add an argument
    pub fn with_arg<K, V>(mut self, key: K, value: V) -> Self 
    where 
        K: Into<String>,
        V: Into<Value>,
    {
        self.args.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple arguments
    pub fn with_args<K, V, I>(mut self, args: I) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in args {
            self.args.insert(key.into(), value.into());
        }
        self
    }
    
    /// Set the context
    pub fn with_context(mut self, context: ToolContext) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Get an argument as a specific type
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: TryFrom<Value>,
    {
        self.args.get(key).and_then(|v| T::try_from(v.clone()).ok())
    }
    
    /// Get an argument as a string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.args.get(key).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => Some(v.to_string()),
        })
    }
}

/// Result of tool execution
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Status of the execution
    pub status: ToolStatus,
    
    /// Result data
    pub data: Value,
    
    /// Metadata about the execution
    pub metadata: HashMap<String, Value>,
}

impl ToolResult {
    /// Create a new successful result
    pub fn success(data: Value) -> Self {
        Self {
            status: ToolStatus::Success,
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new partial result
    pub fn partial(data: Value) -> Self {
        Self {
            status: ToolStatus::Partial,
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new failed result
    pub fn failed(data: Value) -> Self {
        Self {
            status: ToolStatus::Failed,
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Status of tool execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    /// Execution succeeded
    Success,
    
    /// Execution partially succeeded
    Partial,
    
    /// Execution failed
    Failed,
}

/// Error during tool execution
#[derive(Debug, Clone)]
pub struct ToolError {
    /// Error code
    pub code: u32,
    
    /// Error message
    pub message: String,
    
    /// Error details
    pub details: Option<Value>,
}

impl ToolError {
    /// Create a new tool error
    pub fn new<S: Into<String>>(code: u32, message: S) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }
    
    /// Add details to the error
    pub fn with_details<V: Into<Value>>(mut self, details: V) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tool error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for ToolError {}

/// Context for tool execution
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Session ID
    pub session_id: String,
    
    /// User ID
    pub user_id: Option<String>,
    
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new<S: Into<String>>(session_id: S) -> Self {
        Self {
            session_id: session_id.into(),
            user_id: None,
            timeout_ms: None,
            max_memory: None,
        }
    }
    
    /// Set the user ID
    pub fn with_user_id<S: Into<String>>(mut self, user_id: S) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    
    /// Set the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
    
    /// Set the maximum memory usage
    pub fn with_max_memory(mut self, max_memory: u64) -> Self {
        self.max_memory = Some(max_memory);
        self
    }
}
