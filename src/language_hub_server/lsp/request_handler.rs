// Request routing implementation for LSP-like Component
//
// This module implements the routing of LSP requests to the appropriate
// handlers, including registration of handlers for specific methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;

use crate::language_hub_server::lsp::protocol::{Request, Response, Notification, RequestId, ErrorCode};
use crate::language_hub_server::lsp::document_sync::{DocumentSyncManager, SharedDocumentSyncManager};
use crate::language_hub_server::lsp::anarchy_parser_integration::{AnarchyParserIntegration, SharedAnarchyParserIntegration};

/// LSP request handler implementation
pub struct LspRequestHandler {
    /// The document synchronization manager
    document_sync: SharedDocumentSyncManager,
    
    /// The Anarchy parser integration
    parser_integration: SharedAnarchyParserIntegration,
    
    /// Map of method names to request handlers
    request_handlers: HashMap<String, Box<dyn Fn(Value) -> Result<Value, (ErrorCode, String)> + Send + Sync>>,
    
    /// Map of method names to notification handlers
    notification_handlers: HashMap<String, Box<dyn Fn(Value) -> () + Send + Sync>>,
    
    /// Server capabilities
    capabilities: Value,
    
    /// Server initialization status
    initialized: bool,
    
    /// Server shutdown status
    shutdown_requested: bool,
}

impl LspRequestHandler {
    /// Create a new LSP request handler
    pub fn new(
        document_sync: SharedDocumentSyncManager,
        parser_integration: SharedAnarchyParserIntegration
    ) -> Self {
        let mut handler = LspRequestHandler {
            document_sync,
            parser_integration,
            request_handlers: HashMap::new(),
            notification_handlers: HashMap::new(),
            capabilities: Self::create_default_capabilities(),
            initialized: false,
            shutdown_requested: false,
        };
        
        // Register default handlers
        handler.register_default_handlers();
        
        handler
    }
    
    /// Handle an LSP request
    pub fn handle_request(&self, request: Request) -> Response {
        let method = &request.method;
        let params = request.params.clone();
        let id = request.id.clone();
        
        // Check for shutdown status
        if self.shutdown_requested && method != "exit" {
            return Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(crate::language_hub_server::lsp::protocol::ResponseError {
                    code: ErrorCode::InvalidRequest as i64,
                    message: "Server is shutting down".to_string(),
                    data: None,
                }),
            };
        }
        
