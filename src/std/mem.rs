// src/std/mem.rs
// Agent Memory for Anarchy-Inference

use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::value::Value;
use crate::error::LangError;

// Global memory storage
static MEMORY: Lazy<RwLock<HashMap<String, Value>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Set memory value
/// Symbol: 📝 or m
/// Usage: m("key", "val")
pub fn set_memory(key: &str, value: Value) -> Result<Value, LangError> {
    if let Ok(mut memory) = MEMORY.write() {
        memory.insert(key.to_string(), value);
        Ok(Value::boolean(true))
    } else {
        Err(LangError::runtime_error("Failed to acquire write lock for memory"))
    }
}

/// Get memory value
/// Symbol: 📖 or n
/// Usage: n("key") → "val"
pub fn get_memory(key: &str) -> Result<Value, LangError> {
    if let Ok(memory) = MEMORY.read() {
        if let Some(value) = memory.get(key) {
            Ok(value.clone())
        } else {
            Ok(Value::null())
        }
    } else {
        Err(LangError::runtime_error("Failed to acquire read lock for memory"))
    }
}

/// Forget memory key
/// Symbol: 🗑 or f
/// Usage: f("key")
pub fn forget_memory(key: &str) -> Result<Value, LangError> {
    if let Ok(mut memory) = MEMORY.write() {
        memory.remove(key);
        Ok(Value::boolean(true))
    } else {
        Err(LangError::runtime_error("Failed to acquire write lock for memory"))
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
