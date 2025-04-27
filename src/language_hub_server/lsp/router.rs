// Request router module for LSP-like Component
//
// This module handles routing of incoming requests to the appropriate
// handler functions based on the method name.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use crate::language_hub_server::lsp::protocol::{Request, Response, Notification, RequestId, ErrorCode};

/// Type definition for request handler functions
type RequestHandler = Box<dyn Fn(Value) -> Result<Value, (ErrorCode, String)> + Send + Sync>;

/// Type definition for notification handler functions
type NotificationHandler = Box<dyn Fn(Value) -> () + Send + Sync>;

/// Request router for dispatching requests to appropriate handlers
pub struct RequestRouter {
    /// Map of method names to request handlers
    request_handlers: HashMap<String, RequestHandler>,
    
    /// Map of method names to notification handlers
    notification_handlers: HashMap<String, NotificationHandler>,
}

impl RequestRouter {
    /// Create a new request router
    pub fn new() -> Self {
        RequestRouter {
            request_handlers: HashMap::new(),
            notification_handlers: HashMap::new(),
        }
    }
    
    /// Register a request handler for a specific method
    pub fn register_request_handler<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(Value) -> Result<Value, (ErrorCode, String)> + Send + Sync + 'static,
    {
        self.request_handlers.insert(method.to_string(), Box::new(handler));
    }
    
    /// Register a notification handler for a specific method
    pub fn register_notification_handler<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(Value) -> () + Send + Sync + 'static,
    {
        self.notification_handlers.insert(method.to_string(), Box::new(handler));
    }
    
    /// Handle a request and return a response
    pub fn handle_request(&self, request: Request) -> Response {
        let method = &request.method;
        let params = request.params.clone();
        let id = request.id.clone();
        
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
    
    /// Handle a notification (no response)
    pub fn handle_notification(&self, notification: Notification) {
        let method = &notification.method;
        let params = notification.params.clone();
        
        if let Some(handler) = self.notification_handlers.get(method) {
            handler(params);
        }
    }
}

/// Shared router that can be used across threads
pub type SharedRouter = Arc<Mutex<RequestRouter>>;

/// Create a new shared router
pub fn create_shared_router() -> SharedRouter {
    Arc::new(Mutex::new(RequestRouter::new()))
}
