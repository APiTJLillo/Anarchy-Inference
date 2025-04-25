// src/external_tools/web.rs - Web interface for external tools

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use tokio_tungstenite::tungstenite::Message;
use futures::stream::StreamExt;
use futures::SinkExt;
use scraper::{Html, Selector};
use url::Url;
use crate::error::LangError;
use crate::value::Value;
use super::common::{ExternalTool, ToolParams, ToolResult, ToolStatus, ToolError, ToolContext};

/// Connection to a WebSocket
pub struct WebSocketConnection {
    /// The WebSocket stream
    stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    
    /// The URL of the WebSocket
    url: String,
    
    /// Whether the connection is open
    is_open: bool,
}

/// Rate limiter for web requests
pub struct RateLimiter {
    /// Maximum requests per minute
    max_rpm: u32,
    
    /// Current request count
    request_count: Arc<Mutex<u32>>,
    
    /// Last reset time
    last_reset: Arc<Mutex<std::time::Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_rpm: u32) -> Self {
        Self {
            max_rpm,
            request_count: Arc::new(Mutex::new(0)),
            last_reset: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }
    
    /// Check if a request is allowed
    pub fn allow_request(&self) -> bool {
        let mut count = self.request_count.lock().unwrap();
        let mut last_reset = self.last_reset.lock().unwrap();
        
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(*last_reset);
        
        // Reset counter every minute
        if elapsed.as_secs() >= 60 {
            *count = 0;
            *last_reset = now;
        }
        
        if *count >= self.max_rpm {
            false
        } else {
            *count += 1;
            true
        }
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Status code
    pub status: u16,
    
    /// Headers
    pub headers: HashMap<String, String>,
    
    /// Body
    pub body: String,
}

/// HTML document
#[derive(Debug, Clone)]
pub struct HtmlDocument {
    /// Title
    pub title: Option<String>,
    
    /// Body text
    pub body_text: String,
    
    /// Links
    pub links: Vec<HtmlLink>,
    
    /// Images
    pub images: Vec<HtmlImage>,
}

/// HTML link
#[derive(Debug, Clone)]
pub struct HtmlLink {
    /// URL
    pub url: String,
    
    /// Text
    pub text: String,
}

/// HTML image
#[derive(Debug, Clone)]
pub struct HtmlImage {
    /// URL
    pub url: String,
    
    /// Alt text
    pub alt: Option<String>,
}

/// Web tool for HTTP requests and WebSocket communication
pub struct WebTool {
    /// HTTP client
    http_client: reqwest::Client,
    
    /// WebSocket connections
    ws_connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    
    /// Rate limiter
    rate_limiter: RateLimiter,
}

impl WebTool {
    /// Create a new web tool
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            ws_connections: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: RateLimiter::new(100), // 100 requests per minute by default
        }
    }
    
    /// Send an HTTP request
    pub async fn send_request(&self, 
                             method: &str, 
                             url: &str, 
                             headers: Option<HashMap<String, String>>, 
                             body: Option<String>) -> Result<HttpResponse, ToolError> {
        // Check rate limit
        if !self.rate_limiter.allow_request() {
            return Err(ToolError::new(429, "Rate limit exceeded"));
        }
        
        // Parse method
        let method = match method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "PATCH" => Method::PATCH,
            _ => return Err(ToolError::new(400, format!("Invalid HTTP method: {}", method))),
        };
        
        // Build request
        let mut request = self.http_client.request(method, url);
        
        // Add headers
        if let Some(headers) = headers {
            let mut header_map = HeaderMap::new();
            for (key, value) in headers {
                if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), HeaderValue::from_str(&value)) {
                    header_map.insert(name, val);
                }
            }
            request = request.headers(header_map);
        }
        
        // Add body
        if let Some(body) = body {
            request = request.body(body);
        }
        
        // Send request
        let response = request.send().await
            .map_err(|e| ToolError::new(500, format!("Failed to send request: {}", e)))?;
        
        // Get status
        let status = response.status().as_u16();
        
        // Get headers
        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                response_headers.insert(key.to_string(), value_str.to_string());
            }
        }
        
        // Get body
        let body = response.text().await
            .map_err(|e| ToolError::new(500, format!("Failed to read response body: {}", e)))?;
        
        Ok(HttpResponse {
            status,
            headers: response_headers,
            body,
        })
    }
    
    /// Connect to a WebSocket
    pub async fn connect_websocket(&self, url: &str) -> Result<String, ToolError> {
        // Check rate limit
        if !self.rate_limiter.allow_request() {
            return Err(ToolError::new(429, "Rate limit exceeded"));
        }
        
        // Parse URL
        let url_parsed = Url::parse(url)
            .map_err(|e| ToolError::new(400, format!("Invalid WebSocket URL: {}", e)))?;
        
        // Connect to WebSocket
        let (stream, _) = tokio_tungstenite::connect_async(url_parsed).await
            .map_err(|e| ToolError::new(500, format!("Failed to connect to WebSocket: {}", e)))?;
        
        // Generate connection ID
        let connection_id = uuid::Uuid::new_v4().to_string();
        
        // Store connection
        let mut connections = self.ws_connections.lock().unwrap();
        connections.insert(connection_id.clone(), WebSocketConnection {
            stream,
            url: url.to_string(),
            is_open: true,
        });
        
        Ok(connection_id)
    }
    
    /// Send a message to a WebSocket
    pub async fn send_websocket_message(&self, connection_id: &str, message: &str) -> Result<(), ToolError> {
        // Get connection
        let mut connections = self.ws_connections.lock().unwrap();
        let connection = connections.get_mut(connection_id)
            .ok_or_else(|| ToolError::new(404, format!("WebSocket connection not found: {}", connection_id)))?;
        
        // Check if connection is open
        if !connection.is_open {
            return Err(ToolError::new(400, "WebSocket connection is closed"));
        }
        
        // Send message
        connection.stream.send(Message::Text(message.to_string())).await
            .map_err(|e| ToolError::new(500, format!("Failed to send WebSocket message: {}", e)))?;
        
        Ok(())
    }
    
    /// Close a WebSocket connection
    pub async fn close_websocket(&self, connection_id: &str) -> Result<(), ToolError> {
        // Get connection
        let mut connections = self.ws_connections.lock().unwrap();
        let connection = connections.get_mut(connection_id)
            .ok_or_else(|| ToolError::new(404, format!("WebSocket connection not found: {}", connection_id)))?;
        
        // Check if connection is open
        if !connection.is_open {
            return Err(ToolError::new(400, "WebSocket connection is already closed"));
        }
        
        // Close connection
        connection.stream.close(None).await
            .map_err(|e| ToolError::new(500, format!("Failed to close WebSocket connection: {}", e)))?;
        
        // Mark as closed
        connection.is_open = false;
        
        Ok(())
    }
    
    /// Parse HTML content
    pub fn parse_html(&self, html: &str) -> Result<HtmlDocument, ToolError> {
        // Parse HTML
        let document = Html::parse_document(html);
        
        // Get title
        let title_selector = Selector::parse("title").unwrap();
        let title = document.select(&title_selector).next()
            .map(|element| element.text().collect::<String>());
        
        // Get body text
        let body_selector = Selector::parse("body").unwrap();
        let body_text = document.select(&body_selector).next()
            .map(|element| element.text().collect::<String>())
            .unwrap_or_default();
        
        // Get links
        let link_selector = Selector::parse("a[href]").unwrap();
        let mut links = Vec::new();
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                links.push(HtmlLink {
                    url: href.to_string(),
                    text: element.text().collect::<String>(),
                });
            }
        }
        
        // Get images
        let image_selector = Selector::parse("img[src]").unwrap();
        let mut images = Vec::new();
        for element in document.select(&image_selector) {
            if let Some(src) = element.value().attr("src") {
                images.push(HtmlImage {
                    url: src.to_string(),
                    alt: element.value().attr("alt").map(|s| s.to_string()),
                });
            }
        }
        
        Ok(HtmlDocument {
            title,
            body_text,
            links,
            images,
        })
    }
}

