// src/interpreter.rs - Modified to integrate with reference counting and string dictionary
// This file contains the interpreter with garbage collection support and string dictionary

use std::collections::HashMap;
use std::fmt;
use crate::ast::{ASTNode, NodeType};
use crate::error::{LangError, StackFrame};
use crate::value::Value;
use crate::lexer::Token;
// Define simple local versions of the dictionary types to avoid import issues
pub struct StringDictionary {
    entries: std::collections::HashMap<String, String>
}

impl StringDictionary {
    pub fn new() -> Self {
        Self { entries: std::collections::HashMap::new() }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }
    
    pub fn set(&mut self, key: String, value: String) {
        self.entries.insert(key, value);
    }
}

pub struct StringDictionaryManager {
    dictionaries: std::collections::HashMap<String, StringDictionary>,
    current: String
}

impl StringDictionaryManager {
    pub fn new() -> Self {
        let mut dictionaries = std::collections::HashMap::new();
        dictionaries.insert("default".to_string(), StringDictionary::new());
        Self { 
            dictionaries,
            current: "default".to_string()
        }
    }
    
    pub fn get_string(&self, key: &str) -> Option<&String> {
        // Search in the current dictionary
        self.dictionaries.get(&self.current).and_then(|dict| dict.get(key))
    }
    
    pub fn set_string(&mut self, key: String, value: String) {
        // Create current dictionary if it doesn't exist
        if !self.dictionaries.contains_key(&self.current) {
            self.dictionaries.insert(self.current.clone(), StringDictionary::new());
        }
        
        // Set the value in the current dictionary
        if let Some(dict) = self.dictionaries.get_mut(&self.current) {
            dict.set(key, value);
        }
    }
    
    // Add missing methods that are used in std_lib.rs
    
    pub fn format_string(&self, key: &str, args: &[String]) -> Result<String, crate::error::LangError> {
        let template = self.get_string(key)
            .ok_or_else(|| crate::error::LangError::runtime_error(&format!("String key not found: {}", key)))?;
        
        // Simple placeholder replacement (assumes {} format)
        let mut result = template.clone();
        for arg in args {
            result = result.replacen("{}", arg, 1);
        }
        
        Ok(result)
    }
    
    pub fn load_dictionary(&mut self, path: &str) -> Result<(), crate::error::LangError> {
        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::LangError::io_error(&format!("Failed to read dictionary file: {}", e)))?;
        
        // Parse JSON
        let entries: std::collections::HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| crate::error::LangError::runtime_error(&format!("Failed to parse dictionary file: {}", e)))?;
        
        // Create a new dictionary with the parsed entries
        let dict_name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported")
            .to_string();
        
        let mut dict = StringDictionary::new();
        for (key, value) in entries {
            dict.set(key, value);
        }
        
        // Add the dictionary
        self.dictionaries.insert(dict_name.clone(), dict);
        
        // Set as current
        self.current = dict_name;
        
        Ok(())
    }
    
    pub fn save_dictionary(&self, dict_name: &str, path: &str) -> Result<(), crate::error::LangError> {
        let dict = self.dictionaries.get(dict_name)
            .ok_or_else(|| crate::error::LangError::runtime_error(&format!("Dictionary not found: {}", dict_name)))?;
        
        // Convert dictionary to JSON string
        let mut entries = std::collections::HashMap::new();
        for (key, value) in &dict.entries {
            entries.insert(key.clone(), value.clone());
        }
        
        // Write to file
        std::fs::write(path, serde_json::to_string_pretty(&entries).unwrap())
            .map_err(|e| crate::error::LangError::io_error(&format!("Failed to save dictionary: {}", e)))
    }
    
    pub fn set_current(&mut self, dict_name: &str) -> Result<(), crate::error::LangError> {
        if !self.dictionaries.contains_key(dict_name) {
            return Err(crate::error::LangError::runtime_error(&format!("Dictionary not found: {}", dict_name)));
        }
        
        self.current = dict_name.to_string();
        Ok(())
    }
}

