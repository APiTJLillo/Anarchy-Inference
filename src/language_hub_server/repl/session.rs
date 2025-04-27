// Session management module for Advanced REPL Service
//
// This module provides functionality for creating and managing multiple named sessions
// with support for session lifecycle management, resource allocation, and cleanup.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session name
    pub name: String,
    
    /// Session timeout
    pub timeout: Duration,
    
    /// Whether to enable persistence
    pub persistence: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        SessionConfig {
            name: format!("session_{}", Uuid::new_v4()),
            timeout: Duration::from_secs(3600), // 1 hour
            persistence: true,
        }
    }
}

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    
    /// Session configuration
    pub config: SessionConfig,
    
    /// Creation time
    pub created: DateTime<Utc>,
    
    /// Last accessed time
    pub last_accessed: DateTime<Utc>,
    
    /// Session variables
    pub variables: HashMap<String, serde_json::Value>,
    
    /// Execution history
    pub history: Vec<ExecutionHistoryEntry>,
}

/// Execution history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistoryEntry {
    /// Execution ID
    pub id: String,
    
    /// Code executed
    pub code: String,
    
    /// Execution result
    pub result: Option<serde_json::Value>,
    
    /// Execution output
    pub output: Option<String>,
    
    /// Execution duration in milliseconds
    pub duration: u64,
    
    /// Execution status
    pub status: String,
    
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
}

/// Session manager
pub struct SessionManager {
    /// Maximum number of sessions
    max_sessions: usize,
    
    /// Active sessions
    sessions: HashMap<String, Session>,
    
    /// Last cleanup time
    last_cleanup: Instant,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_sessions: usize) -> Self {
        SessionManager {
            max_sessions,
            sessions: HashMap::new(),
            last_cleanup: Instant::now(),
        }
    }
    
    /// Create a new session
    pub fn create_session(&mut self, config: SessionConfig) -> Result<String, String> {
        // Check if we've reached the maximum number of sessions
        if self.sessions.len() >= self.max_sessions {
            // Try to clean up expired sessions
            self.cleanup_expired_sessions();
            
            // Check again
            if self.sessions.len() >= self.max_sessions {
                return Err("Maximum number of sessions reached".to_string());
            }
        }
        
        // Generate a session ID
        let session_id = Uuid::new_v4().to_string();
        
        // Create the session
        let session = Session {
            id: session_id.clone(),
            config,
            created: Utc::now(),
            last_accessed: Utc::now(),
            variables: HashMap::new(),
            history: Vec::new(),
        };
        
        // Add the session to the sessions map
        self.sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    /// Get a session
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }
    
    /// Get a mutable session
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }
    
    /// Check if a session exists
    pub fn session_exists(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }
    
    /// Delete a session
    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session not found: {}", session_id));
        }
        
        self.sessions.remove(session_id);
        
        Ok(())
    }
    
    /// Update session configuration
    pub fn update_session_config(&mut self, session_id: &str, config: SessionConfig) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session not found: {}", session_id));
        }
        
        let session = self.sessions.get_mut(session_id).unwrap();
        session.config = config;
        
        Ok(())
    }
    
    /// List all sessions
    pub fn list_sessions(&self) -> Vec<SessionSummary> {
        self.sessions.iter()
            .map(|(id, session)| {
                SessionSummary {
                    id: id.clone(),
                    name: session.config.name.clone(),
                    created: session.created,
                    last_accessed: session.last_accessed,
                    variable_count: session.variables.len(),
                    history_count: session.history.len(),
                }
            })
            .collect()
    }
    
    /// Add an execution history entry
    pub fn add_history_entry(&mut self, session_id: &str, entry: ExecutionHistoryEntry) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session not found: {}", session_id));
        }
        
        let session = self.sessions.get_mut(session_id).unwrap();
        session.history.push(entry);
        
        // Limit history size
        if session.history.len() > 100 {
            session.history.remove(0);
        }
        
        Ok(())
    }
    
    /// Set a variable
    pub fn set_variable(&mut self, session_id: &str, name: &str, value: serde_json::Value) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session not found: {}", session_id));
        }
        
        let session = self.sessions.get_mut(session_id).unwrap();
        session.variables.insert(name.to_string(), value);
        
        Ok(())
    }
    
    /// Get a variable
    pub fn get_variable(&self, session_id: &str, name: &str) -> Result<Option<&serde_json::Value>, String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session not found: {}", session_id));
        }
        
        let session = self.sessions.get(session_id).unwrap();
        Ok(session.variables.get(name))
    }
    
    /// Delete a variable
    pub fn delete_variable(&mut self, session_id: &str, name: &str) -> Result<(), String> {
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session not found: {}", session_id));
        }
        
        let session = self.sessions.get_mut(session_id).unwrap();
        session.variables.remove(name);
        
        Ok(())
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        // Only clean up once every minute
        if self.last_cleanup.elapsed() < Duration::from_secs(60) {
            return;
        }
        
        // Update the last cleanup time
        self.last_cleanup = Instant::now();
        
        // Find expired sessions
        let now = Utc::now();
        let expired_sessions: Vec<String> = self.sessions.iter()
            .filter_map(|(id, session)| {
                let elapsed = now.signed_duration_since(session.last_accessed);
                if elapsed.num_seconds() as u64 > session.config.timeout.as_secs() {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Remove expired sessions
        for id in expired_sessions {
            self.sessions.remove(&id);
        }
    }
    
    /// Get session statistics
    pub fn get_statistics(&self) -> SessionManagerStatistics {
        let now = Utc::now();
        
        let active_sessions = self.sessions.len();
        
        let mut total_variables = 0;
        let mut total_history_entries = 0;
        let mut oldest_session_age = 0;
        let mut newest_session_age = u64::MAX;
        
        for session in self.sessions.values() {
            total_variables += session.variables.len();
            total_history_entries += session.history.len();
            
            let age = now.signed_duration_since(session.created).num_seconds() as u64;
            oldest_session_age = oldest_session_age.max(age);
            newest_session_age = newest_session_age.min(age);
        }
        
        if active_sessions == 0 {
            newest_session_age = 0;
        }
        
        SessionManagerStatistics {
            active_sessions,
            max_sessions: self.max_sessions,
            total_variables,
            total_history_entries,
            oldest_session_age,
            newest_session_age,
        }
    }
}

/// Session summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID
    pub id: String,
    
    /// Session name
    pub name: String,
    
    /// Creation time
    pub created: DateTime<Utc>,
    
    /// Last accessed time
    pub last_accessed: DateTime<Utc>,
    
    /// Number of variables
    pub variable_count: usize,
    
    /// Number of history entries
    pub history_count: usize,
}

/// Session manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagerStatistics {
    /// Number of active sessions
    pub active_sessions: usize,
    
    /// Maximum number of sessions
    pub max_sessions: usize,
    
    /// Total number of variables across all sessions
    pub total_variables: usize,
    
    /// Total number of history entries across all sessions
    pub total_history_entries: usize,
    
    /// Age of the oldest session in seconds
    pub oldest_session_age: u64,
    
    /// Age of the newest session in seconds
    pub newest_session_age: u64,
}
