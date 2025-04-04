// src/core/value.rs
// Core value types for the language with garbage collection support

use std::fmt;
use std::sync::Arc;
use std::collections::HashSet;
use crate::ast::ASTNode;
use crate::error::LangError;
use crate::gc::managed::GcValueImpl;
use crate::core::gc_types::GarbageCollector;
use crate::gc::managed::GcValue as ManagedGcValue;

/// Types of values in the language
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Null,
    Number,
    Boolean,
    String,
    Object,
    Array,
    Function,
    // Add other value types as needed
}

/// A value in the language
#[derive(Clone)]
pub enum Value {
    Null,
    Number(f64),
    Boolean(bool),
    String(String),
    GcManaged(GcValue),
    // Add other value types as needed
}

/// A garbage-collected value wrapper
#[derive(Clone)]
pub struct GcValue {
    // Unique identifier in the GC
    pub id: usize,
    // Back-reference to the garbage collector
    pub gc: Arc<dyn GarbageCollector>,
}

impl Value {
    /// Create a null value
    pub fn null() -> Self {
        Self::Null
    }
    
    /// Create a number value
    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }
    
    /// Create a boolean value
    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }
    
    /// Create a string value
    pub fn string(s: String) -> Self {
        Self::String(s)
    }
    
    /// Get the type of this value
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Number(_) => ValueType::Number,
            Self::Boolean(_) => ValueType::Boolean,
            Self::String(_) => ValueType::String,
            Self::GcManaged(gc_value) => {
                // Get the type from the GC
                if let Some(value) = gc_value.gc.get_value(gc_value.id) {
                    match value {
                        GcValueImpl::Object(_) => ValueType::Object,
                        GcValueImpl::Array(_) => ValueType::Array,
                        GcValueImpl::Function { .. } => ValueType::Function,
                    }
                } else {
                    // Default to Object if not found
                    ValueType::Object
                }
            }
        }
    }
    
    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Result<Value, LangError> {
        match self {
            Self::GcManaged(gc_value) => {
                if let Some(value) = gc_value.gc.get_value(gc_value.id) {
                    if let GcValueImpl::Object(map) = value {
                        if let Some(prop) = map.get(name) {
                            Ok(prop.clone())
                        } else {
                            Err(LangError::runtime_error(&format!("Property '{}' not found", name)))
                        }
                    } else {
                        Err(LangError::runtime_error("Not an object"))
                    }
                } else {
                    Err(LangError::runtime_error("Invalid object reference"))
                }
            },
            _ => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Set a property on an object
    pub fn set_property(&self, name: String, value: Value) -> Result<(), LangError> {
        match self {
            Self::GcManaged(gc_value) => {
                if let Some(mut obj_value) = gc_value.gc.get_value(gc_value.id) {
                    if let GcValueImpl::Object(ref mut map) = obj_value {
                        map.insert(name, value);
                        // Update references in the GC
                        let references = ManagedGcValue::extract_references(&obj_value);
                        gc_value.gc.update_references(gc_value.id, references);
                        Ok(())
                    } else {
                        Err(LangError::runtime_error("Not an object"))
                    }
                } else {
                    Err(LangError::runtime_error("Invalid object reference"))
                }
            },
            _ => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Get an element from an array
    pub fn get_element(&self, index: usize) -> Result<Value, LangError> {
        match self {
            Self::GcManaged(gc_value) => {
                if let Some(value) = gc_value.gc.get_value(gc_value.id) {
                    if let GcValueImpl::Array(items) = value {
                        if index < items.len() {
                            Ok(items[index].clone())
                        } else {
                            Err(LangError::runtime_error(&format!("Index {} out of bounds", index)))
                        }
                    } else {
                        Err(LangError::runtime_error("Not an array"))
                    }
                } else {
                    Err(LangError::runtime_error("Invalid array reference"))
                }
            },
            _ => Err(LangError::runtime_error("Not an array")),
        }
    }
    
    /// Set an element in an array
    pub fn set_element(&self, index: usize, value: Value) -> Result<(), LangError> {
        match self {
            Self::GcManaged(gc_value) => {
                if let Some(mut arr_value) = gc_value.gc.get_value(gc_value.id) {
                    if let GcValueImpl::Array(ref mut items) = arr_value {
                        if index < items.len() {
                            items[index] = value;
                            // Update references in the GC
                            let references = ManagedGcValue::extract_references(&arr_value);
                            gc_value.gc.update_references(gc_value.id, references);
                            Ok(())
                        } else {
                            Err(LangError::runtime_error(&format!("Index {} out of bounds", index)))
                        }
                    } else {
                        Err(LangError::runtime_error("Not an array"))
                    }
                } else {
                    Err(LangError::runtime_error("Invalid array reference"))
                }
            },
            _ => Err(LangError::runtime_error("Not an array")),
        }
    }
    
    /// Get the function parameters and body
    pub fn get_function(&self) -> Result<(Vec<String>, Box<ASTNode>), LangError> {
        match self {
            Self::GcManaged(gc_value) => {
                if let Some(value) = gc_value.gc.get_value(gc_value.id) {
                    if let GcValueImpl::Function { parameters, body, .. } = value {
                        Ok((parameters, body))
                    } else {
                        Err(LangError::runtime_error("Not a function"))
                    }
                } else {
                    Err(LangError::runtime_error("Invalid function reference"))
                }
            },
            _ => Err(LangError::runtime_error("Not a function")),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::GcManaged(gc_value) => {
                if let Some(value) = gc_value.gc.get_value(gc_value.id) {
                    match value {
                        GcValueImpl::Object(_) => write!(f, "Object(id: {})", gc_value.id),
                        GcValueImpl::Array(_) => write!(f, "Array(id: {})", gc_value.id),
                        GcValueImpl::Function { name, .. } => write!(f, "Function({}, id: {})", name, gc_value.id),
                    }
                } else {
                    write!(f, "GcManaged(id: {}, invalid)", gc_value.id)
                }
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::GcManaged(gc_value) => {
                if let Some(value) = gc_value.gc.get_value(gc_value.id) {
                    match value {
                        GcValueImpl::Object(map) => {
                            write!(f, "{{ ")?;
                            let mut first = true;
                            for (key, value) in map {
                                if !first {
                                    write!(f, ", ")?;
                                }
                                first = false;
                                write!(f, "\"{}\": {}", key, value)?;
                            }
                            write!(f, " }}")
                        },
                        GcValueImpl::Array(items) => {
                            write!(f, "[")?;
                            let mut first = true;
                            for value in items {
                                if !first {
                                    write!(f, ", ")?;
                                }
                                first = false;
                                write!(f, "{}", value)?;
                            }
                            write!(f, "]")
                        },
                        GcValueImpl::Function { name, parameters, .. } => {
                            write!(f, "function {}({}) {{ ... }}", name, parameters.join(", "))
                        },
                    }
                } else {
                    write!(f, "<invalid reference>")
                }
            }
        }
    }
}