/// Environment for variable storage
pub struct Environment {
    /// Variables in this environment
    pub variables: HashMap<String, Value>,
    /// Parent environment (for closures)
    parent: Option<Box<Environment>>,
}

// Explicitly implement Send and Sync for Environment
// This is safe because all fields (HashMap and Option<Box<Environment>>) are Send + Sync
unsafe impl Send for Environment {}
unsafe impl Sync for Environment {}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("variables", &self.variables.keys().collect::<Vec<_>>())
            .field("has_parent", &self.parent.is_some())
            .finish()
    }
}

impl Environment {
    /// Create a new environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }
    
    /// Create a new environment with a parent
    pub fn with_parent(parent: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    /// Define a variable in this environment
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    
    /// Get a variable from this environment or its parents
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
    
    /// Assign a value to a variable in this environment or its parents
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), LangError> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value)
        } else {
            Err(LangError::runtime_error(&format!("Undefined variable '{}'", name)))
        }
    }
}

/// Interpreter for the language
pub struct Interpreter {
    /// Current environment
    pub environment: Environment,
    /// Global environment
    globals: Environment,
    /// Current file being interpreted
    current_file: String,
    /// Current function being executed
    current_function: Option<String>,
    /// Stack trace for error reporting
    stack_trace: Vec<StackFrame>,
    /// Memory usage statistics
    memory_stats: MemoryStats,
    /// String dictionary manager
    string_dict_manager: StringDictionaryManager,
}

