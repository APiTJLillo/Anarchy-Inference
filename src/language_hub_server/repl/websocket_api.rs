// WebSocket API module for Advanced REPL Service
//
// This module provides a WebSocket interface for real-time, asynchronous code execution.

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

use tungstenite::{accept, Message, WebSocket};
use tungstenite::protocol::Role;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// WebSocket API configuration
#[derive(Debug, Clone)]
pub struct WebSocketApiConfig {
    /// Server host
    pub host: String,
    
    /// Server port
    pub port: u16,
    
    /// Whether to enable authentication
    pub enable_auth: bool,
    
    /// API key for authentication (if enabled)
    pub api_key: Option<String>,
}

impl Default for WebSocketApiConfig {
    fn default() -> Self {
        WebSocketApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8082,
            enable_auth: false,
            api_key: None,
        }
    }
}

/// WebSocket API for Advanced REPL Service
pub struct WebSocketApi {
    /// API configuration
    config: WebSocketApiConfig,
    
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
    
    /// Active connections
    connections: Arc<Mutex<HashMap<String, Arc<Mutex<WebSocketConnection>>>>>,
}

/// WebSocket connection
struct WebSocketConnection {
    /// Connection ID
    id: String,
    
    /// Session ID
    session_id: String,
    
    /// WebSocket
    websocket: WebSocket<TcpStream>,
    
    /// Active flag
    active: bool,
    
    /// Last activity time
    last_activity: Instant,
}

/// Client to server message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    /// Execute code
    #[serde(rename = "execute")]
    Execute {
        /// Code to execute
        code: String,
        
        /// Request ID
        id: String,
        
        /// Execution options
        options: Option<ExecuteOptions>,
    },
    
    /// Cancel execution
    #[serde(rename = "cancel")]
    Cancel {
        /// Execution ID
        execution_id: String,
    },
    
    /// Inspect variable
    #[serde(rename = "inspect")]
    Inspect {
        /// Variable name
        variable: String,
        
        /// Inspection depth
        depth: Option<usize>,
    },
    
    /// Ping
    #[serde(rename = "ping")]
    Ping {
        /// Timestamp
        timestamp: u64,
    },
    
    /// Authentication
    #[serde(rename = "auth")]
    Auth {
        /// API key
        api_key: String,
    },
}

/// Execution options
#[derive(Debug, Serialize, Deserialize)]
struct ExecuteOptions {
    /// Execution timeout in milliseconds
    timeout: Option<u64>,
    
    /// Whether to capture output
    capture_output: Option<bool>,
    
    /// Whether to execute asynchronously
    async_execution: Option<bool>,
}

/// Server to client message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    /// Execution start
    #[serde(rename = "executionStart")]
    ExecutionStart {
        /// Execution ID
        id: String,
        
        /// Request ID
        request_id: String,
        
        /// Timestamp
        timestamp: u64,
    },
    
    /// Output
    #[serde(rename = "output")]
    Output {
        /// Execution ID
        execution_id: String,
        
        /// Output content
        content: String,
        
        /// Output channel (stdout or stderr)
        channel: String,
    },
    
    /// Execution result
    #[serde(rename = "executionResult")]
    ExecutionResult {
        /// Execution ID
        execution_id: String,
        
        /// Result
        result: serde_json::Value,
        
        /// Execution duration in milliseconds
        duration: u64,
        
        /// Execution status
        status: String,
    },
    
    /// Error
    #[serde(rename = "error")]
    Error {
        /// Execution ID
        execution_id: Option<String>,
        
        /// Error details
        error: ErrorDetails,
    },
    
    /// Pong
    #[serde(rename = "pong")]
    Pong {
        /// Timestamp
        timestamp: u64,
    },
    
    /// Authentication result
    #[serde(rename = "authResult")]
    AuthResult {
        /// Success flag
        success: bool,
        
        /// Error message
        error: Option<String>,
    },
}

/// Error details
#[derive(Debug, Serialize, Deserialize)]
struct ErrorDetails {
    /// Error type
    #[serde(rename = "type")]
    error_type: String,
    
    /// Error message
    message: String,
    
    /// Error location
    location: Option<ErrorLocation>,
}

