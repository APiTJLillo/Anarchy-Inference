// src/interpreter.rs - Modified to include garbage collection support
// This file contains the interpreter for the language

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use crate::value::Value;
use crate::core::string_dict::{StringDictionary, StringDictionaryManager};
use crate::core::gc_types::GcStats;
use crate::gc::managed::GcValueImpl;
use crate::core::{GarbageCollector, GarbageCollected};
use crate::core::value::GcValue;

/// Environment for variable storage
#[derive(Debug, Clone)]
pub struct Environment {
    // Variable storage
    variables: HashMap<String, Value>,
    // Parent environment for scoping
    parent: Option<Arc<Environment>>,
    // Current file being executed
    current_file: String,
}

/// Interpreter for the language
// #[derive(Debug)] // Temporarily removed due to trait object
pub struct Interpreter {
    // Global environment
    global_env: Environment,
    // Current environment
    current_env: Arc<Environment>,
    // String dictionary manager
    string_dict_manager: StringDictionaryManager,
    // Garbage collector
    garbage_collector: Option<Box<dyn GarbageCollector>>,
}

impl Environment {
    /// Create a new environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
            current_file: String::new(),
        }
    }
    
    /// Create a new environment with a parent
    pub fn with_parent(parent: Arc<Environment>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(parent),
            current_file: parent.current_file.clone(),
        }
    }
    
    /// Get a variable from the environment
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
    
    /// Set a variable in the environment
    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    
    /// Check if a variable exists in the environment
    pub fn has(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            true
        } else if let Some(parent) = &self.parent {
            parent.has(name)
        } else {
            false
        }
    }
    
    /// Set the current file
    pub fn set_current_file(&mut self, file: String) {
        self.current_file = file;
    }
    
    /// Get the current file
    pub fn current_file(&self) -> &str {
        &self.current_file
    }
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        let global_env = Environment::new();
        let current_env = Arc::new(global_env.clone());
        
        let mut interpreter = Self {
            global_env,
            current_env,
            string_dict_manager: StringDictionaryManager::new(),
            garbage_collector: None,
        };
        
        // Initialize the garbage collector
        interpreter.init_garbage_collector();
        
        interpreter
    }
    
    /// Execute a list of AST nodes
    pub fn execute_nodes(&mut self, nodes: &[ASTNode]) -> Result<Value, LangError> {
        let mut result = Value::Null;
        
        for node in nodes {
            result = self.execute_node(node)?;
        }
        
        Ok(result)
    }
    
    /// Execute a single AST node
    pub fn execute_node(&mut self, node: &ASTNode) -> Result<Value, LangError> {
        match &node.node_type {
            NodeType::Number(n) => Ok(Value::Number((*n) as f64)),
            NodeType::Boolean(b) => Ok(Value::Boolean(*b)),
            NodeType::String(s) => Ok(Value::String(s.clone())),
            NodeType::Null => Ok(Value::Null),
            NodeType::Variable(name) => {
                let value = self.current_env.get(name)
                    .ok_or_else(|| LangError::runtime_error(&format!("Variable '{}' not found", name)))?;
                Ok(value)
            },
            NodeType::Assignment { name, value } => {
                let value = self.execute_node(value)?;
                
                // Clone the current environment for mutation
                let mut env = (*self.current_env).clone();
                env.set(name.clone(), value.clone());
                self.current_env = Arc::new(env);
                
                Ok(value)
            },
            NodeType::FunctionDeclaration { name, parameters, body } => {
                // Create a function value
                let function_value = GcValueImpl::new_function(
                    name.clone(),
                    parameters.clone(),
                    body.clone(),
                    self.current_env.clone(),
                );
                
                // Allocate in the garbage collector
                let gc_value = self.allocate_value(function_value);
                
                // Store in the environment
                let mut env = (*self.current_env).clone();
                env.set(name.clone(), Value::Complex(gc_value.clone()));
                self.current_env = Arc::new(env);
                
                Ok(Value::Complex(gc_value))
            },
            NodeType::FunctionCall { callee, arguments } => {
                let function_value = self.execute_node(callee)?;
                
                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.execute_node(arg)?);
                }
                
                // Get function parameters and body
                let (parameters, body) = function_value.get_function()?;
                
                // Check argument count
                if arg_values.len() != parameters.len() {
                    return Err(LangError::runtime_error(&format!(
                        "Function expected {} arguments, got {}",
                        parameters.len(), arg_values.len()
                    )));
                }
                
                // Create a new environment for the function call
                let mut call_env = Environment::with_parent(self.current_env.clone());
                
                // Bind arguments to parameters
                for (param, arg) in parameters.iter().zip(arg_values) {
                    call_env.set(param.clone(), arg);
                }
                
                // Execute the function body in the new environment
                let old_env = self.current_env.clone();
                self.current_env = Arc::new(call_env);
                let result = self.execute_node(&body);
                self.current_env = old_env;
                
                result
            },
            NodeType::Return(value) => {
                self.execute_node(value)
            },
            NodeType::Print(value) => {
                let result = self.execute_node(value)?;
                println!("{}", result);
                Ok(result)
            },
            NodeType::Block(nodes) => {
                let mut result = Value::Null;
                
                // Create a new environment for the block
                let block_env = Environment::with_parent(self.current_env.clone());
                let old_env = self.current_env.clone();
                self.current_env = Arc::new(block_env);
                
                // Execute each node in the block
                for node in nodes {
                    result = self.execute_node(node)?;
                }
                
                // Restore the old environment
                self.current_env = old_env;
                
                Ok(result)
            },
            NodeType::If { condition, then_branch, else_branch } => {
                let condition_value = self.execute_node(condition)?;
                
                match condition_value {
                    Value::Boolean(true) => self.execute_node(then_branch),
                    Value::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            self.execute_node(else_branch)
                        } else {
                            Ok(Value::Null)
                        }
                    },
                    _ => Err(LangError::runtime_error("Condition must be a boolean")),
                }
            },
            NodeType::Binary { operator, left, right } => {
                let left_value = self.execute_node(left)?;
                let right_value = self.execute_node(right)?;
                
                match op.as_str() {
                    "+" => self.add(left_value, right_value),
                    "-" => self.subtract(left_value, right_value),
                    "*" => self.multiply(left_value, right_value),
                    "/" => self.divide(left_value, right_value),
                    "==" => self.equals(left_value, right_value),
                    "!=" => self.not_equals(left_value, right_value),
                    "<" => self.less_than(left_value, right_value),
                    "<=" => self.less_than_equals(left_value, right_value),
                    ">" => self.greater_than(left_value, right_value),
                    ">=" => self.greater_than_equals(left_value, right_value),
                    "&&" => self.logical_and(left_value, right_value),
                    "||" => self.logical_or(left_value, right_value),
                    _ => Err(LangError::runtime_error(&format!("Unknown operator: {}", op))),
                }
            },
            NodeType::Unary { operator, operand } => {
                let operand_value = self.execute_node(operand)?;
                
                match op.as_str() {
                    "-" => self.negate(operand_value),
                    "!" => self.logical_not(operand_value),
                    _ => Err(LangError::runtime_error(&format!("Unknown operator: {}", op))),
                }
            },
            /* NodeType::ObjectLiteral(properties) => {
                // Create a new object
                let mut object = HashMap::new();
                
                // Add properties
                for (key, value_node) in properties {
                    let value = self.execute_node(value_node)?;
                    object.insert(key.clone(), value);
                }
                
                // Create a GC-managed object
                let object_value = GcValueImpl::new_object();
                let gc_value = self.allocate_value(object_value);
                
                // Set properties
                for (key, value) in object {
                    gc_value.set_property(key, value)?;
                }
                
                Ok(Value::Complex(gc_value))
            }, */
            /* NodeType::ArrayLiteral(elements) => {
                // Evaluate elements
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.execute_node(element)?);
                }
                
                // Create a GC-managed array
                let array_value = GcValueImpl::new_array(values);
                let gc_value = self.allocate_value(array_value);
                
                Ok(Value::Complex(gc_value))
            }, */
            NodeType::PropertyAccess { object, property } => {
                let object_value = self.execute_node(object)?;
                object_value.get_property(property)
            },
            /* NodeType::PropertyAssignment { object, property, value } => {
                let object_value = self.execute_node(object)?;
                let value = self.execute_node(value)?;
                
                object_value.set_property(property.clone(), value.clone())?;
                
                Ok(value)
            }, */
            /* NodeType::IndexAccess { array, index } => {
                let array_value = self.execute_node(array)?;
                let index_value = self.execute_node(index)?;
                
                match index_value {
                    Value::Number(n) => {
                        let index = n as usize;
                        array_value.get_element(index)
                    },
                    _ => Err(LangError::runtime_error("Array index must be a number")),
                }
            }, */
            /* NodeType::IndexAssignment { array, index, value } => {
                let array_value = self.execute_node(array)?;
                let index_value = self.execute_node(index)?;
                let value = self.execute_node(value)?;
                
                match index_value {
                    Value::Number(n) => {
                        let index = n as usize;
                        array_value.set_element(index, value.clone())?;
                        Ok(value)
                    },
                    _ => Err(LangError::runtime_error("Array index must be a number")),
                }
            }, */
            NodeType::StringDictRef(key) => {
                let value = self.string_dict_manager.get_string(key)
                    .ok_or_else(|| LangError::runtime_error(&format!("String key '{}' not found in dictionary", key)))?;
                
                Ok(Value::String(value.clone()))
            },
            /* NodeType::StringDictFormat { key, arguments } => {
                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in arguments {
                    let value = self.execute_node(arg)?;
                    arg_values.push(value.to_string());
                }
                
                // Format the string
                let result = self.string_dict_manager.format_string(key, &arg_values)?;
                
                Ok(Value::String(result))
            }, */
            NodeType::UserInput => {
                // Read user input
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)
                    .map_err(|e| LangError::io_error(&format!("Failed to read user input: {}", e)))?;
                
                // Trim newline
                let input = input.trim().to_string();
                
                Ok(Value::String(input))
            },
            // Add other node types as needed
        }
    }
    
    /// Set the current file
    pub fn set_current_file(&mut self, file: String) {
        self.global_env.set_current_file(file.clone());
        
        // Clone the current environment for mutation
        let mut env = (*self.current_env).clone();
        env.set_current_file(file);
        self.current_env = Arc::new(env);
    }
    
    /// Get the string dictionary manager
    pub fn get_string_dict_manager(&self) -> &StringDictionaryManager {
        &self.string_dict_manager
    }
    
    /// Get a mutable reference to the string dictionary manager
    pub fn get_string_dict_manager_mut(&mut self) -> &mut StringDictionaryManager {
        &mut self.string_dict_manager
    }
    
    /// Load a string dictionary from a file
    pub fn load_string_dictionary(&mut self, path: &str) -> Result<(), LangError> {
        self.string_dict_manager.load_dictionary(path)
    }
    
    // Binary operations
    
    fn add(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(LangError::runtime_error("Cannot add values of different types")),
        }
    }
    
    fn subtract(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(LangError::runtime_error("Cannot subtract non-numeric values")),
        }
    }
    
    fn multiply(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(LangError::runtime_error("Cannot multiply non-numeric values")),
        }
    }
    
    fn divide(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    Err(LangError::runtime_error("Division by zero"))
                } else {
                    Ok(Value::Number(a / b))
                }
            },
            _ => Err(LangError::runtime_error("Cannot divide non-numeric values")),
        }
    }
    
    fn equals(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
            (Value::Null, Value::Null) => Ok(Value::Boolean(true)),
            _ => Ok(Value::Boolean(false)),
        }
    }
    
    fn not_equals(&self, left: Value, right: Value) -> Result<Value, LangError> {
        let result = self.equals(left, right)?;
        match result {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(LangError::runtime_error("Expected boolean result")),
        }
    }
    
    fn less_than(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(LangError::runtime_error("Cannot compare non-numeric values")),
        }
    }
    
    fn less_than_equals(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(LangError::runtime_error("Cannot compare non-numeric values")),
        }
    }
    
    fn greater_than(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(LangError::runtime_error("Cannot compare non-numeric values")),
        }
    }
    
    fn greater_than_equals(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(LangError::runtime_error("Cannot compare non-numeric values")),
        }
    }
    
    fn logical_and(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
            _ => Err(LangError::runtime_error("Cannot perform logical AND on non-boolean values")),
        }
    }
    
    fn logical_or(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
            _ => Err(LangError::runtime_error("Cannot perform logical OR on non-boolean values")),
        }
    }
    
    // Unary operations
    
    fn negate(&self, operand: Value) -> Result<Value, LangError> {
        match operand {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(LangError::runtime_error("Cannot negate non-numeric value")),
        }
    }
    
    fn logical_not(&self, operand: Value) -> Result<Value, LangError> {
        match operand {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(LangError::runtime_error("Cannot perform logical NOT on non-boolean value")),
        }
    }
}

// Implement GarbageCollected for Interpreter
impl GarbageCollected for Interpreter {
    fn init_garbage_collector(&mut self) {
        self.garbage_collector = Some(GarbageCollector::new());
    }
    
    fn collect_garbage(&mut self) {
        if let Some(gc) = &self.garbage_collector {
            gc.collect();
        }
    }
    
    fn get_gc_stats(&self) -> GcStats {
        if let Some(gc) = &self.garbage_collector {
            gc.get_stats()
        } else {
            GcStats::default()
        }
    }
    
    fn allocate_value(&mut self, value: GcValueImpl) -> GcValue {
        if let Some(gc) = &self.garbage_collector {
            gc.allocate(value)
        } else {
            // Initialize GC if not already done
            self.init_garbage_collector();
            self.garbage_collector.as_ref().unwrap().allocate(value)
        }
    }
}

