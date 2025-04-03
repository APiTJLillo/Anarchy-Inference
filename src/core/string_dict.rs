// src/core/string_dict.rs - String dictionary implementation
// This file contains the StringDictionary type and related functionality

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::LangError;

/// A string dictionary that maps keys to string values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringDictionary {
    /// The dictionary mapping keys to string values
    strings: HashMap<String, String>,
    /// The name of the dictionary (for multiple dictionaries support)
    name: String,
}

impl StringDictionary {
    /// Create a new empty string dictionary
    pub fn new(name: &str) -> Self {
        Self {
            strings: HashMap::new(),
            name: name.to_string(),
        }
    }
    
    /// Load a string dictionary from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LangError> {
        let content = fs::read_to_string(path)
            .map_err(|e| LangError::io_error(&format!("Failed to read string dictionary file: {}", e)))?;
        
        serde_json::from_str(&content)
            .map_err(|e| LangError::runtime_error(&format!("Failed to parse string dictionary: {}", e)))
    }
    
    /// Save the string dictionary to a JSON file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LangError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| LangError::runtime_error(&format!("Failed to serialize string dictionary: {}", e)))?;
        
        fs::write(path, content)
            .map_err(|e| LangError::io_error(&format!("Failed to write string dictionary file: {}", e)))
    }
    
    /// Get a string from the dictionary
    pub fn get(&self, key: &str) -> Option<&String> {
        self.strings.get(key)
    }
    
    /// Set a string in the dictionary
    pub fn set(&mut self, key: String, value: String) {
        self.strings.insert(key, value);
    }
    
    /// Remove a string from the dictionary
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.strings.remove(key)
    }
    
    /// Check if the dictionary contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.strings.contains_key(key)
    }
    
    /// Get the name of the dictionary
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set the name of the dictionary
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    
    /// Get the number of entries in the dictionary
    pub fn len(&self) -> usize {
        self.strings.len()
    }
    
    /// Check if the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
    
    /// Format a string with arguments
    /// Replaces {} placeholders with the corresponding arguments
    pub fn format(&self, key: &str, args: &[String]) -> Result<String, LangError> {
        let template = self.get(key)
            .ok_or_else(|| LangError::runtime_error(&format!("String key '{}' not found in dictionary", key)))?;
        
        let mut result = template.clone();
        let mut arg_index = 0;
        
        // Simple placeholder replacement
        while let Some(pos) = result.find("{}") {
            if arg_index >= args.len() {
                return Err(LangError::runtime_error(&format!(
                    "Not enough arguments for string formatting: expected more than {}, got {}",
                    arg_index, args.len()
                )));
            }
            
            result.replace_range(pos..pos+2, &args[arg_index]);
            arg_index += 1;
        }
        
        Ok(result)
    }
}

/// Global string dictionary manager
#[derive(Debug, Clone)]
pub struct StringDictionaryManager {
    /// The active dictionaries
    dictionaries: HashMap<String, StringDictionary>,
    /// The current active dictionary name
    current: String,
}

impl StringDictionaryManager {
    /// Create a new string dictionary manager with a default dictionary
    pub fn new() -> Self {
        let mut dictionaries = HashMap::new();
        let default_dict = StringDictionary::new("default");
        dictionaries.insert("default".to_string(), default_dict);
        
        Self {
            dictionaries,
            current: "default".to_string(),
        }
    }
    
    /// Get the current active dictionary
    pub fn current(&self) -> &StringDictionary {
        self.dictionaries.get(&self.current).unwrap()
    }
    
    /// Get a mutable reference to the current active dictionary
    pub fn current_mut(&mut self) -> &mut StringDictionary {
        self.dictionaries.get_mut(&self.current).unwrap()
    }
    
    /// Set the current active dictionary
    pub fn set_current(&mut self, name: &str) -> Result<(), LangError> {
        if !self.dictionaries.contains_key(name) {
            return Err(LangError::runtime_error(&format!("String dictionary '{}' not found", name)));
        }
        
        self.current = name.to_string();
        Ok(())
    }
    