        // Check for initialization status
        if !self.initialized && method != "initialize" && method != "exit" {
            return Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(crate::language_hub_server::lsp::protocol::ResponseError {
                    code: ErrorCode::ServerNotInitialized as i64,
                    message: "Server not initialized".to_string(),
                    data: None,
                }),
            };
        }
        
        // Handle the request
        if let Some(handler) = self.request_handlers.get(method) {
            match handler(params) {
                Ok(result) => Response {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(result),
                    error: None,
                },
                Err((code, message)) => Response {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(crate::language_hub_server::lsp::protocol::ResponseError {
                        code: code as i64,
                        message,
                        data: None,
                    }),
                },
            }
        } else {
            Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(crate::language_hub_server::lsp::protocol::ResponseError {
                    code: ErrorCode::MethodNotFound as i64,
                    message: format!("Method not found: {}", method),
                    data: None,
                }),
            }
        }
    }
    
    /// Handle an LSP notification
    pub fn handle_notification(&self, notification: Notification) {
        let method = &notification.method;
        let params = notification.params.clone();
        
        // Check for shutdown status
        if self.shutdown_requested && method != "exit" {
            return;
        }
        
        // Check for initialization status
        if !self.initialized && method != "initialized" && method != "exit" {
            return;
        }
        
        // Handle the notification
        if let Some(handler) = self.notification_handlers.get(method) {
            handler(params);
        }
    }
    
    /// Register a request handler
    pub fn register_request_handler<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(Value) -> Result<Value, (ErrorCode, String)> + Send + Sync + 'static,
    {
        self.request_handlers.insert(method.to_string(), Box::new(handler));
    }
    
    /// Register a notification handler
    pub fn register_notification_handler<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(Value) -> () + Send + Sync + 'static,
    {
        self.notification_handlers.insert(method.to_string(), Box::new(handler));
    }
    
    /// Register the default request and notification handlers
    fn register_default_handlers(&mut self) {
        // Clone necessary references for the handlers
        let document_sync = self.document_sync.clone();
        let parser_integration = self.parser_integration.clone();
        let capabilities = self.capabilities.clone();
        
        // Register initialize request handler
        let init_capabilities = capabilities.clone();
        self.register_request_handler("initialize", move |params| {
            // Process initialize params
            println!("Received initialize request");
            
            // Return server capabilities
            Ok(serde_json::json!({
                "capabilities": init_capabilities
            }))
        });
        
        // Register shutdown request handler
        let mut shutdown_requested = self.shutdown_requested;
        self.register_request_handler("shutdown", move |_params| {
            println!("Received shutdown request");
            shutdown_requested = true;
            Ok(serde_json::json!(null))
        });
        
        // Register exit notification handler
        let exit_shutdown_requested = self.shutdown_requested;
        self.register_notification_handler("exit", move |_params| {
            println!("Received exit notification");
            if exit_shutdown_requested {
                // Exit with success code
                std::process::exit(0);
            } else {
                // Exit with error code
                std::process::exit(1);
            }
        });
        
        // Register initialized notification handler
        let mut initialized = self.initialized;
        self.register_notification_handler("initialized", move |_params| {
            println!("Received initialized notification");
            initialized = true;
        });
        
        // Register textDocument/didOpen notification handler
        let doc_sync1 = document_sync.clone();
        self.register_notification_handler("textDocument/didOpen", move |params| {
            println!("Received textDocument/didOpen notification");
            
            // Extract parameters
            if let Some(params) = params.as_object() {
                if let Some(text_document) = params.get("textDocument").and_then(|v| v.as_object()) {
                    let uri = text_document.get("uri").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let language_id = text_document.get("languageId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let version = text_document.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
                    let text = text_document.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    
                    // Handle document open
                    let mut sync = doc_sync1.lock().unwrap();
                    if let Err(e) = sync.handle_document_open(&uri, &language_id, version, &text) {
                        eprintln!("Error handling document open: {}", e);
                    }
                }
            }
        });
        
        // Register textDocument/didClose notification handler
        let doc_sync2 = document_sync.clone();
        self.register_notification_handler("textDocument/didClose", move |params| {
            println!("Received textDocument/didClose notification");
            
            // Extract parameters
            if let Some(params) = params.as_object() {
                if let Some(text_document) = params.get("textDocument").and_then(|v| v.as_object()) {
                    let uri = text_document.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    
                    // Handle document close
                    let mut sync = doc_sync2.lock().unwrap();
                    if let Err(e) = sync.handle_document_close(uri) {
                        eprintln!("Error handling document close: {}", e);
                    }
                }
            }
        });
        
        // Register textDocument/didChange notification handler
        let doc_sync3 = document_sync.clone();
        self.register_notification_handler("textDocument/didChange", move |params| {
            println!("Received textDocument/didChange notification");
            
            // Extract parameters
            if let Some(params) = params.as_object() {
                if let Some(text_document) = params.get("textDocument").and_then(|v| v.as_object()) {
                    let uri = text_document.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    let version = text_document.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
                    
                    // Extract content changes
                    let content_changes = params.get("contentChanges")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|change| {
                                    if let Some(change_obj) = change.as_object() {
                                        let text = change_obj.get("text")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        
                                        let range = change_obj.get("range").and_then(|v| {
                                            if let Some(range_obj) = v.as_object() {
                                                let start = range_obj.get("start").and_then(|v| {
                                                    if let Some(pos_obj) = v.as_object() {
                                                        let line = pos_obj.get("line")
                                                            .and_then(|v| v.as_u64())
                                                            .unwrap_or(0) as u32;
                                                        let character = pos_obj.get("character")
                                                            .and_then(|v| v.as_u64())
                                                            .unwrap_or(0) as u32;
                                                        
                                                        Some(crate::language_hub_server::lsp::protocol::Position {
                                                            line,
                                                            character,
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                });
                                                
                                                let end = range_obj.get("end").and_then(|v| {
                                                    if let Some(pos_obj) = v.as_object() {
                                                        let line = pos_obj.get("line")
                                                            .and_then(|v| v.as_u64())
                                                            .unwrap_or(0) as u32;
                                                        let character = pos_obj.get("character")
                                                            .and_then(|v| v.as_u64())
                                                            .unwrap_or(0) as u32;
                                                        
                                                        Some(crate::language_hub_server::lsp::protocol::Position {
                                                            line,
                                                            character,
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                });
                                                
                                                if let (Some(start), Some(end)) = (start, end) {
                                                    Some(crate::language_hub_server::lsp::protocol::Range {
                                                        start,
                                                        end,
                                                    })
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        });
                                        
                                        Some(crate::language_hub_server::lsp::document::TextDocumentContentChangeEvent {
                                            range,
                                            text,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_else(Vec::new);
                    
                    // Handle document change
                    let mut sync = doc_sync3.lock().unwrap();
                    if let Err(e) = sync.handle_document_change(uri, version, content_changes) {
                        eprintln!("Error handling document change: {}", e);
                    }
                }
            }
        });
        
        // Register textDocument/completion request handler
        let parser_int1 = parser_integration.clone();
        let doc_sync4 = document_sync.clone();
        self.register_request_handler("textDocument/completion", move |params| {
            println!("Received textDocument/completion request");
            
            // Extract parameters
            if let Some(params) = params.as_object() {
                if let Some(text_document) = params.get("textDocument").and_then(|v| v.as_object()) {
                    let uri = text_document.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    
                    // Extract position
                    let position = params.get("position").and_then(|v| {
                        if let Some(pos_obj) = v.as_object() {
                            let line = pos_obj.get("line")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            let character = pos_obj.get("character")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            
                            Some(crate::language_hub_server::lsp::protocol::Position {
                                line,
                                character,
                            })
                        } else {
                            None
                        }
                    });
                    
                    if let Some(position) = position {
                        // Get the document
                        let sync = doc_sync4.lock().unwrap();
                        if let Some(document) = sync.get_document(uri) {
                            // Get completions
                            let mut parser = parser_int1.lock().unwrap();
                            let completions = parser.get_completions(&document, position);
                            
                            // Convert completions to JSON
                            let items = completions.iter().map(|item| {
                                let mut json = serde_json::Map::new();
                                json.insert("label".to_string(), serde_json::Value::String(item.label.clone()));
                                json.insert("kind".to_string(), serde_json::Value::Number(serde_json::Number::from(item.kind as u8)));
                                
                                if let Some(ref detail) = item.detail {
                                    json.insert("detail".to_string(), serde_json::Value::String(detail.clone()));
                                }
                                
                                if let Some(ref insert_text) = item.insert_text {
                                    json.insert("insertText".to_string(), serde_json::Value::String(insert_text.clone()));
                                    json.insert("insertTextFormat".to_string(), serde_json::Value::Number(serde_json::Number::from(item.insert_text_format as u8)));
                                }
                                
                                serde_json::Value::Object(json)
                            }).collect::<Vec<_>>();
                            
                            return Ok(serde_json::json!({
                                "isIncomplete": false,
                                "items": items
                            }));
                        }
                    }
                }
            }
            
            // Return empty completion list if parameters are invalid
            Ok(serde_json::json!({
                "isIncomplete": false,
                "items": []
            }))
        });
        
        // Register textDocument/diagnostic request handler
        let parser_int2 = parser_integration.clone();
        let doc_sync5 = document_sync.clone();
        self.register_request_handler("textDocument/diagnostic", move |params| {
            println!("Received textDocument/diagnostic request");
            
            // Extract parameters
            if let Some(params) = params.as_object() {
                if let Some(text_document) = params.get("textDocument").and_then(|v| v.as_object()) {
                    let uri = text_document.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    
                    // Get the document
                    let sync = doc_sync5.lock().unwrap();
                    if let Some(document) = sync.get_document(uri) {
                        // Get diagnostics
                        let mut parser = parser_int2.lock().unwrap();
                        let diagnostics = parser.get_diagnostics(&document);
                        
                        // Convert diagnostics to JSON
                        let items = diagnostics.iter().map(|diag| {
                            let mut json = serde_json::Map::new();
                            
                            // Range
                            let mut range = serde_json::Map::new();
                            
                            let mut start = serde_json::Map::new();
                            start.insert("line".to_string(), serde_json::Value::Number(serde_json::Number::from(diag.range.start.line)));
                            start.insert("character".to_string(), serde_json::Value::Number(serde_json::Number::from(diag.range.start.character)));
                            
                            let mut end = serde_json::Map::new();
                            end.insert("line".to_string(), serde_json::Value::Number(serde_json::Number::from(diag.range.end.line)));
                            end.insert("character".to_string(), serde_json::Value::Number(serde_json::Number::from(diag.range.end.character)));
                            
                            range.insert("start".to_string(), serde_json::Value::Object(start));
                            range.insert("end".to_string(), serde_json::Value::Object(end));
                            
                            json.insert("range".to_string(), serde_json::Value::Object(range));
                            json.insert("message".to_string(), serde_json::Value::String(diag.message.clone()));
                            json.insert("severity".to_string(), serde_json::Value::Number(serde_json::Number::from(diag.severity as u8)));
                            
                            if let Some(ref code) = diag.code {
                                json.insert("code".to_string(), serde_json::Value::String(code.clone()));
                            }
                            
                            serde_json::Value::Object(json)
                        }).collect::<Vec<_>>();
                        
                        return Ok(serde_json::json!({
                            "items": items
                        }));
                    }
                }
            }
            
            // Return empty diagnostic list if parameters are invalid
            Ok(serde_json::json!({
                "items": []
            }))
        });
    }
    
    /// Create default server capabilities
    fn create_default_capabilities() -> Value {
        serde_json::json!({
            "textDocumentSync": {
                "openClose": true,
                "change": 2, // Incremental
                "willSave": false,
                "willSaveWaitUntil": false,
                "save": { "includeText": false }
            },
            "completionProvider": {
                "resolveProvider": true,
                "triggerCharacters": [".", ":", "("]
            },
            "hoverProvider": true,
            "signatureHelpProvider": {
                "triggerCharacters": ["(", ","]
            },
            "definitionProvider": true,
            "referencesProvider": true,
            "documentHighlightProvider": true,
            "documentSymbolProvider": true,
            "workspaceSymbolProvider": true,
            "codeActionProvider": true,
            "codeLensProvider": {
                "resolveProvider": true
            },
            "documentFormattingProvider": true,
            "documentRangeFormattingProvider": true,
            "documentOnTypeFormattingProvider": {
                "firstTriggerCharacter": "}",
                "moreTriggerCharacter": ["\n", ";"]
            },
            "renameProvider": true
        })
    }
}

/// Shared LSP request handler that can be used across threads
pub type SharedLspRequestHandler = Arc<Mutex<LspRequestHandler>>;

/// Create a new shared LSP request handler
pub fn create_shared_lsp_request_handler(
    document_sync: SharedDocumentSyncManager,
    parser_integration: SharedAnarchyParserIntegration
) -> SharedLspRequestHandler {
    Arc::new(Mutex::new(LspRequestHandler::new(document_sync, parser_integration)))
}
