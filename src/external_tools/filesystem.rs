// src/external_tools/filesystem.rs - File system interface for external tools

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::value::Value;
use super::common::{ExternalTool, ToolParams, ToolResult, ToolStatus, ToolError, ToolContext};

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File name
    pub name: String,
    
    /// File path
    pub path: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// Is directory
    pub is_dir: bool,
    
    /// Last modified time
    pub modified: DateTime<Utc>,
    
    /// Last accessed time
    pub accessed: Option<DateTime<Utc>>,
    
    /// Created time
    pub created: Option<DateTime<Utc>>,
}

/// File system operations
#[derive(Debug, Clone)]
pub struct FileSystemOperations {
    /// Allow read operations
    pub allow_read: bool,
    
    /// Allow write operations
    pub allow_write: bool,
    
    /// Allow delete operations
    pub allow_delete: bool,
    
    /// Allow directory operations
    pub allow_dir_ops: bool,
    
    /// Allow operations outside base directory
    pub allow_outside_base: bool,
}

impl Default for FileSystemOperations {
    fn default() -> Self {
        Self {
            allow_read: true,
            allow_write: true,
            allow_delete: true,
            allow_dir_ops: true,
            allow_outside_base: false,
        }
    }
}

/// Security sandbox for file system operations
pub struct SecuritySandbox {
    /// Allowed file extensions for read
    pub allowed_read_extensions: Option<Vec<String>>,
    
    /// Allowed file extensions for write
    pub allowed_write_extensions: Option<Vec<String>>,
    
    /// Maximum file size for read (in bytes)
    pub max_read_size: Option<u64>,
    
    /// Maximum file size for write (in bytes)
    pub max_write_size: Option<u64>,
    
    /// Disallowed paths (glob patterns)
    pub disallowed_paths: Vec<String>,
}

impl Default for SecuritySandbox {
    fn default() -> Self {
        Self {
            allowed_read_extensions: None,
            allowed_write_extensions: None,
            max_read_size: Some(10 * 1024 * 1024), // 10 MB
            max_write_size: Some(5 * 1024 * 1024), // 5 MB
            disallowed_paths: vec![
                "**/.*".to_string(),           // Hidden files
                "**/node_modules/**".to_string(), // Node modules
                "**/target/**".to_string(),    // Rust build directory
                "**/bin/**".to_string(),       // Binary directories
                "**/tmp/**".to_string(),       // Temporary directories
            ],
        }
    }
}

/// File system tool for file and directory operations
pub struct FileSystemTool {
    /// Base directory for relative paths
    base_dir: PathBuf,
    
    /// Allowed operations
    allowed_operations: FileSystemOperations,
    
    /// Security sandbox
    security_sandbox: SecuritySandbox,
}

