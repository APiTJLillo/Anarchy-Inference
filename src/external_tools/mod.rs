// src/external_tools/mod.rs - External tool integration module

//! External tool integration for Anarchy Inference
//! 
//! This module provides interfaces for integrating external tools such as
//! web APIs, search engines, and file system operations into Anarchy Inference.
//! These interfaces enable AI agents to interact with external systems while
//! maintaining the token efficiency benefits of Anarchy Inference.

mod common;
mod web;
mod search;
mod filesystem;
mod manager;

pub use common::{ExternalTool, ToolParams, ToolResult, ToolStatus, ToolError, ToolContext};
pub use web::WebTool;
pub use search::SearchTool;
pub use filesystem::FileSystemTool;
pub use manager::ToolManager;

/// Initialize the external tools module
pub fn init() -> ToolManager {
    let mut manager = ToolManager::new();
    
    // Register default tools
    let web_tool = WebTool::new();
    let search_tool = SearchTool::new();
    let filesystem_tool = FileSystemTool::new(std::path::PathBuf::from("."));
    
    manager.register_tool(web_tool).expect("Failed to register web tool");
    manager.register_tool(search_tool).expect("Failed to register search tool");
    manager.register_tool(filesystem_tool).expect("Failed to register filesystem tool");
    
    manager
}
