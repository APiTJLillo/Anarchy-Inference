// src/std/mod.rs
// Standard library modules for Anarchy-Inference

pub mod fs;
pub mod shell;
pub mod http;
pub mod browser;
pub mod crypto;
pub mod mem;

// Register all standard library functions
pub fn register_stdlib() {
    // Register file system operations
    fs::register_fs_functions();
    
    // Register shell operations
    shell::register_shell_functions();
    
    // Register HTTP operations
    http::register_http_functions();
    
    // Register browser operations
    browser::register_browser_functions();
    
    // Register crypto operations
    crypto::register_crypto_functions();
    
    // Register memory operations
    mem::register_mem_functions();
}