impl ExternalTool for WebTool {
    fn name(&self) -> &str {
        "web"
    }
    
    fn description(&self) -> &str {
        "Web tool for HTTP requests, WebSocket communication, and HTML parsing"
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn execute(&self, params: &ToolParams) -> Result<ToolResult, ToolError> {
        // Get command
        let command = params.command.as_str();
        
        // Execute command
        match command {
            "http" => {
                // Get parameters
                let method = params.get_string("method").unwrap_or_else(|| "GET".to_string());
                let url = params.get_string("url").ok_or_else(|| ToolError::new(400, "Missing URL parameter"))?;
                let headers = params.get::<HashMap<String, String>>("headers");
                let body = params.get_string("body");
                
                // Create future for async execution
                let future = self.send_request(&method, &url, headers, body);
                
                // Execute future
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::new(500, format!("Failed to create runtime: {}", e)))?;
                
                let response = runtime.block_on(future)?;
                
                // Convert response to result
                let mut result_data = HashMap::new();
                result_data.insert("status".to_string(), Value::number(response.status as f64));
                result_data.insert("headers".to_string(), Value::object(response.headers.iter().map(|(k, v)| (k.clone(), Value::string(v.clone()))).collect()));
                result_data.insert("body".to_string(), Value::string(response.body));
                
                Ok(ToolResult::success(Value::object(result_data)))
            },
            "websocket_connect" => {
                // Get parameters
                let url = params.get_string("url").ok_or_else(|| ToolError::new(400, "Missing URL parameter"))?;
                
                // Create future for async execution
                let future = self.connect_websocket(&url);
                
                // Execute future
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::new(500, format!("Failed to create runtime: {}", e)))?;
                
                let connection_id = runtime.block_on(future)?;
                
                // Return connection ID
                Ok(ToolResult::success(Value::string(connection_id)))
            },
            "websocket_send" => {
                // Get parameters
                let connection_id = params.get_string("connection_id").ok_or_else(|| ToolError::new(400, "Missing connection_id parameter"))?;
                let message = params.get_string("message").ok_or_else(|| ToolError::new(400, "Missing message parameter"))?;
                
                // Create future for async execution
                let future = self.send_websocket_message(&connection_id, &message);
                
                // Execute future
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::new(500, format!("Failed to create runtime: {}", e)))?;
                
                runtime.block_on(future)?;
                
                // Return success
                Ok(ToolResult::success(Value::boolean(true)))
            },
            "websocket_close" => {
                // Get parameters
                let connection_id = params.get_string("connection_id").ok_or_else(|| ToolError::new(400, "Missing connection_id parameter"))?;
                
                // Create future for async execution
                let future = self.close_websocket(&connection_id);
                
                // Execute future
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::new(500, format!("Failed to create runtime: {}", e)))?;
                
