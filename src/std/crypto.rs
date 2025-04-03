// src/std/crypto.rs
// Crypto Primitives for Anarchy-Inference

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use md5::Digest as Md5Digest;
use crate::value::Value;
use crate::error::LangError;
// Import security module from parent directory
use crate::security::check_path_allowed;

/// Hash a string
/// Symbol: #
/// Usage: #("abc", "sha256") → "..."
pub fn hash_string(input: &str, algorithm: &str) -> Result<Value, LangError> {
    match algorithm.to_lowercase().as_str() {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            let result = hasher.finalize();
            Ok(Value::string(format!("{:x}", result)))
        },
        "md5" => {
            let mut hasher = md5::Context::new();
            hasher.consume(input.as_bytes());
            let result = hasher.compute();
            Ok(Value::string(format!("{:x}", result)))
        },
        _ => Err(LangError::runtime_error(&format!("Unsupported hash algorithm: {}", algorithm))),
    }
}

/// Hash a file
/// Symbol: #f or h
/// Usage: h("file", "sha1") → "..."
pub fn hash_file(path: &str, algorithm: &str) -> Result<Value, LangError> {
    // Check if file system operations are allowed
    check_path_allowed(path)?;
    
    // Open the file
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to open file '{}': {}", path, e))),
    };
    
    // Read the file contents
    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        return Err(LangError::runtime_error(&format!("Failed to read file '{}': {}", path, e)));
    }
    
    // Hash the file contents
    match algorithm.to_lowercase().as_str() {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            let result = hasher.finalize();
            Ok(Value::string(format!("{:x}", result)))
        },
        "md5" => {
            let mut hasher = md5::Context::new();
            hasher.consume(&buffer);
            let result = hasher.compute();
            Ok(Value::string(format!("{:x}", result)))
        },
        _ => Err(LangError::runtime_error(&format!("Unsupported hash algorithm: {}", algorithm))),
    }
}

/// Register all crypto functions
pub fn register_crypto_functions() {
    // This function will be called from the main module to register all crypto functions
    // Implementation will be added when the token registration system is implemented
    // Example:
    // reg("#", hash_string);
    // reg("#f", hash_file);
    // reg("h", hash_file);
}