    /// Add a dictionary
    pub fn add_dictionary(&mut self, dict: StringDictionary) {
        self.dictionaries.insert(dict.name().to_string(), dict);
    }
    
    /// Remove a dictionary
    pub fn remove_dictionary(&mut self, name: &str) -> Result<StringDictionary, LangError> {
        if name == "default" {
            return Err(LangError::runtime_error("Cannot remove the default dictionary"));
        }
        
        if name == self.current {
            return Err(LangError::runtime_error("Cannot remove the current active dictionary"));
        }
        
        self.dictionaries.remove(name)
            .ok_or_else(|| LangError::runtime_error(&format!("String dictionary '{}' not found", name)))
    }
    
    /// Load a dictionary from a file
    pub fn load_dictionary<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LangError> {
        let dict = StringDictionary::from_file(path)?;
        self.add_dictionary(dict);
        Ok(())
    }
    
    /// Save a dictionary to a file
    pub fn save_dictionary<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), LangError> {
        let dict = self.dictionaries.get(name)
            .ok_or_else(|| LangError::runtime_error(&format!("String dictionary '{}' not found", name)))?;
        
        dict.to_file(path)
    }
    
    /// Get a string from the current dictionary
    pub fn get_string(&self, key: &str) -> Option<&String> {
        self.current().get(key)
    }
    
    /// Set a string in the current dictionary
    pub fn set_string(&mut self, key: String, value: String) {
        self.current_mut().set(key, value);
    }
    
    /// Format a string with arguments from the current dictionary
    pub fn format_string(&self, key: &str, args: &[String]) -> Result<String, LangError> {
        self.current().format(key, args)
    }
}

// Create a default instance for testing
impl Default for StringDictionaryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_dictionary_basic() {
        let mut dict = StringDictionary::new("test");
        dict.set("a".to_string(), "Hello, world!".to_string());
        dict.set("b".to_string(), "Goodbye, world!".to_string());
        
        assert_eq!(dict.get("a"), Some(&"Hello, world!".to_string()));
        assert_eq!(dict.get("b"), Some(&"Goodbye, world!".to_string()));
        assert_eq!(dict.get("c"), None);
        
        assert_eq!(dict.len(), 2);
        assert!(!dict.is_empty());
        
        dict.remove("a");
        assert_eq!(dict.get("a"), None);
        assert_eq!(dict.len(), 1);
    }
    
    #[test]
    fn test_string_formatting() {
        let mut dict = StringDictionary::new("test");
        dict.set("greeting".to_string(), "Hello, {}!".to_string());
        dict.set("complex".to_string(), "Hello, {}! Your score is {}.".to_string());
        
        let result = dict.format("greeting", &["world".to_string()]).unwrap();
        assert_eq!(result, "Hello, world!");
        
        let result = dict.format("complex", &["Alice".to_string(), "42".to_string()]).unwrap();
        assert_eq!(result, "Hello, Alice! Your score is 42.");
        
        // Test error cases
        let err = dict.format("greeting", &[]).unwrap_err();
        assert!(err.to_string().contains("Not enough arguments"));
        
        let err = dict.format("nonexistent", &["test".to_string()]).unwrap_err();
        assert!(err.to_string().contains("not found in dictionary"));
    }
    
    #[test]
    fn test_dictionary_manager() {
        let mut manager = StringDictionaryManager::new();
        
        // Test default dictionary
        manager.set_string("a".to_string(), "Hello, world!".to_string());
        assert_eq!(manager.get_string("a"), Some(&"Hello, world!".to_string()));
        
        // Test adding a new dictionary
        let mut dict = StringDictionary::new("test");
        dict.set("b".to_string(), "Goodbye, world!".to_string());
        manager.add_dictionary(dict);
        
        // Test switching dictionaries
        manager.set_current("test").unwrap();
        assert_eq!(manager.get_string("b"), Some(&"Goodbye, world!".to_string()));
        assert_eq!(manager.get_string("a"), None);
        
        // Test switching back
        manager.set_current("default").unwrap();
        assert_eq!(manager.get_string("a"), Some(&"Hello, world!".to_string()));
        assert_eq!(manager.get_string("b"), None);
    }
}