impl FileSystemTool {
    /// Create a new file system tool
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            allowed_operations: FileSystemOperations::default(),
            security_sandbox: SecuritySandbox::default(),
        }
    }
    
    /// Set allowed operations
    pub fn with_operations(mut self, operations: FileSystemOperations) -> Self {
        self.allowed_operations = operations;
        self
    }
    
    /// Set security sandbox
    pub fn with_sandbox(mut self, sandbox: SecuritySandbox) -> Self {
        self.security_sandbox = sandbox;
        self
    }
    
    /// Resolve a path relative to the base directory
    fn resolve_path(&self, path: &str) -> Result<PathBuf, ToolError> {
        let path = Path::new(path);
        
        // If path is absolute, check if it's allowed
        if path.is_absolute() {
            if !self.allowed_operations.allow_outside_base {
                return Err(ToolError::new(403, "Absolute paths are not allowed"));
            }
            
            // Check if path is within base directory
            match path.strip_prefix(&self.base_dir) {
                Ok(_) => Ok(path.to_path_buf()),
                Err(_) => {
                    if self.allowed_operations.allow_outside_base {
                        Ok(path.to_path_buf())
                    } else {
                        Err(ToolError::new(403, "Path is outside base directory"))
                    }
                }
            }
        } else {
            // Resolve relative path
            let resolved = self.base_dir.join(path);
            
            // Check if resolved path is within base directory
            match resolved.strip_prefix(&self.base_dir) {
                Ok(_) => Ok(resolved),
                Err(_) => Err(ToolError::new(403, "Path resolves outside base directory")),
            }
        }
    }
    
    /// Check if a path is allowed by the security sandbox
    fn check_path_allowed(&self, path: &Path) -> Result<(), ToolError> {
        // Check disallowed paths
        for pattern in &self.security_sandbox.disallowed_paths {
            if glob::Pattern::new(pattern).unwrap().matches_path(path) {
                return Err(ToolError::new(403, format!("Path matches disallowed pattern: {}", pattern)));
            }
        }
        
        Ok(())
    }
    
    /// Check if a file extension is allowed for reading
    fn check_read_extension(&self, path: &Path) -> Result<(), ToolError> {
        if let Some(allowed_extensions) = &self.security_sandbox.allowed_read_extensions {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !allowed_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    return Err(ToolError::new(403, format!("File extension not allowed for reading: {}", ext_str)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a file extension is allowed for writing
    fn check_write_extension(&self, path: &Path) -> Result<(), ToolError> {
        if let Some(allowed_extensions) = &self.security_sandbox.allowed_write_extensions {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !allowed_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    return Err(ToolError::new(403, format!("File extension not allowed for writing: {}", ext_str)));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get file information
    fn get_file_info_internal(&self, path: &Path) -> Result<FileInfo, ToolError> {
        // Get metadata
        let metadata = fs::metadata(path)
            .map_err(|e| ToolError::new(500, format!("Failed to get file metadata: {}", e)))?;
        
        // Get file name
        let name = path.file_name()
            .ok_or_else(|| ToolError::new(400, "Invalid file path"))?
            .to_string_lossy()
            .to_string();
        
        // Get file path
        let path_str = path.to_string_lossy().to_string();
        
        // Get file size
        let size = metadata.len();
        
        // Check if directory
        let is_dir = metadata.is_dir();
        
        // Get modified time
        let modified = metadata.modified()
            .map_err(|e| ToolError::new(500, format!("Failed to get modified time: {}", e)))?;
        let modified = DateTime::<Utc>::from(modified);
        
        // Get accessed time
        let accessed = metadata.accessed()
            .map(|t| DateTime::<Utc>::from(t))
            .ok();
        
        // Get created time
        let created = metadata.created()
            .map(|t| DateTime::<Utc>::from(t))
            .ok();
        
        Ok(FileInfo {
            name,
            path: path_str,
            size,
            is_dir,
            modified,
            accessed,
            created,
        })
    }
    
    /// Read a file
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, ToolError> {
        // Check if read operations are allowed
        if !self.allowed_operations.allow_read {
            return Err(ToolError::new(403, "Read operations are not allowed"));
        }
        
        // Resolve path
        let resolved_path = self.resolve_path(path)?;
        
        // Check if path is allowed
        self.check_path_allowed(&resolved_path)?;
        
        // Check if extension is allowed
        self.check_read_extension(&resolved_path)?;
        
        // Check if file exists
        if !resolved_path.exists() {
            return Err(ToolError::new(404, "File not found"));
        }
        
        // Check if file is a directory
        if resolved_path.is_dir() {
            return Err(ToolError::new(400, "Path is a directory, not a file"));
        }
        
        // Check file size
        if let Some(max_size) = self.security_sandbox.max_read_size {
            let metadata = fs::metadata(&resolved_path)
                .map_err(|e| ToolError::new(500, format!("Failed to get file metadata: {}", e)))?;
            
            if metadata.len() > max_size {
                return Err(ToolError::new(413, format!("File size exceeds maximum allowed size of {} bytes", max_size)));
            }
        }
        
        // Read file
        let mut file = File::open(&resolved_path)
            .map_err(|e| ToolError::new(500, format!("Failed to open file: {}", e)))?;
        
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| ToolError::new(500, format!("Failed to read file: {}", e)))?;
        
        Ok(content)
    }
    
    /// Write to a file
    pub fn write_file(&self, path: &str, content: &[u8], append: bool) -> Result<(), ToolError> {
        // Check if write operations are allowed
        if !self.allowed_operations.allow_write {
            return Err(ToolError::new(403, "Write operations are not allowed"));
        }
        
        // Resolve path
        let resolved_path = self.resolve_path(path)?;
        
        // Check if path is allowed
        self.check_path_allowed(&resolved_path)?;
        
        // Check if extension is allowed
        self.check_write_extension(&resolved_path)?;
        
        // Check content size
        if let Some(max_size) = self.security_sandbox.max_write_size {
            if content.len() as u64 > max_size {
                return Err(ToolError::new(413, format!("Content size exceeds maximum allowed size of {} bytes", max_size)));
            }
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = resolved_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ToolError::new(500, format!("Failed to create parent directories: {}", e)))?;
        }
        
        // Open file
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(&resolved_path)
            .map_err(|e| ToolError::new(500, format!("Failed to open file for writing: {}", e)))?;
        
        // Write content
        file.write_all(content)
            .map_err(|e| ToolError::new(500, format!("Failed to write to file: {}", e)))?;
        
        Ok(())
    }
    
    /// Delete a file
    pub fn delete_file(&self, path: &str) -> Result<(), ToolError> {
        // Check if delete operations are allowed
        if !self.allowed_operations.allow_delete {
            return Err(ToolError::new(403, "Delete operations are not allowed"));
        }
        
        // Resolve path
        let resolved_path = self.resolve_path(path)?;
        
        // Check if path is allowed
        self.check_path_allowed(&resolved_path)?;
        
        // Check if file exists
        if !resolved_path.exists() {
            return Err(ToolError::new(404, "File not found"));
        }
        
        // Check if file is a directory
        if resolved_path.is_dir() {
            return Err(ToolError::new(400, "Path is a directory, not a file"));
        }
        
        // Delete file
        fs::remove_file(&resolved_path)
            .map_err(|e| ToolError::new(500, format!("Failed to delete file: {}", e)))?;
        
        Ok(())
    }
    
    /// Create a directory
    pub fn create_dir(&self, path: &str, recursive: bool) -> Result<(), ToolError> {
        // Check if directory operations are allowed
        if !self.allowed_operations.allow_dir_ops {
            return Err(ToolError::new(403, "Directory operations are not allowed"));
        }
        
        // Resolve path
        let resolved_path = self.resolve_path(path)?;
        
        // Check if path is allowed
        self.check_path_allowed(&resolved_path)?;
        
        // Create directory
        if recursive {
            fs::create_dir_all(&resolved_path)
                .map_err(|e| ToolError::new(500, format!("Failed to create directory: {}", e)))?;
        } else {
            fs::create_dir(&resolved_path)
                .map_err(|e| ToolError::new(500, format!("Failed to create directory: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// List directory contents
    pub fn list_dir(&self, path: &str) -> Result<Vec<FileInfo>, ToolError> {
        // Check if read operations are allowed
        if !self.allowed_operations.allow_read {
            return Err(ToolError::new(403, "Read operations are not allowed"));
        }
        
        // Resolve path
        let resolved_path = self.resolve_path(path)?;
        
        // Check if path is allowed
        self.check_path_allowed(&resolved_path)?;
        
        // Check if directory exists
        if !resolved_path.exists() {
            return Err(ToolError::new(404, "Directory not found"));
        }
        
        // Check if path is a directory
        if !resolved_path.is_dir() {
            return Err(ToolError::new(400, "Path is a file, not a directory"));
        }
        
        // Read directory
        let entries = fs::read_dir(&resolved_path)
            .map_err(|e| ToolError::new(500, format!("Failed to read directory: {}", e)))?;
        
        // Get file info for each entry
        let mut file_infos = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ToolError::new(500, format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            // Skip entries that match disallowed patterns
            if self.check_path_allowed(&path).is_err() {
                continue;
            }
            
            // Get file info
            match self.get_file_info_internal(&path) {
                Ok(info) => file_infos.push(info),
                Err(_) => continue, // Skip entries that can't be read
            }
        }
        
        Ok(file_infos)
    }
    
    /// Get file information
    pub fn get_file_info(&self, path: &str) -> Result<FileInfo, ToolError> {
        // Check if read operations are allowed
        if !self.allowed_operations.allow_read {
            return Err(ToolError::new(403, "Read operations are not allowed"));
        }
        
        // Resolve path
        let resolved_path = self.resolve_path(path)?;
        
        // Check if path is allowed
        self.check_path_allowed(&resolved_path)?;
        
        // Check if file exists
        if !resolved_path.exists() {
            return Err(ToolError::new(404, "File not found"));
        }
        
        // Get file info
        self.get_file_info_internal(&resolved_path)
    }
    
    /// Copy a file
    pub fn copy_file(&self, src: &str, dst: &str, overwrite: bool) -> Result<(), ToolError> {
        // Check if read and write operations are allowed
        if !self.allowed_operations.allow_read || !self.allowed_operations.allow_write {
            return Err(ToolError::new(403, "Read and write operations are required for copy"));
        }
        
        // Resolve paths
        let src_path = self.resolve_path(src)?;
        let dst_path = self.resolve_path(dst)?;
        
        // Check if paths are allowed
        self.check_path_allowed(&src_path)?;
        self.check_path_allowed(&dst_path)?;
        
        // Check if source file exists
        if !src_path.exists() {
            return Err(ToolError::new(404, "Source file not found"));
        }
        
        // Check if source is a directory
        if src_path.is_dir() {
            return Err(ToolError::new(400, "Source path is a directory, not a file"));
        }
        
        // Check if destination exists and overwrite is not allowed
        if dst_path.exists() && !overwrite {
            return Err(ToolError::new(409, "Destination file already exists and overwrite is not allowed"));
        }
        
        // Check if extension is allowed for reading
        self.check_read_extension(&src_path)?;
        
        // Check if extension is allowed for writing
        self.check_write_extension(&dst_path)?;
        
        // Check file size
        if let Some(max_size) = self.security_sandbox.max_read_size {
            let metadata = fs::metadata(&src_path)
                .map_err(|e| ToolError::new(500, format!("Failed to get file metadata: {}", e)))?;
            
            if metadata.len() > max_size {
                return Err(ToolError::new(413, format!("File size exceeds maximum allowed size of {} bytes", max_size)));
            }
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ToolError::new(500, format!("Failed to create parent directories: {}", e)))?;
        }
        
        // Copy file
        fs::copy(&src_path, &dst_path)
            .map_err(|e| ToolError::new(500, format!("Failed to copy file: {}", e)))?;
        
        Ok(())
    }
    
    /// Move a file
    pub fn move_file(&self, src: &str, dst: &str, overwrite: bool) -> Result<(), ToolError> {
        // Check if read, write, and delete operations are allowed
        if !self.allowed_operations.allow_read || !self.allowed_operations.allow_write || !self.allowed_operations.allow_delete {
            return Err(ToolError::new(403, "Read, write, and delete operations are required for move"));
        }
        
        // Resolve paths
        let src_path = self.resolve_path(src)?;
        let dst_path = self.resolve_path(dst)?;
        
        // Check if paths are allowed
        self.check_path_allowed(&src_path)?;
        self.check_path_allowed(&dst_path)?;
        
        // Check if source file exists
        if !src_path.exists() {
            return Err(ToolError::new(404, "Source file not found"));
        }
        
        // Check if source is a directory
        if src_path.is_dir() {
            return Err(ToolError::new(400, "Source path is a directory, not a file"));
        }
        
        // Check if destination exists and overwrite is not allowed
        if dst_path.exists() && !overwrite {
            return Err(ToolError::new(409, "Destination file already exists and overwrite is not allowed"));
        }
        
        // Check if extension is allowed for reading
        self.check_read_extension(&src_path)?;
        
        // Check if extension is allowed for writing
        self.check_write_extension(&dst_path)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ToolError::new(500, format!("Failed to create parent directories: {}", e)))?;
        }
        
        // Move file
        fs::rename(&src_path, &dst_path)
            .map_err(|e| {
                // If rename fails (e.g., across filesystems), try copy and delete
                if e.kind() == io::ErrorKind::CrossesDevices {
                    match fs::copy(&src_path, &dst_path) {
                        Ok(_) => {
                            if let Err(e) = fs::remove_file(&src_path) {
                                return ToolError::new(500, format!("Failed to delete source file after copy: {}", e));
                            }
                            return ToolError::new(0, ""); // Success
                        }
                        Err(e) => return ToolError::new(500, format!("Failed to copy file during move: {}", e)),
                    }
                }
                ToolError::new(500, format!("Failed to move file: {}", e))
            })?;
        
        Ok(())
    }
}

impl ExternalTool for FileSystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }
    
    fn description(&self) -> &str {
        "File system tool for file and directory operations"
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn execute(&self, params: &ToolParams) -> Result<ToolResult, ToolError> {
        // Get command
        let command = params.command.as_str();
        
        // Execute command
        match command {
            "read" => {
                // Get parameters
                let path = params.get_string("path").ok_or_else(|| ToolError::new(400, "Missing path parameter"))?;
                
                // Read file
                let content = self.read_file(&path)?;
                
                // Try to convert to string if possible
                let result_value = match String::from_utf8(content.clone()) {
                    Ok(text) => Value::string(text),
                    Err(_) => {
                        // If not valid UTF-8, return as binary data
                        let mut data = HashMap::new();
                        data.insert("binary".to_string(), Value::boolean(true));
                        data.insert("size".to_string(), Value::number(content.len() as f64));
                        Value::object(data)
                    }
                };
                
                Ok(ToolResult::success(result_value))
            },
            "write" => {
                // Get parameters
                let path = params.get_string("path").ok_or_else(|| ToolError::new(400, "Missing path parameter"))?;
                let content = params.get_string("content").ok_or_else(|| ToolError::new(400, "Missing content parameter"))?;
                let append = params.get::<bool>("append").unwrap_or(false);
                
                // Write file
                self.write_file(&path, content.as_bytes(), append)?;
                
                Ok(ToolResult::success(Value::boolean(true)))
            },
            "delete" => {
                // Get parameters
                let path = params.get_string("path").ok_or_else(|| ToolError::new(400, "Missing path parameter"))?;
                
                // Delete file
                self.delete_file(&path)?;
                
                Ok(ToolResult::success(Value::boolean(true)))
            },
            "mkdir" => {
                // Get parameters
                let path = params.get_string("path").ok_or_else(|| ToolError::new(400, "Missing path parameter"))?;
                let recursive = params.get::<bool>("recursive").unwrap_or(false);
                
                // Create directory
                self.create_dir(&path, recursive)?;
                
                Ok(ToolResult::success(Value::boolean(true)))
            },
            "list" => {
                // Get parameters
                let path = params.get_string("path").ok_or_else(|| ToolError::new(400, "Missing path parameter"))?;
                
                // List directory
                let file_infos = self.list_dir(&path)?;
                
                // Convert to result
                let result_items = file_infos.iter().map(|info| {
                    let mut item = HashMap::new();
                    item.insert("name".to_string(), Value::string(info.name.clone()));
                    item.insert("path".to_string(), Value::string(info.path.clone()));
                    item.insert("size".to_string(), Value::number(info.size as f64));
                    item.insert("is_dir".to_string(), Value::boolean(info.is_dir));
                    item.insert("modified".to_string(), Value::string(info.modified.to_rfc3339()));
                    
                    if let Some(accessed) = &info.accessed {
                        item.insert("accessed".to_string(), Value::string(accessed.to_rfc3339()));
                    }
                    
                    if let Some(created) = &info.created {
                        item.insert("created".to_string(), Value::string(created.to_rfc3339()));
                    }
                    
                    Value::object(item)
                }).collect::<Vec<Value>>();
                
                Ok(ToolResult::success(Value::array(result_items)))
            },
            "info" => {
                // Get parameters
                let path = params.get_string("path").ok_or_else(|| ToolError::new(400, "Missing path parameter"))?;
                
                // Get file info
                let info = self.get_file_info(&path)?;
                
                // Convert to result
                let mut result_data = HashMap::new();
                result_data.insert("name".to_string(), Value::string(info.name));
                result_data.insert("path".to_string(), Value::string(info.path));
                result_data.insert("size".to_string(), Value::number(info.size as f64));
                result_data.insert("is_dir".to_string(), Value::boolean(info.is_dir));
                result_data.insert("modified".to_string(), Value::string(info.modified.to_rfc3339()));
                
                if let Some(accessed) = info.accessed {
                    result_data.insert("accessed".to_string(), Value::string(accessed.to_rfc3339()));
                }
                
                if let Some(created) = info.created {
                    result_data.insert("created".to_string(), Value::string(created.to_rfc3339()));
                }
                
                Ok(ToolResult::success(Value::object(result_data)))
            },
            "copy" => {
                // Get parameters
                let src = params.get_string("src").ok_or_else(|| ToolError::new(400, "Missing src parameter"))?;
                let dst = params.get_string("dst").ok_or_else(|| ToolError::new(400, "Missing dst parameter"))?;
                let overwrite = params.get::<bool>("overwrite").unwrap_or(false);
                
                // Copy file
                self.copy_file(&src, &dst, overwrite)?;
                
                Ok(ToolResult::success(Value::boolean(true)))
            },
            "move" => {
                // Get parameters
                let src = params.get_string("src").ok_or_else(|| ToolError::new(400, "Missing src parameter"))?;
                let dst = params.get_string("dst").ok_or_else(|| ToolError::new(400, "Missing dst parameter"))?;
                let overwrite = params.get::<bool>("overwrite").unwrap_or(false);
                
                // Move file
                self.move_file(&src, &dst, overwrite)?;
                
                Ok(ToolResult::success(Value::boolean(true)))
            },
            _ => Err(ToolError::new(400, format!("Unknown command: {}", command))),
        }
    }
}
