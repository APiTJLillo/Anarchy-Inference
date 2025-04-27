// Persistence management module for Advanced REPL Service
//
// This module provides functionality for persisting session state to disk
// and restoring it when needed, enabling session recovery and long-term storage.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::language_hub_server::repl::session::{Session, SessionConfig};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Whether to enable persistence
    pub enable_persistence: bool,
    
    /// Persistence directory
    pub persistence_dir: String,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        PersistenceConfig {
            enable_persistence: true,
            persistence_dir: "./sessions".to_string(),
        }
    }
}

/// Persistence manager
pub struct PersistenceManager {
    /// Persistence configuration
    config: PersistenceConfig,
    
    /// Last save times for each session
    last_saves: HashMap<String, Instant>,
    
    /// Pending changes for each session
    pending_changes: HashMap<String, bool>,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub fn new(config: PersistenceConfig) -> Self {
        // Create the persistence directory if it doesn't exist
        if config.enable_persistence {
            let path = Path::new(&config.persistence_dir);
            if !path.exists() {
                if let Err(e) = fs::create_dir_all(path) {
                    eprintln!("Warning: Failed to create persistence directory: {}", e);
                }
            }
        }
        
        PersistenceManager {
            config,
            last_saves: HashMap::new(),
            pending_changes: HashMap::new(),
        }
    }
    
    /// Initialize a session for persistence
    pub fn initialize_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.config.enable_persistence {
            return Ok(());
        }
        
        // Mark the session as having pending changes
        self.pending_changes.insert(session_id.to_string(), true);
        
        // Set the last save time
        self.last_saves.insert(session_id.to_string(), Instant::now());
        
        Ok(())
    }
    
    /// Save a session
    pub fn save_session(&mut self, session: &Session) -> Result<(), String> {
        if !self.config.enable_persistence || !session.config.persistence {
            return Ok(());
        }
        
        // Check if there are pending changes
        if !self.pending_changes.get(&session.id).unwrap_or(&false) {
            return Ok(());
        }
        
        // Get the session file path
        let file_path = self.get_session_file_path(&session.id);
        
        // Serialize the session
        let session_json = match serde_json::to_string_pretty(session) {
            Ok(json) => json,
            Err(e) => return Err(format!("Failed to serialize session: {}", e)),
        };
        
        // Create the parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return Err(format!("Failed to create directory: {}", e));
                }
            }
        }
        
        // Write the session to file
        let mut file = match File::create(&file_path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to create file: {}", e)),
        };
        
        if let Err(e) = file.write_all(session_json.as_bytes()) {
            return Err(format!("Failed to write to file: {}", e));
        }
        
        // Update the last save time
        self.last_saves.insert(session.id.clone(), Instant::now());
        
        // Clear the pending changes flag
        self.pending_changes.insert(session.id.clone(), false);
        
        Ok(())
    }
    
    /// Load a session
    pub fn load_session(&self, session_id: &str) -> Result<Session, String> {
        if !self.config.enable_persistence {
            return Err("Persistence is not enabled".to_string());
        }
        
        // Get the session file path
        let file_path = self.get_session_file_path(session_id);
        
        // Check if the file exists
        if !file_path.exists() {
            return Err(format!("Session file not found: {}", file_path.display()));
        }
        
        // Read the file
        let mut file = match File::open(&file_path) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to open file: {}", e)),
        };
        
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            return Err(format!("Failed to read file: {}", e));
        }
        
        // Deserialize the session
        match serde_json::from_str(&contents) {
            Ok(session) => Ok(session),
            Err(e) => Err(format!("Failed to deserialize session: {}", e)),
        }
    }
    
    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.config.enable_persistence {
            return Ok(());
        }
        
        // Get the session file path
        let file_path = self.get_session_file_path(session_id);
        
        // Check if the file exists
        if !file_path.exists() {
            return Ok(());
        }
        
        // Delete the file
        if let Err(e) = fs::remove_file(&file_path) {
            return Err(format!("Failed to delete file: {}", e));
        }
        
        // Remove the session from the maps
        self.last_saves.remove(session_id);
        self.pending_changes.remove(session_id);
        
        Ok(())
    }
    
    /// Mark a session as having pending changes
    pub fn mark_session_changed(&mut self, session_id: &str) -> Result<(), String> {
        if !self.config.enable_persistence {
            return Ok(());
        }
        
        self.pending_changes.insert(session_id.to_string(), true);
        
        Ok(())
    }
    
    /// Save all sessions with pending changes
    pub fn save_all_pending(&mut self, sessions: &HashMap<String, Session>) -> Result<(), String> {
        if !self.config.enable_persistence {
            return Ok(());
        }
        
        for (session_id, session) in sessions {
            if !session.config.persistence {
                continue;
            }
            
            if *self.pending_changes.get(session_id).unwrap_or(&false) {
                if let Err(e) = self.save_session(session) {
                    eprintln!("Warning: Failed to save session {}: {}", session_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// List all persisted sessions
    pub fn list_persisted_sessions(&self) -> Result<Vec<String>, String> {
        if !self.config.enable_persistence {
            return Ok(Vec::new());
        }
        
        // Get the sessions directory
        let sessions_dir = Path::new(&self.config.persistence_dir);
        
        // Check if the directory exists
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }
        
        // Read the directory
        let entries = match fs::read_dir(sessions_dir) {
            Ok(entries) => entries,
            Err(e) => return Err(format!("Failed to read directory: {}", e)),
        };
        
        // Collect session IDs
        let mut session_ids = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        let session_id = file_name.trim_end_matches(".json").to_string();
                        session_ids.push(session_id);
                    }
                }
            }
        }
        
        Ok(session_ids)
    }
    
    /// Get the file path for a session
    fn get_session_file_path(&self, session_id: &str) -> PathBuf {
        let mut path = PathBuf::from(&self.config.persistence_dir);
        path.push(format!("{}.json", session_id));
        path
    }
    
    /// Get persistence statistics
    pub fn get_statistics(&self) -> PersistenceStatistics {
        let sessions_dir = Path::new(&self.config.persistence_dir);
        
        let mut persisted_sessions = 0;
        let mut total_size = 0;
        
        if sessions_dir.exists() {
            if let Ok(entries) = fs::read_dir(sessions_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() {
                                persisted_sessions += 1;
                                total_size += metadata.len();
                            }
                        }
                    }
                }
            }
        }
        
        PersistenceStatistics {
            enabled: self.config.enable_persistence,
            persisted_sessions,
            total_size,
            pending_changes: self.pending_changes.values().filter(|&v| *v).count(),
        }
    }
}

/// Persistence statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceStatistics {
    /// Whether persistence is enabled
    pub enabled: bool,
    
    /// Number of persisted sessions
    pub persisted_sessions: usize,
    
    /// Total size of persisted sessions in bytes
    pub total_size: u64,
    
    /// Number of sessions with pending changes
    pub pending_changes: usize,
}
