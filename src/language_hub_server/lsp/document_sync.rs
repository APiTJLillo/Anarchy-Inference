// Document management system implementation for LSP-like Component
//
// This module extends the basic document management functionality with
// additional features required for the LSP server, including change tracking,
// versioning, and synchronization.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::document::{Document, DocumentManager, TextDocumentContentChangeEvent};
use crate::language_hub_server::lsp::protocol::{Position, Range};

/// Document change tracking system
pub struct DocumentChangeTracker {
    /// The document manager
    document_manager: Arc<Mutex<DocumentManager>>,
    
    /// Map of document URIs to their last known version
    document_versions: HashMap<String, i64>,
    
    /// Map of document URIs to their change history
    change_history: HashMap<String, Vec<DocumentChange>>,
    
    /// Maximum number of changes to keep in history per document
    max_history_length: usize,
}

/// Document change record
#[derive(Debug, Clone)]
pub struct DocumentChange {
    /// The version after this change
    pub version: i64,
    
    /// The changes applied
    pub changes: Vec<TextDocumentContentChangeEvent>,
    
    /// Timestamp of the change
    pub timestamp: std::time::SystemTime,
}

impl DocumentChangeTracker {
    /// Create a new document change tracker
    pub fn new(document_manager: Arc<Mutex<DocumentManager>>) -> Self {
        DocumentChangeTracker {
            document_manager,
            document_versions: HashMap::new(),
            change_history: HashMap::new(),
            max_history_length: 50, // Default to keeping 50 changes
        }
    }
    
    /// Set the maximum history length
    pub fn set_max_history_length(&mut self, length: usize) {
        self.max_history_length = length;
    }
    
    /// Track a document open event
    pub fn track_document_open(&mut self, uri: &str, version: i64) {
        self.document_versions.insert(uri.to_string(), version);
        self.change_history.insert(uri.to_string(), Vec::new());
    }
    
    /// Track a document close event
    pub fn track_document_close(&mut self, uri: &str) {
        self.document_versions.remove(uri);
        self.change_history.remove(uri);
    }
    
    /// Track document changes
    pub fn track_document_changes(&mut self, uri: &str, version: i64, changes: Vec<TextDocumentContentChangeEvent>) -> Result<(), String> {
        // Update the document
        {
            let mut manager = self.document_manager.lock().unwrap();
            manager.update_document(uri, version, changes.clone())?;
        }
        
        // Update version tracking
        self.document_versions.insert(uri.to_string(), version);
        
        // Record the change in history
        if let Some(history) = self.change_history.get_mut(uri) {
            history.push(DocumentChange {
                version,
                changes,
                timestamp: std::time::SystemTime::now(),
            });
            
            // Trim history if it exceeds max length
            if history.len() > self.max_history_length {
                let excess = history.len() - self.max_history_length;
                history.drain(0..excess);
            }
        }
        
        Ok(())
    }
    
    /// Get the current version of a document
    pub fn get_document_version(&self, uri: &str) -> Option<i64> {
        self.document_versions.get(uri).cloned()
    }
    
    /// Get the change history for a document
    pub fn get_change_history(&self, uri: &str) -> Option<&Vec<DocumentChange>> {
        self.change_history.get(uri)
    }
    
    /// Check if a document is being tracked
    pub fn is_tracking_document(&self, uri: &str) -> bool {
        self.document_versions.contains_key(uri)
    }
}

/// Document synchronization manager
pub struct DocumentSyncManager {
    /// The document change tracker
    change_tracker: Arc<Mutex<DocumentChangeTracker>>,
    
    /// Synchronization mode
    sync_mode: TextDocumentSyncKind,
}

/// Text document synchronization kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDocumentSyncKind {
    /// Documents are not synchronized
    None = 0,
    
    /// Documents are synchronized by sending the full content
    Full = 1,
    
    /// Documents are synchronized by sending incremental updates
    Incremental = 2,
}

impl DocumentSyncManager {
    /// Create a new document synchronization manager
    pub fn new(document_manager: Arc<Mutex<DocumentManager>>, sync_mode: TextDocumentSyncKind) -> Self {
        let change_tracker = Arc::new(Mutex::new(DocumentChangeTracker::new(document_manager)));
        
        DocumentSyncManager {
            change_tracker,
            sync_mode,
        }
    }
    
    /// Handle a document open notification
    pub fn handle_document_open(&self, uri: &str, language_id: &str, version: i64, text: &str) -> Result<(), String> {
        // Add the document to the manager
        {
            let document_manager = {
                let tracker = self.change_tracker.lock().unwrap();
                tracker.document_manager.clone()
            };
            
            let mut manager = document_manager.lock().unwrap();
            manager.open_document(uri.to_string(), language_id.to_string(), version, text.to_string());
        }
        
        // Track the document
        let mut tracker = self.change_tracker.lock().unwrap();
        tracker.track_document_open(uri, version);
        
        Ok(())
    }
    
    /// Handle a document close notification
    pub fn handle_document_close(&self, uri: &str) -> Result<(), String> {
        // Remove the document from the manager
        {
            let document_manager = {
                let tracker = self.change_tracker.lock().unwrap();
                tracker.document_manager.clone()
            };
            
            let mut manager = document_manager.lock().unwrap();
            manager.close_document(uri);
        }
        
        // Stop tracking the document
        let mut tracker = self.change_tracker.lock().unwrap();
        tracker.track_document_close(uri);
        
        Ok(())
    }
    
    /// Handle a document change notification
    pub fn handle_document_change(&self, uri: &str, version: i64, changes: Vec<TextDocumentContentChangeEvent>) -> Result<(), String> {
        // Track the changes
        let mut tracker = self.change_tracker.lock().unwrap();
        tracker.track_document_changes(uri, version, changes)
    }
    
    /// Get a document by URI
    pub fn get_document(&self, uri: &str) -> Option<Document> {
        let tracker = self.change_tracker.lock().unwrap();
        let document_manager = tracker.document_manager.clone();
        let manager = document_manager.lock().unwrap();
        
        manager.get_document(uri).cloned()
    }
    
    /// Get the current synchronization mode
    pub fn get_sync_mode(&self) -> TextDocumentSyncKind {
        self.sync_mode
    }
    
    /// Set the synchronization mode
    pub fn set_sync_mode(&mut self, mode: TextDocumentSyncKind) {
        self.sync_mode = mode;
    }
}

/// Shared document synchronization manager that can be used across threads
pub type SharedDocumentSyncManager = Arc<Mutex<DocumentSyncManager>>;

/// Create a new shared document synchronization manager
pub fn create_shared_document_sync_manager(document_manager: Arc<Mutex<DocumentManager>>, sync_mode: TextDocumentSyncKind) -> SharedDocumentSyncManager {
    Arc::new(Mutex::new(DocumentSyncManager::new(document_manager, sync_mode)))
}
