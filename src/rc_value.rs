// src/rc_value.rs
// Simplified garbage collection using Rust's built-in reference counting

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::collections::HashMap;
use crate::ast::ASTNode;
use crate::error::LangError;

/// A reference-counted value wrapper
#[derive(Clone)]
pub struct RcValue<T: Clone> {
    /// The inner value wrapped in Rc<RefCell<>>
    inner: Rc<RefCell<T>>,
}

impl<T: Clone> RcValue<T> {
    /// Create a new reference-counted value
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }
    
    /// Get a reference to the inner value
    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.inner.borrow()
    }
    
    /// Get a mutable reference to the inner value
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.inner.borrow_mut()
    }
    
    /// Get the reference count
    pub fn ref_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }
}

impl<T: fmt::Debug + Clone> fmt::Debug for RcValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcValue")
            .field("value", &self.inner.borrow())
            .field("ref_count", &self.ref_count())
            .finish()
    }
}

/// Types of complex values that need reference counting
#[derive(Debug, Clone)]
pub enum ComplexValueType {
    Object,
    Array,
    Function,
}

/// A complex value that needs reference counting
#[derive(Debug, Clone)]
pub struct ComplexValue {
    /// The type of complex value
    pub value_type: ComplexValueType,
    /// Object data (if this is an object)
    pub object_data: Option<HashMap<String, crate::value::Value>>,
    /// Array data (if this is an array)
    pub array_data: Option<Vec<crate::value::Value>>,
    /// Function data (if this is a function)
    pub function_data: Option<(Vec<String>, Box<ASTNode>)>,
}

impl ComplexValue {
    /// Create a new object value
    pub fn new_object() -> Self {
        Self {
            value_type: ComplexValueType::Object,
            object_data: Some(HashMap::new()),
            array_data: None,
            function_data: None,
        }
    }
    
    /// Create a new array value
    pub fn new_array(elements: Vec<crate::value::Value>) -> Self {
        Self {
            value_type: ComplexValueType::Array,
            object_data: None,
            array_data: Some(elements),
            function_data: None,
        }
    }
    
    /// Create a new function value
    pub fn new_function(params: Vec<String>, body: Box<ASTNode>) -> Self {
        Self {
            value_type: ComplexValueType::Function,
            object_data: None,
            array_data: None,
            function_data: Some((params, body)),
        }
    }
    
    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Result<crate::value::Value, LangError> {
        match &self.object_data {
            Some(obj) => {
                if let Some(value) = obj.get(name) {
                    Ok(value.clone())
                } else {
                    Err(LangError::runtime_error(&format!("Property '{}' not found", name)))
                }
            },
            None => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Set a property on an object
    pub fn set_property(&mut self, name: String, value: crate::value::Value) -> Result<(), LangError> {
        match &mut self.object_data {
            Some(obj) => {
                obj.insert(name, value);
                Ok(())
            },
            None => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Get an element from an array
    pub fn get_element(&self, index: usize) -> Result<crate::value::Value, LangError> {
        match &self.array_data {
            Some(arr) => {
                if index < arr.len() {
                    Ok(arr[index].clone())
                } else {
                    Err(LangError::runtime_error(&format!("Index {} out of bounds", index)))
                }
            },
            None => Err(LangError::runtime_error("Not an array")),
        }
    }
    
    /// Set an element in an array
    pub fn set_element(&mut self, index: usize, value: crate::value::Value) -> Result<(), LangError> {
        match &mut self.array_data {
            Some(arr) => {
                if index < arr.len() {
                    arr[index] = value;
                    Ok(())
                } else {
                    Err(LangError::runtime_error(&format!("Index {} out of bounds", index)))
                }
            },
            None => Err(LangError::runtime_error("Not an array")),
        }
    }
    
    /// Get the function parameters and body
    pub fn get_function(&self) -> Result<(Vec<String>, Box<ASTNode>), LangError> {
        match &self.function_data {
            Some((params, body)) => Ok((params.clone(), body.clone())),
            None => Err(LangError::runtime_error("Not a function")),
        }
    }
}

/// Type alias for a reference-counted complex value
pub type RcComplexValue = RcValue<ComplexValue>;
