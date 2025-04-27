// HTTP API module for Advanced REPL Service
//
// This module provides HTTP endpoints for session management and synchronous code execution.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crate::language_hub_server::repl::session::{Session, SessionManager, SessionConfig};
use crate::language_hub_server::repl::persistence::PersistenceManager;
use crate::language_hub_server::repl::execution::{ExecutionEngine, ExecutionResult, ExecutionConfig};
use crate::language_hub_server::repl::types::*;

/// HTTP API configuration
#[derive(Debug, Clone)]
pub struct HttpApiConfig {
    /// Server host
    pub host: String,
    
    /// Server port
    pub port: u16,
    
    /// Whether to enable authentication
    pub enable_auth: bool,
    
    /// API key for authentication (if enabled)
    pub api_key: Option<String>,
}

impl Default for HttpApiConfig {
    fn default() -> Self {
        HttpApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8081,
            enable_auth: false,
            api_key: None,
        }
    }
}

/// HTTP API for Advanced REPL Service
pub struct HttpApi {
    /// API configuration
    config: HttpApiConfig,
    
    /// Session manager
    session_manager: Arc<Mutex<SessionManager>>,
    
    /// Persistence manager
    persistence_manager: Arc<Mutex<PersistenceManager>>,
    
    /// Execution engine
    execution_engine: Arc<Mutex<ExecutionEngine>>,
    
    /// Running flag
    running: bool,
    
    /// Server thread handle
    server_thread: Option<thread::JoinHandle<()>>,
}

impl HttpApi {
    /// Create a new HTTP API
    pub fn new(
        config: HttpApiConfig,
        session_manager: Arc<Mutex<SessionManager>>,
        persistence_manager: Arc<Mutex<PersistenceManager>>,
        execution_engine: Arc<Mutex<ExecutionEngine>>
    ) -> Self {
        HttpApi {
            config,
            session_manager,
            persistence_manager,
            execution_engine,
            running: false,
            server_thread: None,
        }
    }
    
    /// Start the HTTP API server
    pub fn start(&mut self) -> Result<(), String> {
        if self.running {
            return Err("HTTP API is already running".to_string());
        }
        
        // Set the running flag
        self.running = true;
        
        // Create the TCP listener
        let address = format!("{}:{}", self.config.host, self.config.port);
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(e) => {
                self.running = false;
                return Err(format!("Failed to bind to {}: {}", address, e));
            }
        };
        
        // Clone the required resources for the server thread
        let session_manager = self.session_manager.clone();
        let persistence_manager = self.persistence_manager.clone();
        let execution_engine = self.execution_engine.clone();
        let config = self.config.clone();
        
        // Create the server thread
        let server_thread = thread::spawn(move || {
            println!("HTTP API server started on {}", address);
            
            // Set the listener to non-blocking mode
            listener.set_nonblocking(true).expect("Failed to set non-blocking mode");
            
            // Accept connections
            while Arc::new(Mutex::new(true)).lock().unwrap().clone() {
                // Check for new connections
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("New connection from {}", addr);
                        
                        // Clone the resources for the connection thread
                        let session_manager = session_manager.clone();
                        let persistence_manager = persistence_manager.clone();
                        let execution_engine = execution_engine.clone();
                        let config = config.clone();
                        
                        // Handle the connection in a new thread
                        thread::spawn(move || {
                            if let Err(e) = handle_connection(
                                stream,
                                &session_manager,
                                &persistence_manager,
                                &execution_engine,
                                &config
                            ) {
                                eprintln!("Error handling connection: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No new connections, sleep for a bit
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        break;
                    }
                }
            }
            
            println!("HTTP API server stopped");
        });
        
        // Store the server thread handle
        self.server_thread = Some(server_thread);
        
        Ok(())
    }
    
    /// Stop the HTTP API server
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.running {
            return Err("HTTP API is not running".to_string());
        }
        
        // Set the running flag
        self.running = false;
        
        // Wait for the server thread to finish
        if let Some(thread) = self.server_thread.take() {
            // The thread should exit on its own when the running flag is set to false
            // We'll give it a reasonable timeout
            let timeout = Duration::from_secs(5);
            let start = Instant::now();
            
            while start.elapsed() < timeout {
                if thread.is_finished() {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(100));
            }
            
            return Err("Failed to stop HTTP API server within timeout".to_string());
        }
        
        Ok(())
    }
}

