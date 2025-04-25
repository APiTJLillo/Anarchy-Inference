// src/reasoning/tool_integration.rs - Tool integration for reasoning operations

use std::collections::HashMap;
use crate::error::LangError;
use crate::value::Value;
use crate::external_tools::common::Tool;

/// Manager for external tools used in reasoning operations
pub struct ToolManager {
    /// Registered tools
    tools: HashMap<String, Box<dyn Tool>>,
    /// Execution logs
    logs: Vec<ToolExecutionLog>,
}

/// Log entry for tool execution
struct ToolExecutionLog {
    /// Name of the tool
    tool_name: String,
    /// Arguments passed to the tool
    args: Value,
    /// Result of the execution
    result: Result<Value, LangError>,
    /// Timestamp of the execution
    timestamp: u64,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    /// Register a tool
    pub fn register_tool(&mut self, name: String, tool: Box<dyn Tool>) -> Result<(), LangError> {
        if self.tools.contains_key(&name) {
            return Err(LangError::runtime_error(&format!("Tool '{}' is already registered", name)));
        }
        
        self.tools.insert(name, tool);
        Ok(())
    }
    
    /// Call a tool with arguments
    pub fn call_tool(&mut self, name: &str, args: Value) -> Result<Value, LangError> {
        // Get the tool
        let tool = self.tools.get(name)
            .ok_or_else(|| LangError::runtime_error(&format!("Tool '{}' not found", name)))?;
        
        // Execute the tool
        let result = tool.execute(args.clone());
        
        // Log the execution
        self.log_execution(name, args, result.clone());
        
        result
    }
    
    /// Get a list of available tools
    pub fn get_available_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    /// Check if a tool is available
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
    
    /// Get the execution logs
    pub fn get_logs(&self) -> &[ToolExecutionLog] {
        &self.logs
    }
    
    /// Clear the execution logs
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
    
    /// Log a tool execution
    fn log_execution(&mut self, tool_name: &str, args: Value, result: Result<Value, LangError>) {
        let log_entry = ToolExecutionLog {
            tool_name: tool_name.to_string(),
            args,
            result,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.logs.push(log_entry);
        
        // Limit log size to prevent memory issues
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

/// Extension trait for Value to simplify tool calling
pub trait ToolCallingExt {
    /// Call a tool with this value as the arguments
    fn call_tool(&self, tool_manager: &mut ToolManager, tool_name: &str) -> Result<Value, LangError>;
}

impl ToolCallingExt for Value {
    fn call_tool(&self, tool_manager: &mut ToolManager, tool_name: &str) -> Result<Value, LangError> {
        tool_manager.call_tool(tool_name, self.clone())
    }
}
