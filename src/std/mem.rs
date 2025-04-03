// src/std/mem.rs
// Agent Memory for Anarchy-Inference

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::value::Value;
use crate::error::LangError;

// Global memory storage using thread-safe types
// Using Arc<Mutex<>> instead of RwLock for thread-safety with Value type
static MEMORY: Lazy<Arc<Mutex<HashMap<String, String>>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Set memory value
/// Symbol: 📝 or m
/// Usage: m("key", "val")
pub fn set_memory(key: &str, value: Value) -> Result<Value, LangError> {
    // Convert Value to String for storage
    let value_str = format!("{}", value);
    
    if let Ok(mut memory) = MEMORY.lock() {
        memory.insert(key.to_string(), value_str);
        Ok(Value::boolean(true))
    } else {
        Err(LangError::runtime_error("Failed to acquire lock for memory"))
    }
}

/// Get memory value
/// Symbol: 📖 or n
/// Usage: n("key") → "val"
pub fn get_memory(key: &str) -> Result<Value, LangError> {
    if let Ok(memory) = MEMORY.lock() {
        if let Some(value_str) = memory.get(key) {
            // Return the string value directly
            Ok(Value::string(value_str.clone()))
        } else {
            Ok(Value::null())
        }
    } else {
        Err(LangError::runtime_error("Failed to acquire lock for memory"))
    }
}

/// Forget memory key
/// Symbol: 🗑 or f
/// Usage: f("key")
pub fn forget_memory(key: &str) -> Result<Value, LangError> {
    if let Ok(mut memory) = MEMORY.lock() {
        memory.remove(key);
        Ok(Value::boolean(true))
    } else {
        Err(LangError::runtime_error("Failed to acquire lock for memory"))
    }
}

/// Register all memory functions
pub fn register_mem_functions() {
    // This function will be called from the main module to register all memory functions
    // Implementation will be added when the token registration system is implemented
    // Example:
    // reg("📝", set_memory);
    // reg("m", set_memory);
    // reg("📖", get_memory);
    // reg("n", get_memory);
    // reg("🗑", forget_memory);
    // reg("f", forget_memory);
}
