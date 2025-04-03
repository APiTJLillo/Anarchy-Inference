// src/std.rs
// Main entry point for the standard library

pub mod fs;
pub mod shell;
pub mod http;
pub mod browser;
pub mod security;
pub mod crypto;
pub mod mem;

use crate::value::Value;
use crate::error::LangError;
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

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

// Initialize the standard library
pub fn init_stdlib() {
    // Register file system functions
    register_token("📂", fs_list_dir_wrapper);
    register_token("d", fs_list_dir_wrapper);
    register_token("📖", fs_read_file_wrapper);
    register_token("r", fs_read_file_wrapper);
    register_token("✍", fs_write_file_wrapper);
    register_token("w", fs_write_file_wrapper);
    register_token("✂", fs_remove_path_wrapper);
    register_token("x", fs_remove_path_wrapper);
    register_token("⧉", fs_copy_file_wrapper);
    register_token("c", fs_copy_file_wrapper);
    register_token("↷", fs_move_file_wrapper);
    register_token("m", fs_move_file_wrapper);
    register_token("?", fs_file_exists_wrapper);
    register_token("e", fs_file_exists_wrapper);
    
    // Register shell functions
    register_token("!", shell_execute_wrapper);
    register_token("🖥", shell_current_os_wrapper);
    register_token("s", shell_current_os_wrapper);
    register_token("🌐", shell_get_env_var_wrapper);
    register_token("v", shell_get_env_var_wrapper);
    
    // Register HTTP functions
    register_token("↗", http_get_wrapper);
    register_token("g", http_get_wrapper);
    register_token("↓", http_post_wrapper);
    register_token("p", http_post_wrapper);
    register_token("⎋", http_json_parse_wrapper);
    register_token("j", http_json_parse_wrapper);
    register_token("~", http_websocket_open_wrapper);
    
    // Register browser functions
    register_token("🌐", browser_open_wrapper);
    register_token("b", browser_open_wrapper);
    register_token("🖱", browser_click_wrapper);
    register_token("k", browser_click_wrapper);
    register_token("⌨", browser_input_wrapper);
    register_token("i", browser_input_wrapper);
    register_token("👁", browser_get_text_wrapper);
    register_token("t", browser_get_text_wrapper);
    register_token("🧠", browser_eval_js_wrapper);
    register_token("e", browser_eval_js_wrapper);
    register_token("❌", browser_close_wrapper);
    register_token("z", browser_close_wrapper);
    
    // Register crypto functions
    register_token("#", crypto_hash_string_wrapper);
    register_token("#f", crypto_hash_file_wrapper);
    register_token("h", crypto_hash_file_wrapper);
    
    // Register memory functions
    register_token("📝", mem_set_memory_wrapper);
    register_token("m", mem_set_memory_wrapper);
    register_token("📖", mem_get_memory_wrapper);
    register_token("n", mem_get_memory_wrapper);
    register_token("🗑", mem_forget_memory_wrapper);
    register_token("f", mem_forget_memory_wrapper);
}

// Wrapper functions for file system operations
fn fs_list_dir_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("list_dir requires 1 argument"));
    }
    
    if let Value::String(path) = &args[0] {
        security::check_path_allowed(path)?;
        fs::list_dir(path)
    } else {
        Err(LangError::runtime_error("list_dir requires a string argument"))
    }
}

fn fs_read_file_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("read_file requires 1 argument"));
    }
    
    if let Value::String(path) = &args[0] {
        security::check_path_allowed(path)?;
        fs::read_file(path)
    } else {
        Err(LangError::runtime_error("read_file requires a string argument"))
    }
}

fn fs_write_file_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LangError::runtime_error("write_file requires 2 or 3 arguments"));
    }
    
    if let (Value::String(path), Value::String(contents)) = (&args[0], &args[1]) {
        security::check_path_allowed(path)?;
        
        let mode = if args.len() == 3 {
            if let Value::String(mode_str) = &args[2] {
                Some(mode_str.as_str())
            } else {
                return Err(LangError::runtime_error("write_file mode must be a string"));
            }
        } else {
            None
        };
        
        fs::write_file(path, contents, mode)
    } else {
        Err(LangError::runtime_error("write_file requires string arguments"))
    }
}

fn fs_remove_path_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("remove_path requires 1 argument"));
    }
    
    if let Value::String(path) = &args[0] {
        security::check_path_allowed(path)?;
        fs::remove_path(path)
    } else {
        Err(LangError::runtime_error("remove_path requires a string argument"))
    }
}

fn fs_copy_file_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("copy_file requires 2 arguments"));
    }
    
    if let (Value::String(src), Value::String(dst)) = (&args[0], &args[1]) {
        security::check_path_allowed(src)?;
        security::check_path_allowed(dst)?;
        fs::copy_file(src, dst)
    } else {
        Err(LangError::runtime_error("copy_file requires string arguments"))
    }
}

fn fs_move_file_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("move_file requires 2 arguments"));
    }
    
    if let (Value::String(src), Value::String(dst)) = (&args[0], &args[1]) {
        security::check_path_allowed(src)?;
        security::check_path_allowed(dst)?;
        fs::move_file(src, dst)
    } else {
        Err(LangError::runtime_error("move_file requires string arguments"))
    }
}

fn fs_file_exists_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("file_exists requires 1 argument"));
    }
    
    if let Value::String(path) = &args[0] {
        security::check_path_allowed(path)?;
        fs::file_exists(path)
    } else {
        Err(LangError::runtime_error("file_exists requires a string argument"))
    }
}

