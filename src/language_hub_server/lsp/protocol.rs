// Protocol module for LSP-like Component
//
// This module defines the JSON-RPC protocol used for communication
// between the LSP server and clients.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// JSON-RPC request object
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    /// The JSON-RPC protocol version
    pub jsonrpc: String,
    
    /// The method to be invoked
    pub method: String,
    
    /// The method parameters
    #[serde(default)]
    pub params: serde_json::Value,
    
    /// The request id
    pub id: RequestId,
}

/// JSON-RPC response object
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    /// The JSON-RPC protocol version
    pub jsonrpc: String,
    
    /// The request id
    pub id: RequestId,
    
    /// The result of the request (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    /// The error object (if request failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// JSON-RPC notification object (request without id)
#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    /// The JSON-RPC protocol version
    pub jsonrpc: String,
    
    /// The method to be invoked
    pub method: String,
    
    /// The method parameters
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Request ID - can be string, number, or null
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

/// Response error object
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    /// The error code
    pub code: i64,
    
    /// The error message
    pub message: String,
    
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Standard error codes
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    /// Invalid JSON was received by the server.
    ParseError = -32700,
    
    /// The JSON sent is not a valid Request object.
    InvalidRequest = -32600,
    
    /// The method does not exist / is not available.
    MethodNotFound = -32601,
    
    /// Invalid method parameter(s).
    InvalidParams = -32602,
    
    /// Internal JSON-RPC error.
    InternalError = -32603,
    
    /// Reserved for implementation-defined server-errors.
    ServerErrorStart = -32099,
    
    /// Reserved for implementation-defined server-errors.
    ServerErrorEnd = -32000,
    
    /// Request was cancelled.
    RequestCancelled = -32800,
    
    /// Content modified.
    ContentModified = -32801,
}

/// Position in a text document expressed as zero-based line and character offset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line position (zero-based).
    pub line: u32,
    
    /// Character offset on a line (zero-based).
    pub character: u32,
}

/// A range in a text document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    /// The range's start position.
    pub start: Position,
    
    /// The range's end position.
    pub end: Position,
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// The document's URI.
    pub uri: String,
    
    /// The position within the document.
    pub range: Range,
}

/// Create a new JSON-RPC request
pub fn create_request(method: &str, params: serde_json::Value, id: RequestId) -> Request {
    Request {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id,
    }
}

/// Create a new JSON-RPC notification
pub fn create_notification(method: &str, params: serde_json::Value) -> Notification {
    Notification {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
    }
}

/// Create a new JSON-RPC success response
pub fn create_success_response(id: RequestId, result: serde_json::Value) -> Response {
    Response {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

/// Create a new JSON-RPC error response
pub fn create_error_response(id: RequestId, code: ErrorCode, message: &str, data: Option<serde_json::Value>) -> Response {
    Response {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(ResponseError {
            code: code as i64,
            message: message.to_string(),
            data,
        }),
    }
}
