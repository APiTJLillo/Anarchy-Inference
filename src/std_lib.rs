// src/std_lib.rs
// Main entry point for the standard library

// Import directly from the std directory modules
use crate::value::Value;
use crate::error::LangError;
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::std::security;
use crate::std::fs;
use crate::std::shell;
use crate::std::http;
use crate::std::browser;
use crate::std::crypto;
use crate::std::mem;

// Token registration table
type TokenFn = fn(&[Value]) -> Result<Value, LangError>;
static TOKEN_TABLE: Lazy<RwLock<HashMap<String, TokenFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

// Register a token function
pub fn register_token(token: &str, func: TokenFn) {
    if let Ok(mut table) = TOKEN_TABLE.write() {
        table.insert(token.to_string(), func);
    }
}

// Look up a token function
pub fn lookup_token(token: &str) -> Option<TokenFn> {
    if let Ok(table) = TOKEN_TABLE.read() {
        table.get(token).copied()
    } else {
        None
    }
}
