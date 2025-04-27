// JSON-RPC communication layer for LSP-like Component
//
// This module implements the JSON-RPC communication protocol used by the
// Language Server Protocol, handling message parsing, formatting, and transport.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use crate::language_hub_server::lsp::protocol::{Request, Response, Notification, ErrorCode};

/// Content type for LSP messages
const CONTENT_TYPE_JSON: &str = "application/vscode-jsonrpc; charset=utf-8";

/// Header field for content length
const HEADER_CONTENT_LENGTH: &str = "Content-Length: ";

/// Header field for content type
const HEADER_CONTENT_TYPE: &str = "Content-Type: ";

/// Carriage return and line feed characters
const CRLF: &str = "\r\n";

/// JSON-RPC message reader
pub struct MessageReader {
    /// The underlying reader
    reader: BufReader<TcpStream>,
    
    /// Buffer for storing partial messages
    buffer: String,
}

impl MessageReader {
    /// Create a new message reader
    pub fn new(stream: TcpStream) -> Self {
        MessageReader {
            reader: BufReader::new(stream),
            buffer: String::new(),
        }
    }
    
    /// Read the next message from the stream
    pub fn read_message(&mut self) -> Result<Value, String> {
        // Read headers
        let mut content_length: Option<usize> = None;
        let mut line = String::new();
        
        loop {
            line.clear();
            let bytes_read = self.reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                return Err("Connection closed".to_string());
            }
            
            let line = line.trim();
            if line.is_empty() {
                // Empty line indicates end of headers
                break;
            }
            
            if line.starts_with(HEADER_CONTENT_LENGTH) {
                let length_str = &line[HEADER_CONTENT_LENGTH.len()..];
                content_length = Some(length_str.parse::<usize>().map_err(|e| e.to_string())?);
            }
            
            // We ignore Content-Type header for now
        }
        
        // Ensure we have a content length
        let content_length = content_length.ok_or("Missing Content-Length header".to_string())?;
        
        // Read the message content
        let mut content = vec![0; content_length];
        self.reader.read_exact(&mut content).map_err(|e| e.to_string())?;
        
        // Parse the JSON content
        let content_str = String::from_utf8(content).map_err(|e| e.to_string())?;
        let message = serde_json::from_str(&content_str).map_err(|e| e.to_string())?;
        
        Ok(message)
    }
}

/// JSON-RPC message writer
pub struct MessageWriter {
    /// The underlying writer
    writer: Arc<Mutex<TcpStream>>,
}

impl MessageWriter {
    /// Create a new message writer
    pub fn new(stream: TcpStream) -> Self {
        MessageWriter {
            writer: Arc::new(Mutex::new(stream)),
        }
    }
    
    /// Write a message to the stream
    pub fn write_message(&self, message: &Value) -> Result<(), String> {
        let content = serde_json::to_string(message).map_err(|e| e.to_string())?;
        let content_length = content.len();
        
        let mut writer = self.writer.lock().unwrap();
        
        // Write headers
        write!(writer, "{}{}{}", HEADER_CONTENT_LENGTH, content_length, CRLF).map_err(|e| e.to_string())?;
        write!(writer, "{}{}{}", HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON, CRLF).map_err(|e| e.to_string())?;
        write!(writer, "{}", CRLF).map_err(|e| e.to_string())?;
        
        // Write content
        writer.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// Write a response to the stream
    pub fn write_response(&self, response: &Response) -> Result<(), String> {
        let message = serde_json::to_value(response).map_err(|e| e.to_string())?;
        self.write_message(&message)
    }
    
    /// Write a notification to the stream
    pub fn write_notification(&self, notification: &Notification) -> Result<(), String> {
        let message = serde_json::to_value(notification).map_err(|e| e.to_string())?;
        self.write_message(&message)
    }
}

/// JSON-RPC connection handler
pub struct JsonRpcConnection {
    /// The message reader
    reader: MessageReader,
    
    /// The message writer
    writer: MessageWriter,
    
    /// Flag indicating whether the connection is open
    is_open: bool,
}

impl JsonRpcConnection {
    /// Create a new JSON-RPC connection
    pub fn new(stream: TcpStream) -> Result<Self, String> {
        let reader_stream = stream.try_clone().map_err(|e| e.to_string())?;
        let writer_stream = stream;
        
        Ok(JsonRpcConnection {
            reader: MessageReader::new(reader_stream),
            writer: MessageWriter::new(writer_stream),
            is_open: true,
        })
    }
    
    /// Read the next message from the connection
    pub fn read_message(&mut self) -> Result<Value, String> {
        if !self.is_open {
            return Err("Connection is closed".to_string());
        }
        
        self.reader.read_message()
    }
    
    /// Write a message to the connection
    pub fn write_message(&self, message: &Value) -> Result<(), String> {
        if !self.is_open {
            return Err("Connection is closed".to_string());
        }
        
        self.writer.write_message(message)
    }
    
    /// Write a response to the connection
    pub fn write_response(&self, response: &Response) -> Result<(), String> {
        if !self.is_open {
            return Err("Connection is closed".to_string());
        }
        
        self.writer.write_response(response)
    }
    
    /// Write a notification to the connection
    pub fn write_notification(&self, notification: &Notification) -> Result<(), String> {
        if !self.is_open {
            return Err("Connection is closed".to_string());
        }
        
        self.writer.write_notification(notification)
    }
    
    /// Close the connection
    pub fn close(&mut self) {
        self.is_open = false;
    }
    
    /// Check if the connection is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Process a JSON-RPC message
pub fn process_message(message: Value) -> Result<MessageType, String> {
    // Check if it's a request, response, or notification
    if message.get("method").is_some() {
        if message.get("id").is_some() {
            // It's a request
            let request: Request = serde_json::from_value(message)
                .map_err(|e| format!("Invalid request format: {}", e))?;
            Ok(MessageType::Request(request))
        } else {
            // It's a notification
            let notification: Notification = serde_json::from_value(message)
                .map_err(|e| format!("Invalid notification format: {}", e))?;
            Ok(MessageType::Notification(notification))
        }
    } else if message.get("id").is_some() {
        // It's a response
        let response: Response = serde_json::from_value(message)
            .map_err(|e| format!("Invalid response format: {}", e))?;
        Ok(MessageType::Response(response))
    } else {
        Err("Invalid JSON-RPC message format".to_string())
    }
}

/// Types of JSON-RPC messages
pub enum MessageType {
    /// A request message
    Request(Request),
    
    /// A response message
    Response(Response),
    
    /// A notification message
    Notification(Notification),
}