/// Handle an HTTP connection
fn handle_connection(
    mut stream: TcpStream,
    session_manager: &Arc<Mutex<SessionManager>>,
    persistence_manager: &Arc<Mutex<PersistenceManager>>,
    execution_engine: &Arc<Mutex<ExecutionEngine>>,
    config: &HttpApiConfig
) -> Result<(), String> {
    // Read the request
    let mut buffer = [0; 1024];
    let mut request = String::new();
    
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                request.push_str(&String::from_utf8_lossy(&buffer[0..n]));
                if request.contains("\r\n\r\n") {
                    break;
                }
            }
            Err(e) => return Err(format!("Error reading from stream: {}", e)),
        }
    }
    
    // Parse the request
    let request_lines: Vec<&str> = request.lines().collect();
    if request_lines.is_empty() {
        return Err("Empty request".to_string());
    }
    
    // Parse the request line
    let request_line_parts: Vec<&str> = request_lines[0].split_whitespace().collect();
    if request_line_parts.len() != 3 {
        return send_response(&mut stream, 400, "Bad Request", "Invalid request line");
    }
    
    let method = request_line_parts[0];
    let path = request_line_parts[1];
    
    // Check authentication if enabled
    if config.enable_auth {
        let mut authorized = false;
        
        // Look for the Authorization header
        for line in &request_lines {
            if line.starts_with("Authorization: ") {
                let auth_header = &line[14..];
                if auth_header.starts_with("Bearer ") {
                    let token = &auth_header[7..];
                    if let Some(api_key) = &config.api_key {
                        if token == api_key {
                            authorized = true;
                            break;
                        }
                    }
                }
            }
        }
        
        if !authorized {
            return send_response(&mut stream, 401, "Unauthorized", "Invalid or missing API key");
        }
    }
    
    // Parse the request body
    let mut body = String::new();
    if let Some(pos) = request.find("\r\n\r\n") {
        body = request[pos + 4..].to_string();
    }
    
    // Handle the request based on the path and method
    match (method, path) {
        // Session management
        ("POST", "/api/sessions") => handle_create_session(&mut stream, &body, session_manager, persistence_manager),
        ("GET", "/api/sessions") => handle_list_sessions(&mut stream, session_manager),
        ("GET", p) if p.starts_with("/api/sessions/") => {
            let session_id = &p[14..];
            handle_get_session(&mut stream, session_id, session_manager)
        }
        ("DELETE", p) if p.starts_with("/api/sessions/") => {
            let session_id = &p[14..];
            handle_delete_session(&mut stream, session_id, session_manager, persistence_manager)
        }
        ("PUT", p) if p.starts_with("/api/sessions/") && p.ends_with("/config") => {
            let session_id = &p[14..p.len() - 7]; // Remove "/config"
            handle_update_session_config(&mut stream, session_id, &body, session_manager)
        }
        
        // Code execution
        ("POST", p) if p.starts_with("/api/sessions/") && p.ends_with("/execute") => {
            let session_id = &p[14..p.len() - 9]; // Remove "/execute"
            handle_execute_code(&mut stream, session_id, &body, session_manager, execution_engine)
        }
        ("GET", p) if p.starts_with("/api/sessions/") && p.ends_with("/variables") => {
            let session_id = &p[14..p.len() - 11]; // Remove "/variables"
            handle_get_variables(&mut stream, session_id, session_manager)
        }
        ("GET", p) if p.starts_with("/api/sessions/") && p.ends_with("/history") => {
            let session_id = &p[14..p.len() - 9]; // Remove "/history"
            handle_get_history(&mut stream, session_id, session_manager)
        }
        
        // Unknown path or method
        _ => send_response(&mut stream, 404, "Not Found", "The requested resource was not found"),
    }
}

/// Handle create session request
fn handle_create_session(
    stream: &mut TcpStream,
    body: &str,
    session_manager: &Arc<Mutex<SessionManager>>,
    persistence_manager: &Arc<Mutex<PersistenceManager>>
) -> Result<(), String> {
    // Parse the request body as JSON
    let request: serde_json::Value = match serde_json::from_str(body) {
        Ok(json) => json,
        Err(e) => return send_response(stream, 400, "Bad Request", &format!("Invalid JSON: {}", e)),
    };
    
    // Extract the session name (optional)
    let name = if let Some(name) = request.get("name").and_then(|n| n.as_str()) {
        name.to_string()
    } else {
        format!("session_{}", uuid::Uuid::new_v4())
    };
    
    // Extract the timeout (optional)
    let timeout = request.get("timeout")
        .and_then(|t| t.as_u64())
        .unwrap_or(3600); // Default: 1 hour
    
    // Extract the persistence flag (optional)
    let persistence = request.get("persistence")
        .and_then(|p| p.as_bool())
        .unwrap_or(true);
    
    // Create the session configuration
    let config = SessionConfig {
        name: name.clone(),
        timeout: Duration::from_secs(timeout),
        persistence,
    };
    
    // Create the session
    let mut session_manager = session_manager.lock().unwrap();
    let session_id = match session_manager.create_session(config) {
        Ok(id) => id,
        Err(e) => return send_response(stream, 500, "Internal Server Error", &format!("Failed to create session: {}", e)),
    };
    
    // If persistence is enabled, initialize the session state
    if persistence {
        let mut persistence_manager = persistence_manager.lock().unwrap();
        if let Err(e) = persistence_manager.initialize_session(&session_id) {
            eprintln!("Warning: Failed to initialize session persistence: {}", e);
        }
    }
    
    // Create the response
    let response = serde_json::json!({
        "id": session_id,
        "name": name,
        "timeout": timeout,
        "persistence": persistence,
        "created": chrono::Utc::now().to_rfc3339(),
    });
    
    // Send the response
    send_json_response(stream, 201, "Created", &response)
}

