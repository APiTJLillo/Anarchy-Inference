// src/std/security.rs
// Security Gate for Anarchy-Inference

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use std::collections::HashSet;
use std::path::Path;
use once_cell::sync::Lazy;
use crate::error::LangError;

// Security configuration flags
static ALLOW_FS: AtomicBool = AtomicBool::new(false);
static ALLOW_SHELL: AtomicBool = AtomicBool::new(false);
static ALLOW_NETWORK: AtomicBool = AtomicBool::new(false);

// Allowed paths for file system operations
static ALLOWED_PATHS: Lazy<RwLock<HashSet<String>>> = Lazy::new(|| RwLock::new(HashSet::new()));

/// Set file system access permission
/// Symbol: 🔓_fs
/// Usage: Set true/false before interpreter start
pub fn set_allow_fs(allow: bool) {
    ALLOW_FS.store(allow, Ordering::SeqCst);
}

/// Set shell access permission
/// Symbol: 🔓_sh
/// Usage: Enable/disable !() shell command
pub fn set_allow_shell(allow: bool) {
    ALLOW_SHELL.store(allow, Ordering::SeqCst);
}

/// Set network access permission
/// Symbol: 🔓_net
/// Usage: Enable/disable g(), p(), ~()
pub fn set_allow_network(allow: bool) {
    ALLOW_NETWORK.store(allow, Ordering::SeqCst);
}

/// Add allowed path for file system operations
/// Symbol: 📁_allow
/// Usage: Add path to allowed paths list
pub fn add_allowed_path(path: &str) {
    if let Ok(mut paths) = ALLOWED_PATHS.write() {
        paths.insert(path.to_string());
    }
}

/// Clear allowed paths
pub fn clear_allowed_paths() {
    if let Ok(mut paths) = ALLOWED_PATHS.write() {
        paths.clear();
    }
}

/// Check if file system operations are allowed
pub fn check_fs_allowed() -> Result<(), LangError> {
    if !ALLOW_FS.load(Ordering::SeqCst) {
        return Err(LangError::runtime_error("File system operations are not allowed"));
    }
    Ok(())
}

/// Check if shell operations are allowed
pub fn check_shell_allowed() -> Result<(), LangError> {
    if !ALLOW_SHELL.load(Ordering::SeqCst) {
        return Err(LangError::runtime_error("Shell operations are not allowed"));
    }
    Ok(())
}

/// Check if network operations are allowed
pub fn check_network_allowed() -> Result<(), LangError> {
    if !ALLOW_NETWORK.load(Ordering::SeqCst) {
        return Err(LangError::runtime_error("Network operations are not allowed"));
    }
    Ok(())
}

/// Check if path is allowed for file system operations
pub fn check_path_allowed(path: &str) -> Result<(), LangError> {
    // First check if file system operations are allowed at all
    check_fs_allowed()?;
    
    // If no paths are explicitly allowed, all paths are allowed
    if let Ok(paths) = ALLOWED_PATHS.read() {
        if paths.is_empty() {
            return Ok(());
        }
        
        // Check if the path or any of its parent directories are in the allowed paths
        let path_obj = Path::new(path);
        let path_str = path_obj.to_string_lossy().to_string();
        
        // Check if the path itself is allowed
        if paths.contains(&path_str) {
            return Ok(());
        }
        
        // Check if any parent directory is allowed
        for allowed_path in paths.iter() {
            if path_str.starts_with(allowed_path) {
                return Ok(());
            }
        }
        
        return Err(LangError::runtime_error(&format!("Path '{}' is not in the allowed paths", path)));
    }
    
    Ok(())
}

/// Register all security functions
pub fn register_security_functions() {
    // This function will be called from the main module to register all security functions
    // Implementation will be added when the token registration system is implemented
}