                runtime.block_on(future)?;
                
                // Return success
                Ok(ToolResult::success(Value::boolean(true)))
            },
            "parse_html" => {
                // Get parameters
                let html = params.get_string("html").ok_or_else(|| ToolError::new(400, "Missing HTML parameter"))?;
                
                // Parse HTML
                let document = self.parse_html(&html)?;
                
                // Convert document to result
                let mut result_data = HashMap::new();
                result_data.insert("title".to_string(), match document.title {
                    Some(title) => Value::string(title),
                    None => Value::null(),
                });
                result_data.insert("body_text".to_string(), Value::string(document.body_text));
                result_data.insert("links".to_string(), Value::array(document.links.iter().map(|link| {
                    let mut link_data = HashMap::new();
                    link_data.insert("url".to_string(), Value::string(link.url.clone()));
                    link_data.insert("text".to_string(), Value::string(link.text.clone()));
                    Value::object(link_data)
                }).collect()));
                result_data.insert("images".to_string(), Value::array(document.images.iter().map(|image| {
                    let mut image_data = HashMap::new();
                    image_data.insert("url".to_string(), Value::string(image.url.clone()));
                    image_data.insert("alt".to_string(), match &image.alt {
                        Some(alt) => Value::string(alt.clone()),
                        None => Value::null(),
                    });
                    Value::object(image_data)
                }).collect()));
                
                Ok(ToolResult::success(Value::object(result_data)))
            },
            _ => Err(ToolError::new(400, format!("Unknown command: {}", command))),
        }
    }
}