/// Handle list sessions request
fn handle_list_sessions(
    stream: &mut TcpStream,
    session_manager: &Arc<Mutex<SessionManager>>
) -> Result<(), String> {
    // Get the session manager
    let session_manager = session_manager.lock().unwrap();
    
    // Get the sessions
    let sessions = session_manager.list_sessions();
    
    // Create the response
    let response = serde_json::json!({
        "sessions": sessions,
        "count": sessions.len(),
    });
    
    // Send the response
    send_json_response(stream, 200, "OK", &response)
}

/// Handle get session request
fn handle_get_session(
    stream: &mut TcpStream,
    session_id: &str,
    session_manager: &Arc<Mutex<SessionManager>>
) -> Result<(), String> {
    // Get the session manager
    let session_manager = session_manager.lock().unwrap();
    
    // Get the session
    let session = match session_manager.get_session(session_id) {
        Some(session) => session,
        None => return send_response(stream, 404, "Not Found", &format!("Session not found: {}", session_id)),
    };
    
    // Create the response
    let response = serde_json::json!({
        "id": session_id,
        "name": session.config.name,
        "timeout": session.config.timeout.as_secs(),
        "persistence": session.config.persistence,
        "created": session.created.to_rfc3339(),
        "lastAccessed": session.last_accessed.to_rfc3339(),
    });
    
    // Send the response
    send_json_response(stream, 200, "OK", &response)
}

/// Handle delete session request
fn handle_delete_session(
    stream: &mut TcpStream,
    session_id: &str,
    session_manager: &Arc<Mutex<SessionManager>>,
    persistence_manager: &Arc<Mutex<PersistenceManager>>
) -> Result<(), String> {
    // Get the session manager
    let mut session_manager = session_manager.lock().unwrap();
    
    // Check if the session exists
    if !session_manager.session_exists(session_id) {
        return send_response(stream, 404, "Not Found", &format!("Session not found: {}", session_id));
    }
    
    // Delete the session
    if let Err(e) = session_manager.delete_session(session_id) {
        return send_response(stream, 500, "Internal Server Error", &format!("Failed to delete session: {}", e));
    }
    
    // Delete the session state if persistence is enabled
    let mut persistence_manager = persistence_manager.lock().unwrap();
    if let Err(e) = persistence_manager.delete_session(session_id) {
        eprintln!("Warning: Failed to delete session persistence: {}", e);
    }
    
    // Send the response
    send_response(stream, 204, "No Content", "")
}

/// Handle update session config request
fn handle_update_session_config(
    stream: &mut TcpStream,
    session_id: &str,
    body: &str,
    session_manager: &Arc<Mutex<SessionManager>>
) -> Result<(), String> {
    // Parse the request body as JSON
    let request: serde_json::Value = match serde_json::from_str(body) {
        Ok(json) => json,
        Err(e) => return send_response(stream, 400, "Bad Request", &format!("Invalid JSON: {}", e)),
    };
    
    // Get the session manager
    let mut session_manager = session_manager.lock().unwrap();
    
    // Check if the session exists
    if !session_manager.session_exists(session_id) {
        return send_response(stream, 404, "Not Found", &format!("Session not found: {}", session_id));
    }
    
    // Get the current session config
    let session = session_manager.get_session(session_id).unwrap();
    let mut config = session.config.clone();
    
    // Update the config with the provided values
    if let Some(name) = request.get("name").and_then(|n| n.as_str()) {
        config.name = name.to_string();
    }
    
    if let Some(timeout) = request.get("timeout").and_then(|t| t.as_u64()) {
        config.timeout = Duration::from_secs(timeout);
    }
    
    if let Some(persistence) = request.get("persistence").and_then(|p| p.as_bool()) {
        config.persistence = persistence;
    }
    
    // Update the session config
    if let Err(e) = session_manager.update_session_config(session_id, config.clone()) {
        return send_response(stream, 500, "Internal Server Error", &format!("Failed to update session config: {}", e));
    }
    
    // Create the response
    let response = serde_json::json!({
        "id": session_id,
        "name": config.name,
        "timeout": config.timeout.as_secs(),
        "persistence": config.persistence,
    });
    
    // Send the response
    send_json_response(stream, 200, "OK", &response)
}

