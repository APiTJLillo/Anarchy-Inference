// src/value.rs - Modified to integrate with reference counting
// This file contains the Value type with reference counting for complex values

use std::fmt;
use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::error::LangError;
use crate::rc_value::{RcComplexValue, ComplexValue, ComplexValueType};

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
}

/// A value in the language
#[derive(Clone)]
pub enum Value {
    Null,
    Number(f64),
    Boolean(bool),
    String(String),
    Complex(RcComplexValue),
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
    
    /// Create an object value
    pub fn object(obj: HashMap<String, Value>) -> Self {
        let mut complex = ComplexValue::new_object();
        if let Some(obj_data) = &mut complex.object_data {
            *obj_data = obj;
        }
        Self::Complex(RcComplexValue::new(complex))
    }
    
    /// Create an empty object value
    pub fn empty_object() -> Self {
        Self::Complex(RcComplexValue::new(ComplexValue::new_object()))
    }
    
    /// Create an array value
    pub fn array(elements: Vec<Value>) -> Self {
        Self::Complex(RcComplexValue::new(ComplexValue::new_array(elements)))
    }
    
    /// Create a function value
    pub fn function(params: Vec<String>, body: Box<ASTNode>) -> Self {
        Self::Complex(RcComplexValue::new(ComplexValue::new_function(params, body)))
    }
    
    /// Get the type of this value
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Number(_) => ValueType::Number,
            Self::Boolean(_) => ValueType::Boolean,
            Self::String(_) => ValueType::String,
            Self::Complex(complex) => {
                match complex.borrow().value_type {
                    ComplexValueType::Object => ValueType::Object,
                    ComplexValueType::Array => ValueType::Array,
                    ComplexValueType::Function => ValueType::Function,
                }
            }
        }
    }
    
    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Result<Value, LangError> {
        match self {
            Self::Complex(complex) => {
                complex.borrow().get_property(name)
            },
            _ => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Set a property on an object
    pub fn set_property(&self, name: String, value: Value) -> Result<(), LangError> {
        match self {
            Self::Complex(complex) => {
                complex.borrow_mut().set_property(name, value)
            },
            _ => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Get an element from an array
    pub fn get_element(&self, index: usize) -> Result<Value, LangError> {
        match self {
            Self::Complex(complex) => {
                complex.borrow().get_element(index)
            },
            _ => Err(LangError::runtime_error("Not an array")),
        }
    }
    
    /// Set an element in an array
    pub fn set_element(&self, index: usize, value: Value) -> Result<(), LangError> {
        match self {
            Self::Complex(complex) => {
                complex.borrow_mut().set_element(index, value)
            },
            _ => Err(LangError::runtime_error("Not an array")),
        }
    }
    
    /// Get the function parameters and body
    pub fn get_function(&self) -> Result<(Vec<String>, Box<ASTNode>), LangError> {
        match self {
            Self::Complex(complex) => {
                complex.borrow().get_function()
            },
            _ => Err(LangError::runtime_error("Not a function")),
        }
    }
    
    /// Get the reference count for a complex value
    pub fn ref_count(&self) -> usize {
        match self {
            Self::Complex(complex) => complex.ref_count(),
            _ => 1, // Primitive values always have a reference count of 1
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
            Self::Complex(complex) => {
                write!(f, "{:?}", complex)
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
            Self::Complex(complex) => {
                let borrowed = complex.borrow();
                match borrowed.value_type {
                    crate::rc_value::ComplexValueType::Object => {
                        if let Some(obj) = &borrowed.object_data {
                            write!(f, "{{ ")?;
                            let mut first = true;
                            for (key, value) in obj {
                                if !first {
                                    write!(f, ", ")?;
                                }
                                first = false;
                                write!(f, "\"{}\": {}", key, value)?;
                            }
                            write!(f, " }}")
                        } else {
                            write!(f, "{{ }}")
                        }
                    },
                    ComplexValueType::Array => {
                        if let Some(arr) = &borrowed.array_data {
                            write!(f, "[")?;
                            let mut first = true;
                            for value in arr {
                                if !first {
                                    write!(f, ", ")?;
                                }
                                first = false;
                                write!(f, "{}", value)?;
                            }
                            write!(f, "]")
                        } else {
                            write!(f, "[]")
                        }
                    },
                    ComplexValueType::Function => {
                        if let Some((params, _)) = &borrowed.function_data {
                            write!(f, "function({}) {{ ... }}", params.join(", "))
                        } else {
                            write!(f, "function() {{ ... }}")
                        }
                    },
                }
            }
        }
    }
}
