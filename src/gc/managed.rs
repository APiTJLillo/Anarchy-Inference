// Re-export GcValue for public use
pub use crate::core::value::GcValue;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::ast::ASTNode;
use crate::core::value::Value;
use crate::interpreter::Environment;

/// Concrete implementations of complex values managed by the garbage collector
#[derive(Debug, Clone)]
pub enum GcValueImpl {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<ASTNode>,
        closure: Arc<Environment>,
    },
    // Other complex types that need GC
}

impl GcValue {
    /// Extract references from a value
    pub fn extract_references(value: &GcValueImpl) -> HashSet<usize> {
        let mut references = HashSet::new();
        
        match value {
            GcValueImpl::Object(map) => {
                for (_, v) in map {
                    if let Value::GcManaged(gc_value) = v {
                        references.insert(gc_value.id);
                        
                        // Also check for nested references
                        if let Some(nested_value) = gc_value.gc.get_value(gc_value.id) {
                            let nested_refs = Self::extract_references(&nested_value);
                            references.extend(nested_refs);
                        }
                    }
                }
            },
            GcValueImpl::Array(items) => {
                for item in items {
                    if let Value::GcManaged(gc_value) = item {
                        references.insert(gc_value.id);
                        
                        // Also check for nested references
                        if let Some(nested_value) = gc_value.gc.get_value(gc_value.id) {
                            let nested_refs = Self::extract_references(&nested_value);
                            references.extend(nested_refs);
                        }
                    }
                }
            },
            GcValueImpl::Function { closure, .. } => {
                // Extract references from the closure environment
                // This would need a more complex implementation to traverse the environment
                // and find all GcValue references
                
                // For now, we'll just mark functions as potential cycle candidates
                // and let the cycle detection algorithm handle them
            },
        }
        
        references
    }
    
    /// Get the size of a value in bytes (approximate)
    pub fn get_size(value: &GcValueImpl) -> usize {
        match value {
            GcValueImpl::Object(map) => {
                // Base size + size of each key-value pair
                std::mem::size_of::<GcValueImpl>() + 
                map.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
            },
            GcValueImpl::Array(items) => {
                // Base size + size of each element
                std::mem::size_of::<GcValueImpl>() + 
                items.len() * std::mem::size_of::<Value>()
            },
            GcValueImpl::Function { name, parameters, body, closure } => {
                // Base size + size of name, parameters, and an estimate for body and closure
                std::mem::size_of::<GcValueImpl>() + 
                name.len() + 
                parameters.len() * std::mem::size_of::<String>() + 
                256  // Estimate for body and closure
            },
        }
    }
}

impl GcValueImpl {
    /// Create a new object
    pub fn new_object() -> Self {
        Self::Object(HashMap::new())
    }
    
    /// Create a new array
    pub fn new_array(elements: Vec<Value>) -> Self {
        Self::Array(elements)
    }
    
    /// Create a new function
    pub fn new_function(name: String, parameters: Vec<String>, body: Box<ASTNode>, closure: Arc<Environment>) -> Self {
        Self::Function {
            name,
            parameters,
            body,
            closure,
        }
    }
    
    /// Get the type of this value as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Object(_) => "Object",
            Self::Array(_) => "Array",
            Self::Function { .. } => "Function",
        }
    }
    
    /// Check if this value might form a reference cycle
    pub fn might_form_cycle(&self) -> bool {
        match self {
            Self::Object(_) | Self::Array(_) | Self::Function { .. } => true,
            // Add other complex types that might form cycles
        }
    }
    
    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Option<Value> {
        match self {
            Self::Object(map) => map.get(name).cloned(),
            _ => None,
        }
    }
    
    /// Set a property on an object
    pub fn set_property(&mut self, name: String, value: Value) -> bool {
        match self {
            Self::Object(map) => {
                map.insert(name, value);
                true
            },
            _ => false,
        }
    }
    
    /// Get an element from an array
    pub fn get_element(&self, index: usize) -> Option<Value> {
        match self {
            Self::Array(items) => {
                if index < items.len() {
                    Some(items[index].clone())
                } else {
                    None
                }
            },
            _ => None,
        }
    }
    
    /// Set an element in an array
    pub fn set_element(&mut self, index: usize, value: Value) -> bool {
        match self {
            Self::Array(items) => {
                if index < items.len() {
                    items[index] = value;
                    true
                } else {
                    false
                }
            },
            _ => false,
        }
    }
}
