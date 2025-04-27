// Advanced REPL Service module for Anarchy Inference
//
// This module provides a modern HTTP/WebSocket API for interactive code execution
// with support for multiple named sessions, state persistence, and real-time feedback.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod http_api;
mod websocket_api;
mod session;
mod persistence;
mod execution;
mod types;

pub use http_api::HttpApi;
pub use websocket_api::WebSocketApi;
pub use session::{Session, SessionManager, SessionConfig};
pub use persistence::{PersistenceManager, PersistenceConfig};
pub use execution::{ExecutionEngine, ExecutionResult, ExecutionConfig};
pub use types::*;

/// Advanced REPL Service configuration
#[derive(Debug, Clone)]
pub struct ReplServiceConfig {
    /// HTTP server host
    pub http_host: String,
    
    /// HTTP server port
    pub http_port: u16,
    
    /// WebSocket server host
    pub ws_host: String,
    
    /// WebSocket server port
    pub ws_port: u16,
    
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    
    /// Default session timeout in seconds
    pub default_session_timeout: u64,
    
    /// Whether to enable session persistence
    pub enable_persistence: bool,
    
    /// Persistence storage directory
    pub persistence_dir: String,
    
    /// Maximum execution time in milliseconds
    pub max_execution_time: u64,
    
    /// Maximum memory usage in megabytes
    pub max_memory_usage: u64,
    
    /// Whether to enable authentication
    pub enable_auth: bool,
    
    /// API key for authentication (if enabled)
    pub api_key: Option<String>,
}

impl Default for ReplServiceConfig {
    fn default() -> Self {
        ReplServiceConfig {
            http_host: "127.0.0.1".to_string(),
            http_port: 8081,
            ws_host: "127.0.0.1".to_string(),
            ws_port: 8082,
            max_sessions: 100,
            default_session_timeout: 3600, // 1 hour
            enable_persistence: true,
            persistence_dir: "./sessions".to_string(),
            max_execution_time: 5000, // 5 seconds
            max_memory_usage: 100, // 100 MB
            enable_auth: false,
            api_key: None,
        }
    }
}

/// Advanced REPL Service
pub struct ReplService {
    /// Service configuration
    config: ReplServiceConfig,
    
    /// HTTP API
    http_api: Arc<Mutex<HttpApi>>,
    
    /// WebSocket API
    websocket_api: Arc<Mutex<WebSocketApi>>,
    
    /// Session manager
    session_manager: Arc<Mutex<SessionManager>>,
    
    /// Persistence manager
    persistence_manager: Arc<Mutex<PersistenceManager>>,
    
    /// Execution engine
    execution_engine: Arc<Mutex<ExecutionEngine>>,
    
    /// Running flag
    running: Arc<Mutex<bool>>,
}

impl ReplService {
    /// Create a new Advanced REPL Service
    pub fn new(config: Option<ReplServiceConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        // Create the session manager
        let session_manager = Arc::new(Mutex::new(SessionManager::new(config.max_sessions)));
        
        // Create the persistence manager
        let persistence_config = PersistenceConfig {
            enable_persistence: config.enable_persistence,
            persistence_dir: config.persistence_dir.clone(),
        };
        let persistence_manager = Arc::new(Mutex::new(PersistenceManager::new(persistence_config)));
        
        // Create the execution engine
        let execution_config = ExecutionConfig {
            max_execution_time: config.max_execution_time,
            max_memory_usage: config.max_memory_usage,
        };
        let execution_engine = Arc::new(Mutex::new(ExecutionEngine::new(execution_config)));
        
        // Create the HTTP API
        let http_config = http_api::HttpApiConfig {
            host: config.http_host.clone(),
            port: config.http_port,
            enable_auth: config.enable_auth,
            api_key: config.api_key.clone(),
        };
        let http_api = Arc::new(Mutex::new(HttpApi::new(
            http_config,
            session_manager.clone(),
            persistence_manager.clone(),
            execution_engine.clone()
        )));
        
        // Create the WebSocket API
        let ws_config = websocket_api::WebSocketApiConfig {
            host: config.ws_host.clone(),
            port: config.ws_port,
            enable_auth: config.enable_auth,
            api_key: config.api_key.clone(),
        };
        let websocket_api = Arc::new(Mutex::new(WebSocketApi::new(
            ws_config,
            session_manager.clone(),
            persistence_manager.clone(),
            execution_engine.clone()
        )));
        
        ReplService {
            config,
            http_api,
            websocket_api,
            session_manager,
            persistence_manager,
            execution_engine,
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the service
    pub fn start(&self) -> Result<(), String> {
        // Set the running flag
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err("Service is already running".to_string());
        }
        *running = true;
        drop(running);
        
        // Start the HTTP API
        let http_api = self.http_api.clone();
        std::thread::spawn(move || {
            let mut http_api = http_api.lock().unwrap();
            if let Err(e) = http_api.start() {
                eprintln!("Error starting HTTP API: {}", e);
            }
        });
        
        // Start the WebSocket API
        let websocket_api = self.websocket_api.clone();
        std::thread::spawn(move || {
            let mut websocket_api = websocket_api.lock().unwrap();
            if let Err(e) = websocket_api.start() {
                eprintln!("Error starting WebSocket API: {}", e);
            }
        });
        
        println!("Advanced REPL Service started");
        println!("HTTP API listening on {}:{}", self.config.http_host, self.config.http_port);
        println!("WebSocket API listening on {}:{}", self.config.ws_host, self.config.ws_port);
        
        Ok(())
    }
    
    /// Stop the service
    pub fn stop(&self) -> Result<(), String> {
        // Check the running flag
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err("Service is not running".to_string());
        }
        *running = false;
        drop(running);
        
        // Stop the HTTP API
        let mut http_api = self.http_api.lock().unwrap();
        http_api.stop()?;
        
        // Stop the WebSocket API
        let mut websocket_api = self.websocket_api.lock().unwrap();
        websocket_api.stop()?;
        
        println!("Advanced REPL Service stopped");
        
        Ok(())
    }
    
    /// Get the service configuration
    pub fn get_config(&self) -> ReplServiceConfig {
        self.config.clone()
    }
    
    /// Set the service configuration
    pub fn set_config(&mut self, config: ReplServiceConfig) -> Result<(), String> {
        // Check if the service is running
        let running = self.running.lock().unwrap();
        if *running {
            return Err("Cannot change configuration while service is running".to_string());
        }
        
        self.config = config;
        
        Ok(())
    }
    
    /// Get the session manager
    pub fn get_session_manager(&self) -> Arc<Mutex<SessionManager>> {
        self.session_manager.clone()
    }
    
    /// Get the persistence manager
    pub fn get_persistence_manager(&self) -> Arc<Mutex<PersistenceManager>> {
        self.persistence_manager.clone()
    }
    
    /// Get the execution engine
    pub fn get_execution_engine(&self) -> Arc<Mutex<ExecutionEngine>> {
        self.execution_engine.clone()
    }
}

/// Create a new Advanced REPL Service
pub fn create_repl_service(config: Option<ReplServiceConfig>) -> ReplService {
    ReplService::new(config)
}