// Wrapper functions for shell operations
fn shell_execute_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("execute_shell requires 1 argument"));
    }
    
    security::check_shell_allowed()?;
    
    if let Value::String(command) = &args[0] {
        shell::execute_shell(command)
    } else {
        Err(LangError::runtime_error("execute_shell requires a string argument"))
    }
}

fn shell_current_os_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if !args.is_empty() {
        return Err(LangError::runtime_error("current_os takes no arguments"));
    }
    
    shell::current_os()
}

fn shell_get_env_var_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("get_env_var requires 1 argument"));
    }
    
    if let Value::String(name) = &args[0] {
        shell::get_env_var(name)
    } else {
        Err(LangError::runtime_error("get_env_var requires a string argument"))
    }
}

// Wrapper functions for HTTP operations
fn http_get_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("http_get requires 1 argument"));
    }
    
    security::check_network_allowed()?;
    
    if let Value::String(url) = &args[0] {
        http::http_get(url)
    } else {
        Err(LangError::runtime_error("http_get requires a string argument"))
    }
}

fn http_post_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("http_post requires 2 arguments"));
    }
    
    security::check_network_allowed()?;
    
    if let (Value::String(url), Value::String(body)) = (&args[0], &args[1]) {
        http::http_post(url, body)
    } else {
        Err(LangError::runtime_error("http_post requires string arguments"))
    }
}

fn http_json_parse_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("json_parse requires 1 argument"));
    }
    
    if let Value::String(json_str) = &args[0] {
        http::json_parse(json_str)
    } else {
        Err(LangError::runtime_error("json_parse requires a string argument"))
    }
}

fn http_websocket_open_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("websocket_open requires 1 argument"));
    }
    
    security::check_network_allowed()?;
    
    if let Value::String(url) = &args[0] {
        http::websocket_open(url)
    } else {
        Err(LangError::runtime_error("websocket_open requires a string argument"))
    }
}

// Wrapper functions for browser operations
fn browser_open_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("browser_open requires 1 argument"));
    }
    
    security::check_network_allowed()?;
    
    if let Value::String(url) = &args[0] {
        browser::browser_open(url)
    } else {
        Err(LangError::runtime_error("browser_open requires a string argument"))
    }
}

fn browser_click_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("browser_click requires 2 arguments"));
    }
    
    if let (Value::Number(browser_id), Value::String(selector)) = (&args[0], &args[1]) {
        browser::browser_click(*browser_id, selector)
    } else {
        Err(LangError::runtime_error("browser_click requires a number and a string argument"))
    }
}

fn browser_input_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 3 {
        return Err(LangError::runtime_error("browser_input requires 3 arguments"));
    }
    
    if let (Value::Number(browser_id), Value::String(selector), Value::String(text)) = (&args[0], &args[1], &args[2]) {
        browser::browser_input(*browser_id, selector, text)
    } else {
        Err(LangError::runtime_error("browser_input requires a number and two string arguments"))
    }
}

fn browser_get_text_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("browser_get_text requires 2 arguments"));
    }
    
    if let (Value::Number(browser_id), Value::String(selector)) = (&args[0], &args[1]) {
        browser::browser_get_text(*browser_id, selector)
    } else {
        Err(LangError::runtime_error("browser_get_text requires a number and a string argument"))
    }
}

fn browser_eval_js_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("browser_eval_js requires 2 arguments"));
    }
    
    if let (Value::Number(browser_id), Value::String(js_code)) = (&args[0], &args[1]) {
        browser::browser_eval_js(*browser_id, js_code)
    } else {
        Err(LangError::runtime_error("browser_eval_js requires a number and a string argument"))
    }
}

fn browser_close_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("browser_close requires 1 argument"));
    }
    
    if let Value::Number(browser_id) = &args[0] {
        browser::browser_close(*browser_id)
    } else {
        Err(LangError::runtime_error("browser_close requires a number argument"))
    }
}

// Wrapper functions for crypto operations
fn crypto_hash_string_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("hash_string requires 2 arguments"));
    }
    
    if let (Value::String(input), Value::String(algorithm)) = (&args[0], &args[1]) {
        crypto::hash_string(input, algorithm)
    } else {
        Err(LangError::runtime_error("hash_string requires string arguments"))
    }
}

fn crypto_hash_file_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("hash_file requires 2 arguments"));
    }
    
    if let (Value::String(path), Value::String(algorithm)) = (&args[0], &args[1]) {
        security::check_path_allowed(path)?;
        crypto::hash_file(path, algorithm)
    } else {
        Err(LangError::runtime_error("hash_file requires string arguments"))
    }
}

// Wrapper functions for memory operations
fn mem_set_memory_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 2 {
        return Err(LangError::runtime_error("set_memory requires 2 arguments"));
    }
    
    if let Value::String(key) = &args[0] {
        mem::set_memory(key, args[1].clone())
    } else {
        Err(LangError::runtime_error("set_memory requires a string key"))
    }
}

fn mem_get_memory_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("get_memory requires 1 argument"));
    }
    
    if let Value::String(key) = &args[0] {
        mem::get_memory(key)
    } else {
        Err(LangError::runtime_error("get_memory requires a string argument"))
    }
}

fn mem_forget_memory_wrapper(args: &[Value]) -> Result<Value, LangError> {
    if args.len() != 1 {
        return Err(LangError::runtime_error("forget_memory requires 1 argument"));
    }
    
    if let Value::String(key) = &args[0] {
        mem::forget_memory(key)
    } else {
        Err(LangError::runtime_error("forget_memory requires a string argument"))
    }
}