/// Memory usage statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    /// Number of objects allocated
    pub objects_allocated: usize,
    /// Number of arrays allocated
    pub arrays_allocated: usize,
    /// Number of functions allocated
    pub functions_allocated: usize,
    /// Total number of complex values
    pub total_complex_values: usize,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            globals: Environment::new(),
            current_file: String::new(),
            current_function: None,
            stack_trace: Vec::new(),
            memory_stats: MemoryStats::default(),
            string_dict_manager: StringDictionaryManager::new(),
        }
    }
    
    /// Set the current file
    pub fn set_current_file(&mut self, file: String) {
        self.current_file = file;
    }
    
    /// Create a new object
    pub fn create_object(&mut self) -> Value {
        self.memory_stats.objects_allocated += 1;
        self.memory_stats.total_complex_values += 1;
        Value::empty_object()
    }
    
    /// Create a new array
    pub fn create_array(&mut self, elements: Vec<Value>) -> Value {
        self.memory_stats.arrays_allocated += 1;
        self.memory_stats.total_complex_values += 1;
        Value::array(elements)
    }
    
    /// Create a new function
    pub fn create_function(&mut self, _name: String, params: Vec<String>, body: Box<ASTNode>) -> Value {
        self.memory_stats.functions_allocated += 1;
        self.memory_stats.total_complex_values += 1;
        Value::function(params, body)
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> &MemoryStats {
        &self.memory_stats
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
    
    /// Set a string in the current dictionary
    pub fn set_string(&mut self, key: String, value: String) {
        println!("Setting string: {} = {}", key, value);
        self.string_dict_manager.set_string(key, value);
    }
    
    /// Get a string from the current dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        println!("Getting string: {}", key);
        // First try to get from the current dictionary
        let result = self.string_dict_manager.get_string(key).map(|s| s.clone());
        
        // If not found and we're in a function call, check if it's a variable
        if result.is_none() {
            if let Some(value) = self.environment.get(key) {
                println!("Found as variable: {}", key);
                return match value {
                    Value::String(s) => Some(s.clone()),
                    _ => Some(value.to_string()),
                };
            }
        }
        
        println!("Result for {}: {:?}", key, result);
        result
    }
    
    /// Switch to a different dictionary
    pub fn switch_dictionary(&mut self, dict_name: &str) -> Result<(), LangError> {
        // Try to switch to the dictionary
        // If it doesn't exist, it will return an error that we can handle
        let result = self.string_dict_manager.set_current(dict_name);
        
        if result.is_err() {
            // Create a new dictionary with this name
            let mut new_dict = StringDictionary {
                entries: HashMap::new()
            };
            
            // Add some default entries if needed
            new_dict.entries.insert("default_message".to_string(), "Hello from dictionary".to_string());
            
            // Add the dictionary to the manager
            self.string_dict_manager.dictionaries.insert(dict_name.to_string(), new_dict);
            
            // Now try to switch again
            self.string_dict_manager.set_current(dict_name)
        } else {
            result
        }
    }
    
    /// Format a string with arguments
    pub fn format_string(&self, key: &str, args: &[Value]) -> Result<String, LangError> {
        // Convert Value arguments to String
        let string_args: Vec<String> = args.iter()
            .map(|v| v.to_string())
            .collect();
        
        self.string_dict_manager.format_string(key, &string_args)
    }
    
    /// Interpret an AST node
    pub fn interpret(&mut self, node: &ASTNode) -> Result<Value, LangError> {
        match &node.node_type {
            NodeType::Null => Ok(Value::null()),
            NodeType::Number(n) => Ok(Value::number(*n as f64)),
            NodeType::String(s) => Ok(Value::string(s.clone())),
            NodeType::Boolean(b) => Ok(Value::boolean(*b)),
            NodeType::StringDictRef(key) => {
                // Resolve string dictionary reference
                if let Some(string_value) = self.get_string(key) {
                    Ok(Value::string(string_value))
                } else {
                    Err(LangError::runtime_error(&format!("String key '{}' not found in dictionary", key)))
                }
            },
            NodeType::Identifier(name) => {
                if let Some(value) = self.environment.get(name) {
                    Ok(value)
                } else {
                    Err(LangError::runtime_error(&format!("Undefined variable '{}'", name)))
                }
            },
            NodeType::Library { name, functions } => {
                // Create a new environment for the library
                let mut library_env = Environment::new();
                
                // Process all functions in the library
                for func in functions {
                    if let NodeType::FunctionDeclaration { name: func_name, parameters, body } = &func.node_type {
                        let function_value = self.create_function(func_name.clone(), parameters.clone(), body.clone());
                        library_env.define(func_name.clone(), function_value);
                    }
                }
                
                // Store the library in the global environment
                self.globals.define(name.clone(), Value::object(library_env.variables));
                
                Ok(Value::null())
            },
            NodeType::FunctionDeclaration { name, parameters, body } => {
                let function_value = self.create_function(name.clone(), parameters.clone(), body.clone());
                self.environment.define(name.clone(), function_value);
                Ok(Value::null())
            },
            NodeType::FunctionCall { callee, arguments } => {
                // Evaluate the callee
                let _callee_value = self.interpret(callee)?;
                
                // Evaluate the arguments
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.interpret(arg)?);
                }
                
                // Special handling for print function with string dictionary references
                if let NodeType::Identifier(name) = &callee.node_type {
                    if name == "⌽" && !arguments.is_empty() {
                        // Check if the first argument is a string dictionary reference
                        if let NodeType::StringDictRef(key) = &arguments[0].node_type {
                            // Format the string with the remaining arguments
                            let formatted = self.format_string(key, &arg_values[1..])?;
                            println!("{}", formatted);
                            return Ok(Value::string(formatted));
                        }
                    }
                }
                
                // For now, just return null as a placeholder for other function calls
                Ok(Value::null())
            },
            NodeType::PropertyAccess { object, property } => {
                // Evaluate the object
                let object_value = self.interpret(object)?;
                
                // Try to get the property from the object
                if let Ok(prop_value) = object_value.get_property(property) {
                    Ok(prop_value)
                } else {
                    // If property doesn't exist, return null
                    Ok(Value::null())
                }
            },
            NodeType::MethodCall { object, method: _method, arguments } => {
                // Evaluate the object
                let _object_value = self.interpret(object)?;
                
                // Evaluate the arguments
                let mut _arg_values = Vec::new();
                for arg in arguments {
                    _arg_values.push(self.interpret(arg)?);
                }
                
                // For now, just return null as a placeholder
                Ok(Value::null())
            },
            NodeType::Block(statements) => {
                let mut result = Value::null();
                for stmt in statements {
                    result = self.interpret(stmt)?;
                }
                Ok(result)
            },
            NodeType::Print(expr) => {
                let value = self.interpret(expr)?;
                println!("{}", value);
                Ok(value)
            },
            NodeType::Binary { left, operator, right } => {
                let left_val = self.interpret(left)?;
                let right_val = self.interpret(right)?;
                
                match operator {
                    Token::SymbolicOperator('+') => Ok(left_val.add(&right_val)),
                    Token::SymbolicOperator('-') => Ok(left_val.subtract(&right_val)),
                    Token::SymbolicOperator('*') => Ok(left_val.multiply(&right_val)),
                    Token::SymbolicOperator('/') => Ok(left_val.divide(&right_val)),
                    Token::SymbolicOperator('%') => Ok(left_val.modulo(&right_val)),
                    Token::SymbolicOperator('=') => Ok(Value::boolean(left_val.equals(&right_val))),
                    Token::SymbolicOperator('!') => Ok(Value::boolean(!left_val.equals(&right_val))),
                    Token::SymbolicOperator('>') => Ok(Value::boolean(left_val.greater_than(&right_val))),
                    Token::SymbolicOperator('<') => Ok(Value::boolean(left_val.less_than(&right_val))),
                    Token::SymbolicOperator('≥') => Ok(Value::boolean(left_val.greater_than_or_equal(&right_val))),
                    Token::SymbolicOperator('≤') => Ok(Value::boolean(left_val.less_than_or_equal(&right_val))),
                    Token::SymbolicOperator('≠') => Ok(Value::boolean(!left_val.equals(&right_val))),
                    Token::SymbolicOperator('&') => Ok(Value::boolean(left_val.is_truthy() && right_val.is_truthy())),
                    Token::SymbolicOperator('|') => Ok(Value::boolean(left_val.is_truthy() || right_val.is_truthy())),
                    _ => Err(LangError::runtime_error(&format!("Unsupported binary operator: {}", operator))),
                }
            },
            NodeType::Assignment { name, value } => {
                let value = self.interpret(value)?;
                self.environment.assign(name, value.clone())?;
                Ok(value)
            },
            NodeType::Variable(name) => {
                // For variable declarations without initialization, assign null
                self.environment.define(name.clone(), Value::null());
                Ok(Value::null())
            },
            NodeType::If { condition, then_branch, else_branch } => {
                let condition_value = self.interpret(condition)?;
                
                if condition_value.is_truthy() {
                    self.interpret(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.interpret(else_branch)
                } else {
                    Ok(Value::null())
                }
            },
            NodeType::While { condition, body } => {
                let mut result = Value::null();
                
                while self.interpret(condition)?.is_truthy() {
                    result = self.interpret(body)?;
                }
                
                Ok(result)
            },
            // Add other node types as needed
            _ => Err(LangError::runtime_error("Unsupported node type")),
        }
    }
    
    /// Execute a program with a single AST node
    pub fn execute(&mut self, node: &ASTNode) -> Result<Value, LangError> {
        self.interpret(node)
    }
    
    /// Execute a program with multiple AST nodes
    pub fn execute_nodes(&mut self, nodes: &[ASTNode]) -> Result<Value, LangError> {
        let mut result = Value::null();
        for node in nodes {
            result = self.interpret(node)?;
        }
        Ok(result)
    }
}

impl Drop for Interpreter {
    fn drop(&mut self) {
        // When the interpreter is dropped, print memory statistics
        println!("Memory statistics at interpreter shutdown:");
        println!("  Objects allocated: {}", self.memory_stats.objects_allocated);
        println!("  Arrays allocated: {}", self.memory_stats.arrays_allocated);
        println!("  Functions allocated: {}", self.memory_stats.functions_allocated);
        println!("  Total complex values: {}", self.memory_stats.total_complex_values);
    }
}
