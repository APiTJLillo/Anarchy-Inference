// src/value.rs - Modified to integrate with reference counting
// This file contains the Value type with reference counting for complex values

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::ASTNode;
use crate::error::LangError;

// Define RcValue directly here to avoid circular dependency
/// A reference-counted value wrapper
#[derive(Clone)]
pub struct RcValue<T: Clone> {
    /// The inner value wrapped in Rc<RefCell<>>
    inner: Rc<RefCell<T>>,
}

impl<T: Clone + PartialEq> PartialEq for RcValue<T> {
    fn eq(&self, other: &Self) -> bool {
        // Compare the inner values
        *self.inner.borrow() == *other.inner.borrow()
    }
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
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexValueType {
    Object,
    Array,
    Function,
    NativeFunction,
}

/// A complex value that needs reference counting
#[derive(Clone)]
pub struct ComplexValue {
    /// The type of complex value
    pub value_type: ComplexValueType,
    /// Object data (if this is an object)
    pub object_data: Option<HashMap<String, Value>>,
    /// Array data (if this is an array)
    pub array_data: Option<Vec<Value>>,
    /// Function data (if this is a function)
    pub function_data: Option<(Vec<String>, Box<ASTNode>)>,
    /// Native function data (if this is a native function)
    pub native_function_data: Option<Rc<dyn Fn(&mut crate::interpreter::Interpreter, Vec<Value>) -> Result<Value, LangError>>>,
}

// Custom implementation of Debug for ComplexValue to handle function types
impl fmt::Debug for ComplexValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("ComplexValue");
        debug_struct.field("value_type", &self.value_type);
        
        if let Some(obj) = &self.object_data {
            debug_struct.field("object_data", obj);
        }
        
        if let Some(arr) = &self.array_data {
            debug_struct.field("array_data", arr);
        }
        
        if let Some((params, _)) = &self.function_data {
            debug_struct.field("function_params", params);
            debug_struct.field("has_function_body", &true);
        }
        
        if self.native_function_data.is_some() {
            debug_struct.field("has_native_function", &true);
        }
        
        debug_struct.finish()
    }
}

// Custom implementation of PartialEq for ComplexValue to handle ASTNode
impl PartialEq for ComplexValue {
    fn eq(&self, other: &Self) -> bool {
        self.value_type == other.value_type &&
        self.object_data == other.object_data &&
        self.array_data == other.array_data &&
        // Skip comparing function_data since ASTNode doesn't implement PartialEq
        match (&self.function_data, &other.function_data) {
            (None, None) => true,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (Some((self_params, _)), Some((other_params, _))) => {
                // Compare only the parameters, not the ASTNode body
                self_params == other_params
            }
        }
        // Skip comparing native_function_data since functions can't be compared
    }
}

/// A reference-counted complex value
pub type RcComplexValue = RcValue<ComplexValue>;

impl ComplexValue {
    /// Create a new object value
    pub fn new_object() -> Self {
        Self {
            value_type: ComplexValueType::Object,
            object_data: Some(HashMap::new()),
            array_data: None,
            function_data: None,
            native_function_data: None,
        }
    }
    
    /// Create a new array value
    pub fn new_array(elements: Vec<Value>) -> Self {
        Self {
            value_type: ComplexValueType::Array,
            object_data: None,
            array_data: Some(elements),
            function_data: None,
            native_function_data: None,
        }
    }
    
    /// Create a new function value
    pub fn new_function(params: Vec<String>, body: Box<ASTNode>) -> Self {
        Self {
            value_type: ComplexValueType::Function,
            object_data: None,
            array_data: None,
            function_data: Some((params, body)),
            native_function_data: None,
        }
    }
    
    /// Create a new native function value
    pub fn new_native_function<F>(func: F) -> Self 
    where 
        F: Fn(&mut crate::interpreter::Interpreter, Vec<Value>) -> Result<Value, LangError> + 'static
    {
        Self {
            value_type: ComplexValueType::NativeFunction,
            object_data: None,
            array_data: None,
            function_data: None,
            native_function_data: Some(Rc::new(func)),
        }
    }
    
    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Result<Value, LangError> {
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
    pub fn set_property(&mut self, name: String, value: Value) -> Result<(), LangError> {
        match &mut self.object_data {
            Some(obj) => {
                obj.insert(name, value);
                Ok(())
            },
            None => Err(LangError::runtime_error("Not an object")),
        }
    }
    
    /// Get an element from an array
    pub fn get_element(&self, index: usize) -> Result<Value, LangError> {
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
    pub fn set_element(&mut self, index: usize, value: Value) -> Result<(), LangError> {
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
    NativeFunction,
}

/// A value in the language
#[derive(Clone, PartialEq)]
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
    pub fn string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
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
    
    /// Create a native function value
    pub fn native_function<F>(func: F) -> Self 
    where 
        F: Fn(&mut crate::interpreter::Interpreter, Vec<Value>) -> Result<Value, LangError> + 'static
    {
        Self::Complex(RcComplexValue::new(ComplexValue::new_native_function(func)))
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
                    ComplexValueType::NativeFunction => ValueType::NativeFunction,
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
            Self::String(s) => write!(f, "{}", s),
            Self::Complex(complex) => {
                let borrowed = complex.borrow();
                match borrowed.value_type {
                    ComplexValueType::Object => {
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
                    ComplexValueType::NativeFunction => {
                        write!(f, "native_function() {{ ... }}")
                    }
                }
            }
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<LangError> for Value {
    fn from(e: LangError) -> Self {
        Self::String(format!("Error: {}", e))
    }
}
