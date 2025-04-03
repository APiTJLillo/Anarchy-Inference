// src/std/fs.rs
// File system operations for Anarchy-Inference

use std::fs;
use std::path::Path;
use std::io::{self, Read, Write};
use crate::value::Value;
use crate::error::LangError;

/// List directory contents
/// Symbol: 📂 or d
/// Usage: d("path") → [files...]
pub fn list_dir(path: &str) -> Result<Value, LangError> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to read directory '{}': {}", path, e))),
    };

    let mut files = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                if let Some(file_name) = entry.file_name().to_str() {
                    files.push(Value::string(file_name.to_string()));
                }
            },
            Err(e) => return Err(LangError::runtime_error(&format!("Error reading directory entry: {}", e))),
        }
    }

    Ok(Value::array(files))
}

/// Read file contents
/// Symbol: 📖 or r
/// Usage: r("file") → "contents"
pub fn read_file(path: &str) -> Result<Value, LangError> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to open file '{}': {}", path, e))),
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        return Err(LangError::runtime_error(&format!("Failed to read file '{}': {}", path, e)));
    }

    Ok(Value::string(contents))
}

/// Write file contents
/// Symbol: ✍ or w
/// Usage: w("file", "contents", [mode]) where mode is optional
pub fn write_file(path: &str, contents: &str, mode: Option<&str>) -> Result<Value, LangError> {
    let result = if let Some("a") = mode {
        // Append mode
        let mut file = match fs::OpenOptions::new().append(true).create(true).open(path) {
            Ok(file) => file,
            Err(e) => return Err(LangError::runtime_error(&format!("Failed to open file '{}' for appending: {}", path, e))),
        };

        match file.write_all(contents.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(LangError::runtime_error(&format!("Failed to append to file '{}': {}", path, e))),
        }
    } else {
        // Write mode (default)
        match fs::write(path, contents) {
            Ok(_) => Ok(()),
            Err(e) => Err(LangError::runtime_error(&format!("Failed to write to file '{}': {}", path, e))),
        }
    };

    match result {
        Ok(_) => Ok(Value::boolean(true)), // Return ✓ on success
        Err(e) => Err(e),
    }
}

/// Remove file or directory
/// Symbol: ✂ or x
/// Usage: x("path")
pub fn remove_path(path: &str) -> Result<Value, LangError> {
    let path_obj = Path::new(path);
    let result = if path_obj.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    match result {
        Ok(_) => Ok(Value::boolean(true)), // Return ✓ on success
        Err(e) => Err(LangError::runtime_error(&format!("Failed to remove '{}': {}", path, e))),
    }
}

/// Copy file
/// Symbol: ⧉ or c
/// Usage: c("src", "dst")
pub fn copy_file(src: &str, dst: &str) -> Result<Value, LangError> {
    match fs::copy(src, dst) {
        Ok(_) => Ok(Value::boolean(true)), // Return ✓ on success
        Err(e) => Err(LangError::runtime_error(&format!("Failed to copy '{}' to '{}': {}", src, dst, e))),
    }
}

/// Move file
/// Symbol: ↷ or m
/// Usage: m("src", "dst")
pub fn move_file(src: &str, dst: &str) -> Result<Value, LangError> {
    match fs::rename(src, dst) {
        Ok(_) => Ok(Value::boolean(true)), // Return ✓ on success
        Err(e) => Err(LangError::runtime_error(&format!("Failed to move '{}' to '{}': {}", src, dst, e))),
    }
}

/// Check if file exists
/// Symbol: ? or e
/// Usage: e("path") → bool
pub fn file_exists(path: &str) -> Result<Value, LangError> {
    Ok(Value::boolean(Path::new(path).exists()))
}

/// Register all file system functions
pub fn register_fs_functions() {
    // This function will be called from the main module to register all file system functions
    // Implementation will be added when the token registration system is implemented
    // Example:
    // reg("📂", list_dir);
    // reg("d", list_dir);
    // reg("📖", read_file);
    // reg("r", read_file);
    // etc.
}