/// Error location
#[derive(Debug, Serialize, Deserialize)]
struct ErrorLocation {
    /// Line number
    line: usize,
    
    /// Column number
    column: usize,
}

impl WebSocketApi {
    /// Create a new WebSocket API
    pub fn new(
        config: WebSocketApiConfig,
        session_manager: Arc<Mutex<SessionManager>>,
        persistence_manager: Arc<Mutex<PersistenceManager>>,
        execution_engine: Arc<Mutex<ExecutionEngine>>
    ) -> Self {
        WebSocketApi {
            config,
            session_manager,
            persistence_manager,
            execution_engine,
            running: false,
            server_thread: None,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start the WebSocket API server
    pub fn start(&mut self) -> Result<(), String> {
        if self.running {
            return Err("WebSocket API is already running".to_string());
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
        let connections = self.connections.clone();
        let running = Arc::new(Mutex::new(self.running));
        
        // Create the server thread
        let server_thread = thread::spawn(move || {
            println!("WebSocket API server started on {}", address);
            
            // Set the listener to non-blocking mode
            listener.set_nonblocking(true).expect("Failed to set non-blocking mode");
            
            // Accept connections
            while running.lock().unwrap().clone() {
                // Check for new connections
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("New WebSocket connection from {}", addr);
                        
                        // Clone the resources for the connection thread
                        let session_manager = session_manager.clone();
                        let persistence_manager = persistence_manager.clone();
                        let execution_engine = execution_engine.clone();
                        let config = config.clone();
                        let connections = connections.clone();
                        
                        // Handle the connection in a new thread
                        thread::spawn(move || {
                            if let Err(e) = handle_websocket_connection(
                                stream,
                                &session_manager,
                                &persistence_manager,
                                &execution_engine,
                                &config,
                                &connections
                            ) {
                                eprintln!("Error handling WebSocket connection: {}", e);
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
                
                // Clean up inactive connections
                clean_up_connections(&connections);
            }
            
            println!("WebSocket API server stopped");
        });
        
        // Store the server thread handle
        self.server_thread = Some(server_thread);
        
        Ok(())
    }
    
    /// Stop the WebSocket API server
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.running {
            return Err("WebSocket API is not running".to_string());
        }
        
        // Set the running flag
        self.running = false;
        
        // Close all connections
        let mut connections = self.connections.lock().unwrap();
        for (_, connection) in connections.iter_mut() {
            let mut connection = connection.lock().unwrap();
            connection.active = false;
            
            // Try to send a close message
            let _ = connection.websocket.close(None);
        }
        
        // Clear the connections
        connections.clear();
        
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
            
            return Err("Failed to stop WebSocket API server within timeout".to_string());
        }
        
        Ok(())
    }
}

/// Handle a WebSocket connection
fn handle_websocket_connection(
    stream: TcpStream,
    session_manager: &Arc<Mutex<SessionManager>>,
    persistence_manager: &Arc<Mutex<PersistenceManager>>,
    execution_engine: &Arc<Mutex<ExecutionEngine>>,
    config: &WebSocketApiConfig,
    connections: &Arc<Mutex<HashMap<String, Arc<Mutex<WebSocketConnection>>>>>
) -> Result<(), String> {
    // Accept the WebSocket connection
    let websocket = match accept(stream) {
        Ok(websocket) => websocket,
        Err(e) => return Err(format!("Failed to accept WebSocket connection: {}", e)),
    };
    
    // Extract the session ID from the URL
    let session_id = extract_session_id_from_url(&websocket)?;
    
    // Check if the session exists
    {
        let session_manager = session_manager.lock().unwrap();
        if !session_manager.session_exists(&session_id) {
            // Send an error message
            let error_message = ServerMessage::Error {
                execution_id: None,
                error: ErrorDetails {
                    error_type: "SessionError".to_string(),
                    message: format!("Session not found: {}", session_id),
                    location: None,
                },
            };
            
            let message = serde_json::to_string(&error_message)
                .map_err(|e| format!("Failed to serialize error message: {}", e))?;
            
            let mut websocket = websocket;
            websocket.write_message(Message::Text(message))
                .map_err(|e| format!("Failed to send error message: {}", e))?;
            
            return Err(format!("Session not found: {}", session_id));
        }
    }
    
    // Create a connection ID
    let connection_id = Uuid::new_v4().to_string();
    
    // Create the connection
    let connection = Arc::new(Mutex::new(WebSocketConnection {
        id: connection_id.clone(),
        session_id: session_id.clone(),
        websocket,
        active: true,
        last_activity: Instant::now(),
    }));
    
    // Add the connection to the connections map
    {
        let mut connections = connections.lock().unwrap();
        connections.insert(connection_id.clone(), connection.clone());
    }
    
    // Authentication state
    let mut authenticated = !config.enable_auth;
    
    // Handle messages
    loop {
        // Check if the connection is still active
        {
            let connection = connection.lock().unwrap();
            if !connection.active {
                break;
            }
        }
        
        // Read a message
        let message = {
            let mut connection = connection.lock().unwrap();
            match connection.websocket.read_message() {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("Error reading WebSocket message: {}", e);
                    connection.active = false;
                    break;
                }
            }
        };
        
        // Update the last activity time
        {
            let mut connection = connection.lock().unwrap();
            connection.last_activity = Instant::now();
        }
        
        // Handle the message
        match message {
            Message::Text(text) => {
                // Parse the message
                let client_message: ClientMessage = match serde_json::from_str(&text) {
                    Ok(message) => message,
                    Err(e) => {
                        // Send an error message
                        let error_message = ServerMessage::Error {
                            execution_id: None,
                            error: ErrorDetails {
                                error_type: "ParseError".to_string(),
                                message: format!("Failed to parse message: {}", e),
                                location: None,
                            },
                        };
                        
                        let message = serde_json::to_string(&error_message)
                            .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                        
                        let mut connection = connection.lock().unwrap();
                        connection.websocket.write_message(Message::Text(message))
                            .map_err(|e| format!("Failed to send error message: {}", e))?;
                        
                        continue;
                    }
                };
                
                // Handle the client message
                match client_message {
                    ClientMessage::Auth { api_key } => {
                        // Check authentication
                        if config.enable_auth {
                            if let Some(expected_api_key) = &config.api_key {
                                if api_key == *expected_api_key {
                                    authenticated = true;
                                    
                                    // Send authentication success
                                    let auth_result = ServerMessage::AuthResult {
                                        success: true,
                                        error: None,
                                    };
                                    
                                    let message = serde_json::to_string(&auth_result)
                                        .map_err(|e| format!("Failed to serialize auth result: {}", e))?;
                                    
                                    let mut connection = connection.lock().unwrap();
                                    connection.websocket.write_message(Message::Text(message))
                                        .map_err(|e| format!("Failed to send auth result: {}", e))?;
                                } else {
                                    // Send authentication failure
                                    let auth_result = ServerMessage::AuthResult {
                                        success: false,
                                        error: Some("Invalid API key".to_string()),
                                    };
                                    
                                    let message = serde_json::to_string(&auth_result)
                                        .map_err(|e| format!("Failed to serialize auth result: {}", e))?;
                                    
                                    let mut connection = connection.lock().unwrap();
                                    connection.websocket.write_message(Message::Text(message))
                                        .map_err(|e| format!("Failed to send auth result: {}", e))?;
                                }
                            }
                        }
                    }
                    ClientMessage::Execute { code, id, options } if authenticated => {
                        // Get the session
                        let mut session_manager = session_manager.lock().unwrap();
                        let session = match session_manager.get_session_mut(&session_id) {
                            Some(session) => session,
                            None => {
                                // Send an error message
                                let error_message = ServerMessage::Error {
                                    execution_id: None,
                                    error: ErrorDetails {
                                        error_type: "SessionError".to_string(),
                                        message: format!("Session not found: {}", session_id),
                                        location: None,
                                    },
                                };
                                
                                let message = serde_json::to_string(&error_message)
                                    .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                                
                                let mut connection = connection.lock().unwrap();
                                connection.websocket.write_message(Message::Text(message))
                                    .map_err(|e| format!("Failed to send error message: {}", e))?;
                                
                                continue;
                            }
                        };
                        
                        // Update the last accessed time
                        session.last_accessed = chrono::Utc::now();
                        
                        // Extract options
                        let timeout = options.as_ref().and_then(|o| o.timeout).unwrap_or(5000);
                        let capture_output = options.as_ref().and_then(|o| o.capture_output).unwrap_or(true);
                        let async_execution = options.as_ref().and_then(|o| o.async_execution).unwrap_or(true);
                        
                        // Generate an execution ID
                        let execution_id = Uuid::new_v4().to_string();
                        
                        // Send execution start message
                        let start_message = ServerMessage::ExecutionStart {
                            id: execution_id.clone(),
                            request_id: id.clone(),
                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        };
                        
                        let message = serde_json::to_string(&start_message)
                            .map_err(|e| format!("Failed to serialize start message: {}", e))?;
                        
                        {
                            let mut connection = connection.lock().unwrap();
                            connection.websocket.write_message(Message::Text(message))
                                .map_err(|e| format!("Failed to send start message: {}", e))?;
                        }
                        
                        // Clone resources for the execution thread
                        let execution_engine = execution_engine.clone();
                        let connection = connection.clone();
                        let session_id = session_id.clone();
                        
                        if async_execution {
                            // Execute the code asynchronously
                            thread::spawn(move || {
                                // Get the session
                                let mut session_manager = session_manager.lock().unwrap();
                                let session = match session_manager.get_session_mut(&session_id) {
                                    Some(session) => session,
                                    None => {
                                        // Send an error message
                                        let error_message = ServerMessage::Error {
                                            execution_id: Some(execution_id.clone()),
                                            error: ErrorDetails {
                                                error_type: "SessionError".to_string(),
                                                message: format!("Session not found: {}", session_id),
                                                location: None,
                                            },
                                        };
                                        
                                        if let Ok(message) = serde_json::to_string(&error_message) {
                                            let mut connection = connection.lock().unwrap();
                                            let _ = connection.websocket.write_message(Message::Text(message));
                                        }
                                        
                                        return;
                                    }
                                };
                                
                                // Get the execution engine
                                let mut execution_engine = execution_engine.lock().unwrap();
                                
                                // Execute the code
                                match execution_engine.execute(session, &code, timeout, capture_output) {
                                    Ok(result) => {
                                        // Send the result
                                        let result_message = ServerMessage::ExecutionResult {
                                            execution_id: execution_id.clone(),
                                            result: result.result,
                                            duration: result.duration,
                                            status: result.status,
                                        };
                                        
                                        if let Ok(message) = serde_json::to_string(&result_message) {
                                            let mut connection = connection.lock().unwrap();
                                            let _ = connection.websocket.write_message(Message::Text(message));
                                        }
                                        
                                        // Send the output if any
                                        if let Some(output) = result.output {
                                            let output_message = ServerMessage::Output {
                                                execution_id: execution_id.clone(),
                                                content: output,
                                                channel: "stdout".to_string(),
                                            };
                                            
                                            if let Ok(message) = serde_json::to_string(&output_message) {
                                                let mut connection = connection.lock().unwrap();
                                                let _ = connection.websocket.write_message(Message::Text(message));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        // Send an error message
                                        let error_message = ServerMessage::Error {
                                            execution_id: Some(execution_id.clone()),
                                            error: ErrorDetails {
                                                error_type: "ExecutionError".to_string(),
                                                message: format!("Failed to execute code: {}", e),
                                                location: None,
                                            },
                                        };
                                        
                                        if let Ok(message) = serde_json::to_string(&error_message) {
                                            let mut connection = connection.lock().unwrap();
                                            let _ = connection.websocket.write_message(Message::Text(message));
                                        }
                                    }
                                }
                            });
                        } else {
                            // Execute the code synchronously
                            let mut execution_engine = execution_engine.lock().unwrap();
                            
                            match execution_engine.execute(session, &code, timeout, capture_output) {
                                Ok(result) => {
                                    // Send the result
                                    let result_message = ServerMessage::ExecutionResult {
                                        execution_id: execution_id.clone(),
                                        result: result.result,
                                        duration: result.duration,
                                        status: result.status,
                                    };
                                    
                                    let message = serde_json::to_string(&result_message)
                                        .map_err(|e| format!("Failed to serialize result message: {}", e))?;
                                    
                                    {
                                        let mut connection = connection.lock().unwrap();
                                        connection.websocket.write_message(Message::Text(message))
                                            .map_err(|e| format!("Failed to send result message: {}", e))?;
                                    }
                                    
                                    // Send the output if any
                                    if let Some(output) = result.output {
                                        let output_message = ServerMessage::Output {
                                            execution_id: execution_id.clone(),
                                            content: output,
                                            channel: "stdout".to_string(),
                                        };
                                        
                                        let message = serde_json::to_string(&output_message)
                                            .map_err(|e| format!("Failed to serialize output message: {}", e))?;
                                        
                                        let mut connection = connection.lock().unwrap();
                                        connection.websocket.write_message(Message::Text(message))
                                            .map_err(|e| format!("Failed to send output message: {}", e))?;
                                    }
                                }
                                Err(e) => {
                                    // Send an error message
                                    let error_message = ServerMessage::Error {
                                        execution_id: Some(execution_id.clone()),
                                        error: ErrorDetails {
                                            error_type: "ExecutionError".to_string(),
                                            message: format!("Failed to execute code: {}", e),
                                            location: None,
                                        },
                                    };
                                    
                                    let message = serde_json::to_string(&error_message)
                                        .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                                    
                                    let mut connection = connection.lock().unwrap();
                                    connection.websocket.write_message(Message::Text(message))
                                        .map_err(|e| format!("Failed to send error message: {}", e))?;
                                }
                            }
                        }
                    }
                    ClientMessage::Cancel { execution_id } if authenticated => {
                        // TODO: Implement execution cancellation
                        // This would require tracking active executions and providing a way to cancel them
                        
                        // For now, just send an error message
                        let error_message = ServerMessage::Error {
                            execution_id: Some(execution_id.clone()),
                            error: ErrorDetails {
                                error_type: "NotImplementedError".to_string(),
                                message: "Execution cancellation is not implemented yet".to_string(),
                                location: None,
                            },
                        };
                        
                        let message = serde_json::to_string(&error_message)
                            .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                        
                        let mut connection = connection.lock().unwrap();
                        connection.websocket.write_message(Message::Text(message))
                            .map_err(|e| format!("Failed to send error message: {}", e))?;
                    }
                    ClientMessage::Inspect { variable, depth } if authenticated => {
                        // Get the session
                        let session_manager = session_manager.lock().unwrap();
                        let session = match session_manager.get_session(&session_id) {
                            Some(session) => session,
                            None => {
                                // Send an error message
                                let error_message = ServerMessage::Error {
                                    execution_id: None,
                                    error: ErrorDetails {
                                        error_type: "SessionError".to_string(),
                                        message: format!("Session not found: {}", session_id),
                                        location: None,
                                    },
                                };
                                
                                let message = serde_json::to_string(&error_message)
                                    .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                                
                                let mut connection = connection.lock().unwrap();
                                connection.websocket.write_message(Message::Text(message))
                                    .map_err(|e| format!("Failed to send error message: {}", e))?;
                                
                                continue;
                            }
                        };
                        
                        // Get the variable value
                        let value = match session.variables.get(&variable) {
                            Some(value) => value.clone(),
                            None => {
                                // Send an error message
                                let error_message = ServerMessage::Error {
                                    execution_id: None,
                                    error: ErrorDetails {
                                        error_type: "VariableError".to_string(),
                                        message: format!("Variable not found: {}", variable),
                                        location: None,
                                    },
                                };
                                
                                let message = serde_json::to_string(&error_message)
                                    .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                                
                                let mut connection = connection.lock().unwrap();
                                connection.websocket.write_message(Message::Text(message))
                                    .map_err(|e| format!("Failed to send error message: {}", e))?;
                                
                                continue;
                            }
                        };
                        
                        // Create a custom result message
                        let result_message = serde_json::json!({
                            "type": "inspectionResult",
                            "variable": variable,
                            "value": value,
                            "depth": depth.unwrap_or(1)
                        });
                        
                        let message = serde_json::to_string(&result_message)
                            .map_err(|e| format!("Failed to serialize inspection result: {}", e))?;
                        
                        let mut connection = connection.lock().unwrap();
                        connection.websocket.write_message(Message::Text(message))
                            .map_err(|e| format!("Failed to send inspection result: {}", e))?;
                    }
                    ClientMessage::Ping { timestamp } => {
                        // Send a pong message
                        let pong_message = ServerMessage::Pong {
                            timestamp,
                        };
                        
                        let message = serde_json::to_string(&pong_message)
                            .map_err(|e| format!("Failed to serialize pong message: {}", e))?;
                        
                        let mut connection = connection.lock().unwrap();
                        connection.websocket.write_message(Message::Text(message))
                            .map_err(|e| format!("Failed to send pong message: {}", e))?;
                    }
                    _ if !authenticated => {
                        // Send an error message
                        let error_message = ServerMessage::Error {
                            execution_id: None,
                            error: ErrorDetails {
                                error_type: "AuthenticationError".to_string(),
                                message: "Authentication required".to_string(),
                                location: None,
                            },
                        };
                        
                        let message = serde_json::to_string(&error_message)
                            .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                        
                        let mut connection = connection.lock().unwrap();
                        connection.websocket.write_message(Message::Text(message))
                            .map_err(|e| format!("Failed to send error message: {}", e))?;
                    }
                }
            }
            Message::Binary(_) => {
                // Binary messages are not supported
                let error_message = ServerMessage::Error {
                    execution_id: None,
                    error: ErrorDetails {
                        error_type: "UnsupportedMessageType".to_string(),
                        message: "Binary messages are not supported".to_string(),
                        location: None,
                    },
                };
                
                let message = serde_json::to_string(&error_message)
                    .map_err(|e| format!("Failed to serialize error message: {}", e))?;
                
                let mut connection = connection.lock().unwrap();
                connection.websocket.write_message(Message::Text(message))
                    .map_err(|e| format!("Failed to send error message: {}", e))?;
            }
            Message::Ping(data) => {
                // Respond with a pong
                let mut connection = connection.lock().unwrap();
                connection.websocket.write_message(Message::Pong(data))
                    .map_err(|e| format!("Failed to send pong: {}", e))?;
            }
            Message::Pong(_) => {
                // Ignore pong messages
            }
            Message::Close(_) => {
                // Close the connection
                let mut connection = connection.lock().unwrap();
                connection.active = false;
                break;
            }
            Message::Frame(_) => {
                // Ignore frame messages
            }
        }
    }
    
    // Remove the connection from the connections map
    {
        let mut connections = connections.lock().unwrap();
        connections.remove(&connection_id);
    }
    
    Ok(())
}

/// Extract the session ID from the URL
fn extract_session_id_from_url(websocket: &WebSocket<TcpStream>) -> Result<String, String> {
    // Get the request
    let request = websocket.get_ref();
    
    // Get the peer address
    let peer_addr = match request.peer_addr() {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Failed to get peer address: {}", e)),
    };
    
    // For a real implementation, we would extract the session ID from the URL path
    // However, since we don't have access to the URL path in this simplified implementation,
    // we'll use a default session ID for demonstration purposes
    
    // In a real implementation, this would be something like:
    // let path = request.path();
    // let session_id = path.strip_prefix("/api/sessions/").unwrap_or("").split("/").next().unwrap_or("");
    
    // For now, we'll just return a default session ID
    Ok("default_session".to_string())
}

/// Clean up inactive connections
fn clean_up_connections(connections: &Arc<Mutex<HashMap<String, Arc<Mutex<WebSocketConnection>>>>>) {
    let mut connections = connections.lock().unwrap();
    
    // Find inactive connections
    let inactive_connections: Vec<String> = connections.iter()
        .filter_map(|(id, connection)| {
            let connection = connection.lock().unwrap();
            if !connection.active || connection.last_activity.elapsed() > Duration::from_secs(300) {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect();
    
    // Remove inactive connections
    for id in inactive_connections {
        connections.remove(&id);
    }
}
