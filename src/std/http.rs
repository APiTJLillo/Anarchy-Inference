// src/std/http.rs
// HTTP & Networking for Anarchy-Inference

use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use serde_json::{Value as JsonValue, from_str as json_from_str};
use crate::value::Value;
use crate::error::LangError;

/// Perform HTTP GET request
/// Symbol: ↗ or g
/// Usage: g("https://site") → {s:status, b:body}
pub fn http_get(url: &str) -> Result<Value, LangError> {
    let client = Client::new();
    let response = match client.get(url).timeout(Duration::from_secs(30)).send() {
        Ok(response) => response,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to perform GET request to '{}': {}", url, e))),
    };

    create_response_object(response)
}

/// Perform HTTP POST request
/// Symbol: ↓ or p
/// Usage: p("url", "body") → {s:status, b:body}
pub fn http_post(url: &str, body: &str) -> Result<Value, LangError> {
    let client = Client::new();
    let response = match client.post(url).body(body.to_string()).timeout(Duration::from_secs(30)).send() {
        Ok(response) => response,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to perform POST request to '{}': {}", url, e))),
    };

    create_response_object(response)
}

/// Parse JSON string
/// Symbol: ⎋ or j
/// Usage: j("{...}") → {key: val}
pub fn json_parse(json_str: &str) -> Result<Value, LangError> {
    match json_from_str::<JsonValue>(json_str) {
        Ok(json_value) => json_to_value(json_value),
        Err(e) => Err(LangError::runtime_error(&format!("Failed to parse JSON: {}", e))),
    }
}

/// Open WebSocket connection
/// Symbol: ~
/// Usage: ~("ws://...") → socket handle
pub fn websocket_open(_url: &str) -> Result<Value, LangError> {
    // This is a placeholder for WebSocket implementation
    // WebSocket implementation requires more complex async handling
    // For now, return an error indicating it's not implemented yet
    Err(LangError::runtime_error("WebSocket support not implemented yet"))
}

// Helper function to create a response object from an HTTP response
fn create_response_object(response: Response) -> Result<Value, LangError> {
    let status = response.status().as_u16() as f64;
    let body = match response.text() {
        Ok(text) => text,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to read response body: {}", e))),
    };

    let mut result = Value::empty_object();
    result.set_property("s".to_string(), Value::number(status))?;
    result.set_property("b".to_string(), Value::string(body))?;

    Ok(result)
}

// Helper function to convert a JSON value to a language Value
fn json_to_value(json_value: JsonValue) -> Result<Value, LangError> {
    match json_value {
        JsonValue::Null => Ok(Value::null()),
        JsonValue::Bool(b) => Ok(Value::boolean(b)),
        JsonValue::Number(n) => {
            if let Some(f) = n.as_f64() {
                Ok(Value::number(f))
            } else {
                Err(LangError::runtime_error("Failed to convert JSON number"))
            }
        },
        JsonValue::String(s) => Ok(Value::string(s)),
        JsonValue::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                values.push(json_to_value(item)?);
            }
            Ok(Value::array(values))
        },
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (key, value) in obj {
                map.insert(key, json_to_value(value)?);
            }
            Ok(Value::object(map))
        },
    }
}

/// Register all HTTP functions
pub fn register_http_functions() {
    // This function will be called from the main module to register all HTTP functions
    // Implementation will be added when the token registration system is implemented
    // Example:
    // reg("↗", http_get);
    // reg("g", http_get);
    // reg("↓", http_post);
    // reg("p", http_post);
    // reg("⎋", json_parse);
    // reg("j", json_parse);
    // reg("~", websocket_open);
}