/// Handle execute code request
fn handle_execute_code(
    stream: &mut TcpStream,
    session_id: &str,
    body: &str,
    session_manager: &Arc<Mutex<SessionManager>>,
    execution_engine: &Arc<Mutex<ExecutionEngine>>
) -> Result<(), String> {
    // Parse the request body as JSON
    let request: serde_json::Value = match serde_json::from_str(body) {
        Ok(json) => json,
        Err(e) => return send_response(stream, 400, "Bad Request", &format!("Invalid JSON: {}", e)),
    };
    
    // Extract the code
    let code = match request.get("code").and_then(|c| c.as_str()) {
        Some(code) => code,
        None => return send_response(stream, 400, "Bad Request", "Missing 'code' parameter"),
    };
    
    // Extract the timeout (optional)
    let timeout = request.get("timeout")
        .and_then(|t| t.as_u64())
        .unwrap_or(5000); // Default: 5 seconds
    
    // Extract the capture output flag (optional)
    let capture_output = request.get("captureOutput")
        .and_then(|c| c.as_bool())
        .unwrap_or(true);
    
    // Get the session manager
    let mut session_manager = session_manager.lock().unwrap();
    
    // Check if the session exists
    if !session_manager.session_exists(session_id) {
        return send_response(stream, 404, "Not Found", &format!("Session not found: {}", session_id));
    }
    
    // Get the session
    let session = session_manager.get_session_mut(session_id).unwrap();
    
    // Update the last accessed time
    session.last_accessed = chrono::Utc::now();
    
    // Get the execution engine
    let mut execution_engine = execution_engine.lock().unwrap();
    
    // Execute the code
    let result = match execution_engine.execute(session, code, timeout, capture_output) {
        Ok(result) => result,
        Err(e) => return send_response(stream, 500, "Internal Server Error", &format!("Failed to execute code: {}", e)),
    };
    
    // Create the response
    let response = serde_json::json!({
        "result": result.result,
        "output": result.output,
        "duration": result.duration,
        "status": result.status,
    });
    
    // Send the response
    send_json_response(stream, 200, "OK", &response)
}

/// Handle get variables request
fn handle_get_variables(
    stream: &mut TcpStream,
    session_id: &str,
    session_manager: &Arc<Mutex<SessionManager>>
) -> Result<(), String> {
    // Get the session manager
    let session_manager = session_manager.lock().unwrap();
    
    // Check if the session exists
    if !session_manager.session_exists(session_id) {
        return send_response(stream, 404, "Not Found", &format!("Session not found: {}", session_id));
    }
    
    // Get the session
    let session = session_manager.get_session(session_id).unwrap();
    
    // Create the response
    let response = serde_json::json!({
        "variables": session.variables,
        "count": session.variables.len(),
    });
    
    // Send the response
    send_json_response(stream, 200, "OK", &response)
}

/// Handle get history request
fn handle_get_history(
    stream: &mut TcpStream,
    session_id: &str,
    session_manager: &Arc<Mutex<SessionManager>>
) -> Result<(), String> {
    // Get the session manager
    let session_manager = session_manager.lock().unwrap();
    
    // Check if the session exists
    if !session_manager.session_exists(session_id) {
        return send_response(stream, 404, "Not Found", &format!("Session not found: {}", session_id));
    }
    
    // Get the session
    let session = session_manager.get_session(session_id).unwrap();
    
    // Create the response
    let response = serde_json::json!({
        "history": session.history,
        "count": session.history.len(),
    });
    
    // Send the response
    send_json_response(stream, 200, "OK", &response)
}

/// Send an HTTP response
fn send_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    body: &str
) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status_text,
        body.len(),
        body
    );
    
    match stream.write_all(response.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error writing to stream: {}", e)),
    }
}

/// Send a JSON HTTP response
fn send_json_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    json: &serde_json::Value
) -> Result<(), String> {
    let body = match serde_json::to_string(json) {
        Ok(body) => body,
        Err(e) => return Err(format!("Error serializing JSON: {}", e)),
    };
    
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status_text,
        body.len(),
        body
    );
    
    match stream.write_all(response.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error writing to stream: {}", e)),
    }
}
