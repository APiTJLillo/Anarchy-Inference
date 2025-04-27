// Main server implementation for LSP-like Component
//
// This module integrates all the components of the LSP server and provides
// the main entry point for starting and running the server.

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{BufReader, BufWriter};

use crate::language_hub_server::lsp::document::{Document, DocumentManager};
use crate::language_hub_server::lsp::document_sync::{DocumentSyncManager, TextDocumentSyncKind, create_shared_document_sync_manager};
use crate::language_hub_server::lsp::anarchy_parser_integration::{AnarchyParserIntegration, create_shared_anarchy_parser_integration};
use crate::language_hub_server::lsp::request_handler::{LspRequestHandler, create_shared_lsp_request_handler};
use crate::language_hub_server::lsp::json_rpc::{JsonRpcConnection, MessageType};
use crate::language_hub_server::lsp::protocol::{Request, Response, Notification};

/// LSP server implementation
pub struct LspServer {
    /// The host address to bind to
    host: String,
    
    /// The port to listen on
    port: u16,
    
    /// The document manager
    document_manager: Arc<Mutex<DocumentManager>>,
    
    /// The document synchronization manager
    document_sync: Arc<Mutex<DocumentSyncManager>>,
    
    /// The Anarchy parser integration
    parser_integration: Arc<Mutex<AnarchyParserIntegration>>,
    
    /// The request handler
    request_handler: Arc<Mutex<LspRequestHandler>>,
    
    /// Flag indicating whether the server is running
    running: Arc<Mutex<bool>>,
}

impl LspServer {
    /// Create a new LSP server
    pub fn new(host: &str, port: u16, anarchy_path: &str) -> Result<Self, String> {
        // Create the document manager
        let document_manager = Arc::new(Mutex::new(DocumentManager::new()));
        
        // Create the document sync manager
        let document_sync = create_shared_document_sync_manager(
            document_manager.clone(),
            TextDocumentSyncKind::Incremental
        );
        
        // Create the parser integration
        let parser_integration = create_shared_anarchy_parser_integration(anarchy_path);
        
        // Create the request handler
        let request_handler = create_shared_lsp_request_handler(
            document_sync.clone(),
            parser_integration.clone()
        );
        
        Ok(LspServer {
            host: host.to_string(),
            port,
            document_manager,
            document_sync,
            parser_integration,
            request_handler,
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Start the server
    pub fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err("Server is already running".to_string());
        }
        
        *running = true;
        
        // Clone the necessary Arc references for the server thread
        let request_handler = self.request_handler.clone();
        let running_flag = self.running.clone();
        let host = self.host.clone();
        let port = self.port;
        
        // Start the server in a separate thread
        thread::spawn(move || {
            let address = format!("{}:{}", host, port);
            let listener = match TcpListener::bind(&address) {
                Ok(listener) => listener,
                Err(e) => {
                    eprintln!("Failed to bind to {}: {}", address, e);
                    let mut running = running_flag.lock().unwrap();
                    *running = false;
                    return;
                }
            };
            
            println!("LSP server listening on {}", address);
            
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        // Clone the Arc references for the client thread
                        let request_handler = request_handler.clone();
                        
                        // Handle each client in a separate thread
                        thread::spawn(move || {
                            if let Err(e) = handle_client(stream, request_handler) {
                                eprintln!("Error handling client: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
                
                // Check if we should stop the server
                let running = running_flag.lock().unwrap();
                if !*running {
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the server
    pub fn stop(&self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err("Server is not running".to_string());
        }
        
        *running = false;
        
        // TODO: Implement proper shutdown by closing the listener
        
        Ok(())
    }
    
    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        let running = self.running.lock().unwrap();
        *running
    }
}

/// Handle a client connection
fn handle_client(
    stream: TcpStream,
    request_handler: Arc<Mutex<LspRequestHandler>>
) -> Result<(), String> {
    // Create a JSON-RPC connection
    let mut connection = JsonRpcConnection::new(stream)?;
    
    // Process messages from the client
    while connection.is_open() {
        // Read a message
        let message = match connection.read_message() {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
        };
        
        // Process the message
        match crate::language_hub_server::lsp::json_rpc::process_message(message) {
            Ok(MessageType::Request(request)) => {
                // Handle the request
                let handler = request_handler.lock().unwrap();
                let response = handler.handle_request(request);
                
                // Send the response
                if let Err(e) = connection.write_response(&response) {
                    eprintln!("Error writing response: {}", e);
                    break;
                }
            }
            Ok(MessageType::Notification(notification)) => {
                // Handle the notification
                let handler = request_handler.lock().unwrap();
                handler.handle_notification(notification);
            }
            Ok(MessageType::Response(_)) => {
                // We don't expect responses from clients
                eprintln!("Unexpected response from client");
            }
            Err(e) => {
                eprintln!("Error processing message: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
