// Document management module for LSP-like Component
//
// This module handles the management of text documents, including
// parsing, tracking changes, and providing access to document content.

use std::collections::HashMap;
use crate::language_hub_server::lsp::protocol::{Position, Range};

/// Represents a text document managed by the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    /// The document's URI
    pub uri: String,
    
    /// The document's language ID
    pub language_id: String,
    
    /// The document's version number
    pub version: i64,
    
    /// The document's content
    pub text: String,
    
    /// Line index for efficient position calculations
    line_index: Vec<usize>,
}

impl Document {
    /// Create a new document
    pub fn new(uri: String, language_id: String, version: i64, text: String) -> Self {
        let mut doc = Document {
            uri,
            language_id,
            version,
            text,
            line_index: Vec::new(),
        };
        doc.update_line_index();
        doc
    }
    
    /// Update the document's content
    pub fn update(&mut self, version: i64, text: String) {
        self.version = version;
        self.text = text;
        self.update_line_index();
    }
    
    /// Apply changes to the document
    pub fn apply_changes(&mut self, version: i64, changes: Vec<TextDocumentContentChangeEvent>) {
        self.version = version;
        
        for change in changes {
            if let Some(range) = change.range {
                let start_offset = self.position_to_offset(range.start);
                let end_offset = self.position_to_offset(range.end);
                
                if start_offset <= end_offset && end_offset <= self.text.len() {
                    let prefix = &self.text[..start_offset];
                    let suffix = &self.text[end_offset..];
                    self.text = format!("{}{}{}", prefix, change.text, suffix);
                }
            } else {
                // Full document update
                self.text = change.text;
            }
        }
        
        self.update_line_index();
    }
    
    /// Get the text at the specified range
    pub fn get_text_at_range(&self, range: Range) -> String {
        let start_offset = self.position_to_offset(range.start);
        let end_offset = self.position_to_offset(range.end);
        
        if start_offset <= end_offset && end_offset <= self.text.len() {
            self.text[start_offset..end_offset].to_string()
        } else {
            String::new()
        }
    }
    
    /// Convert a position to an offset in the document
    pub fn position_to_offset(&self, position: Position) -> usize {
        let line = position.line as usize;
        let character = position.character as usize;
        
        if line >= self.line_index.len() {
            return self.text.len();
        }
        
        let line_start = self.line_index[line];
        let line_length = if line + 1 < self.line_index.len() {
            self.line_index[line + 1] - line_start - 1 // -1 for newline
        } else {
            self.text.len() - line_start
        };
        
        line_start + std::cmp::min(character, line_length)
    }
    
    /// Convert an offset to a position in the document
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let offset = std::cmp::min(offset, self.text.len());
        
        // Binary search to find the line
        let mut low = 0;
        let mut high = self.line_index.len();
        
        while low < high {
            let mid = (low + high) / 2;
            let line_offset = self.line_index[mid];
            
            if line_offset <= offset {
                if mid + 1 >= self.line_index.len() || self.line_index[mid + 1] > offset {
                    // Found the line
                    let line = mid;
                    let character = offset - line_offset;
                    return Position {
                        line: line as u32,
                        character: character as u32,
                    };
                }
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        
        // Default to position 0,0 if not found
        Position {
            line: 0,
            character: 0,
        }
    }
    
    /// Get the line at the specified line number
    pub fn get_line(&self, line: u32) -> Option<String> {
        let line = line as usize;
        if line >= self.line_index.len() {
            return None;
        }
        
        let start = self.line_index[line];
        let end = if line + 1 < self.line_index.len() {
            self.line_index[line + 1] - 1 // -1 to exclude newline
        } else {
            self.text.len()
        };
        
        Some(self.text[start..end].to_string())
    }
    
    /// Get the number of lines in the document
    pub fn line_count(&self) -> usize {
        self.line_index.len()
    }
    
    /// Update the line index for efficient position calculations
    fn update_line_index(&mut self) {
        self.line_index.clear();
        self.line_index.push(0); // First line starts at offset 0
        
        let mut offset = 0;
        for c in self.text.chars() {
            offset += 1;
            if c == '\n' {
                self.line_index.push(offset);
            }
        }
    }
}

/// Document manager for handling multiple documents
pub struct DocumentManager {
    documents: HashMap<String, Document>,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        DocumentManager {
            documents: HashMap::new(),
        }
    }
    
    /// Open a document
    pub fn open_document(&mut self, uri: String, language_id: String, version: i64, text: String) {
        let document = Document::new(uri.clone(), language_id, version, text);
        self.documents.insert(uri, document);
    }
    
    /// Close a document
    pub fn close_document(&mut self, uri: &str) -> Option<Document> {
        self.documents.remove(uri)
    }
    
    /// Get a document by URI
    pub fn get_document(&self, uri: &str) -> Option<&Document> {
        self.documents.get(uri)
    }
    
    /// Get a mutable reference to a document by URI
    pub fn get_document_mut(&mut self, uri: &str) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }
    
    /// Check if a document exists
    pub fn has_document(&self, uri: &str) -> bool {
        self.documents.contains_key(uri)
    }
    
    /// Get all documents
    pub fn get_all_documents(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }
    
    /// Update a document
    pub fn update_document(&mut self, uri: &str, version: i64, changes: Vec<TextDocumentContentChangeEvent>) -> Result<(), String> {
        if let Some(document) = self.get_document_mut(uri) {
            document.apply_changes(version, changes);
            Ok(())
        } else {
            Err(format!("Document not found: {}", uri))
        }
    }
}

/// Text document content change event
#[derive(Debug, Clone)]
pub struct TextDocumentContentChangeEvent {
    /// The range of the document that changed
    pub range: Option<Range>,
    
    /// The new text for the range
    pub text: String,
}
