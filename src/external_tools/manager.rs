// src/external_tools/manager.rs - Tool manager for external tools

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::value::Value;
use super::common::{ExternalTool, ToolParams, ToolResult, ToolStatus, ToolError, ToolContext};

/// Configuration for the tool manager
#[derive(Debug, Clone)]
pub struct ToolManagerConfig {
    /// Maximum number of tools
    pub max_tools: usize,
    
    /// Default timeout in milliseconds
    pub default_timeout_ms: u64,
    
    /// Default memory limit in bytes
    pub default_max_memory: u64,
    
    /// Enable logging
    pub enable_logging: bool,
}

impl Default for ToolManagerConfig {
    fn default() -> Self {
        Self {
            max_tools: 100,
            default_timeout_ms: 30000, // 30 seconds
            default_max_memory: 100 * 1024 * 1024, // 100 MB
            enable_logging: true,
        }
    }
}

/// Manager for external tools
pub struct ToolManager {
    /// Registered tools
    tools: HashMap<String, Box<dyn ExternalTool>>,
    
    /// Global configuration
    config: ToolManagerConfig,
    
    /// Execution log
    log: Arc<Mutex<Vec<ToolExecution>>>,
}

/// Tool execution log entry
#[derive(Debug, Clone)]
struct ToolExecution {
    /// Tool name
    tool_name: String,
    
    /// Command
    command: String,
    
    /// Timestamp
    timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Duration in milliseconds
    duration_ms: u64,
    
    /// Status
    status: ToolStatus,
    
    /// Error message (if any)
    error: Option<String>,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            config: ToolManagerConfig::default(),
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a new tool manager with custom configuration
    pub fn with_config(config: ToolManagerConfig) -> Self {
        Self {
            tools: HashMap::new(),
            config,
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Register a tool
    pub fn register_tool<T: ExternalTool + 'static>(&mut self, tool: T) -> Result<(), ToolError> {
        let name = tool.name().to_string();
        
        // Check if tool with this name already exists
        if self.tools.contains_key(&name) {
            return Err(ToolError::new(409, format!("Tool with name '{}' already registered", name)));
        }
        
        // Check if maximum number of tools is reached
        if self.tools.len() >= self.config.max_tools {
            return Err(ToolError::new(507, format!("Maximum number of tools ({}) reached", self.config.max_tools)));
        }
        
        // Register tool
        self.tools.insert(name.clone(), Box::new(tool));
        
        Ok(())
    }
    
    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn ExternalTool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }
    
    /// Get a tool by name with mutable access
    pub fn get_tool_mut(&mut self, name: &str) -> Option<&mut dyn ExternalTool> {
        self.tools.get_mut(name).map(|tool| tool.as_mut())
    }
    
    /// Execute a tool
    pub fn execute_tool(&self, name: &str, params: &ToolParams) -> Result<ToolResult, ToolError> {
        // Get tool
        let tool = self.get_tool(name)
            .ok_or_else(|| ToolError::new(404, format!("Tool not found: {}", name)))?;
        
        // Check if tool is available
        if !tool.is_available() {
            return Err(ToolError::new(503, format!("Tool is not available: {}", name)));
        }
        
        // Start timer
        let start_time = std::time::Instant::now();
        let timestamp = chrono::Utc::now();
        
        // Execute tool
        let result = tool.execute(params);
        
        // Calculate duration
        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;
        
        // Log execution if enabled
        if self.config.enable_logging {
            let mut log = self.log.lock().unwrap();
            log.push(ToolExecution {
                tool_name: name.to_string(),
                command: params.command.clone(),
                timestamp,
                duration_ms,
                status: match &result {
                    Ok(r) => r.status.clone(),
                    Err(_) => ToolStatus::Failed,
                },
                error: match &result {
                    Ok(_) => None,
                    Err(e) => Some(e.message.clone()),
                },
            });
        }
        
        result
    }
    
    /// List available tools
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    /// Get tool descriptions
    pub fn get_tool_descriptions(&self) -> HashMap<String, String> {
        self.tools.iter()
            .map(|(name, tool)| (name.clone(), tool.description().to_string()))
            .collect()
    }
    
    /// Get execution log
    pub fn get_log(&self) -> Vec<HashMap<String, Value>> {
        let log = self.log.lock().unwrap();
        log.iter().map(|entry| {
            let mut map = HashMap::new();
            map.insert("tool_name".to_string(), Value::string(entry.tool_name.clone()));
            map.insert("command".to_string(), Value::string(entry.command.clone()));
            map.insert("timestamp".to_string(), Value::string(entry.timestamp.to_rfc3339()));
            map.insert("duration_ms".to_string(), Value::number(entry.duration_ms as f64));
            map.insert("status".to_string(), Value::string(match entry.status {
                ToolStatus::Success => "success",
                ToolStatus::Partial => "partial",
                ToolStatus::Failed => "failed",
            }.to_string()));
            if let Some(error) = &entry.error {
                map.insert("error".to_string(), Value::string(error.clone()));
            }
            map
        }).collect()
    }
    
    /// Clear execution log
    pub fn clear_log(&mut self) {
        let mut log = self.log.lock().unwrap();
        log.clear();
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &ToolManagerConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: ToolManagerConfig) {
        self.config = config;
    }
}
