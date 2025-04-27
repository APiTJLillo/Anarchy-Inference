// Type checking system module for LSP-like Component
//
// This module provides a comprehensive type checking system for
// Anarchy Inference code, including type inference, type validation,
// and type-related diagnostics.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range};
use crate::language_hub_server::lsp::document::Document;
use crate::language_hub_server::lsp::parser_integration::{AstNode, DiagnosticSeverity};
use crate::language_hub_server::lsp::semantic_analyzer::{TypeInfo, SemanticError};
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager, SymbolInformation};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Type error information
#[derive(Debug, Clone)]
pub struct TypeError {
    /// The range where the error occurred
    pub range: Range,
    
    /// The error message
    pub message: String,
    
    /// The error code (if available)
    pub code: Option<String>,
    
    /// The severity of the error
    pub severity: DiagnosticSeverity,
    
    /// The expected type
    pub expected_type: TypeInfo,
    
    /// The actual type
    pub actual_type: TypeInfo,
}

/// Type environment for tracking types in a scope
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// The parent environment (if any)
    parent: Option<Box<TypeEnvironment>>,
    
    /// The types in this environment
    types: HashMap<String, TypeInfo>,
}

impl TypeEnvironment {
    /// Create a new type environment
    pub fn new() -> Self {
        TypeEnvironment {
            parent: None,
            types: HashMap::new(),
        }
    }
    
    /// Create a new type environment with a parent
    pub fn with_parent(parent: TypeEnvironment) -> Self {
        TypeEnvironment {
            parent: Some(Box::new(parent)),
            types: HashMap::new(),
        }
    }
    
    /// Define a type in this environment
    pub fn define(&mut self, name: &str, type_info: TypeInfo) {
        self.types.insert(name.to_string(), type_info);
    }
    
    /// Look up a type in this environment
    pub fn lookup(&self, name: &str) -> Option<TypeInfo> {
        if let Some(type_info) = self.types.get(name) {
            Some(type_info.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
    
    /// Check if a type is defined in this environment
    pub fn is_defined(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.parent.as_ref().map_or(false, |p| p.is_defined(name))
    }
    
    /// Get all types in this environment
    pub fn get_all_types(&self) -> HashMap<String, TypeInfo> {
        let mut all_types = HashMap::new();
        
        if let Some(parent) = &self.parent {
            all_types.extend(parent.get_all_types());
        }
        
        all_types.extend(self.types.clone());
        
        all_types
    }
}

/// Type checker for Anarchy Inference code
pub struct TypeChecker {
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// The global type environment
    global_env: TypeEnvironment,
    
    /// Cache of type-checked documents
    type_cache: HashMap<String, (i64, HashMap<String, TypeInfo>)>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new(symbol_manager: SharedSymbolManager) -> Self {
        let mut global_env = TypeEnvironment::new();
        
        // Define built-in types
        global_env.define("Number", TypeInfo::Number);
        global_env.define("String", TypeInfo::String);
        global_env.define("Boolean", TypeInfo::Boolean);
        global_env.define("Array", TypeInfo::Array(Box::new(TypeInfo::Any)));
        global_env.define("Object", TypeInfo::Object(HashMap::new()));
        global_env.define("Function", TypeInfo::Function {
            params: Vec::new(),
            return_type: Box::new(TypeInfo::Any),
        });
        
        // Define built-in functions
        let mut math_exports = HashMap::new();
        math_exports.insert("abs".to_string(), TypeInfo::Function {
            params: vec![TypeInfo::Number],
            return_type: Box::new(TypeInfo::Number),
        });
        math_exports.insert("sqrt".to_string(), TypeInfo::Function {
            params: vec![TypeInfo::Number],
            return_type: Box::new(TypeInfo::Number),
        });
        math_exports.insert("max".to_string(), TypeInfo::Function {
            params: vec![TypeInfo::Number, TypeInfo::Number],
            return_type: Box::new(TypeInfo::Number),
        });
        math_exports.insert("min".to_string(), TypeInfo::Function {
            params: vec![TypeInfo::Number, TypeInfo::Number],
            return_type: Box::new(TypeInfo::Number),
        });
        
        global_env.define("Math", TypeInfo::Module(math_exports));
        
        TypeChecker {
            symbol_manager,
            global_env,
            type_cache: HashMap::new(),
        }
    }
    
    /// Type check a document
    pub fn type_check(&mut self, document: &Document, ast: &AstNode) -> Result<Vec<TypeError>, String> {
        // Check if we have already type-checked this version of the document
        if let Some((version, _)) = self.type_cache.get(&document.uri) {
            if *version == document.version {
                // We've already type-checked this version, so just return no errors
                return Ok(Vec::new());
            }
        }
        
        // Create a new type environment for this document
        let mut env = TypeEnvironment::with_parent(self.global_env.clone());
        
        // Type check the AST
        let mut errors = Vec::new();
        let types = self.type_check_node(document, ast, &mut env, &mut errors)?;
        
        // Cache the results
        self.type_cache.insert(document.uri.clone(), (document.version, types));
        
        Ok(errors)
    }
    
    /// Type check an AST node
    fn type_check_node(
        &self,
        document: &Document,
        node: &AstNode,
        env: &mut TypeEnvironment,
        errors: &mut Vec<TypeError>
    ) -> Result<HashMap<String, TypeInfo>, String> {
        let mut types = HashMap::new();
        
        match node.node_type.as_str() {
            "Program" => {
                // Type check all children
                for child in &node.children {
                    let child_types = self.type_check_node(document, child, env, errors)?;
                    types.extend(child_types);
                }
            }
            
            "ModuleDeclaration" => {
                // Get the module name
                let module_name = node.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Create a new environment for the module
                let mut module_env = TypeEnvironment::with_parent(env.clone());
                
                // Type check all children
                let mut module_exports = HashMap::new();
                
                for child in &node.children {
                    let child_types = self.type_check_node(document, child, &mut module_env, errors)?;
                    
                    // Add exported symbols to the module exports
                    if child.node_type == "FunctionDeclaration" || child.node_type == "VariableDeclaration" {
                        if let Some(name) = child.properties.get("name").and_then(|v| v.as_str()) {
                            if let Some(type_info) = child_types.get(name) {
                                module_exports.insert(name.to_string(), type_info.clone());
                            }
                        }
                    }
                    
                    types.extend(child_types);
                }
                
                // Create a module type
                let module_type = TypeInfo::Module(module_exports);
                
                // Add the module to the environment
                env.define(module_name, module_type.clone());
                
                // Add the module to the types
                types.insert(module_name.to_string(), module_type);
            }
            
            "FunctionDeclaration" => {
                // Get the function name
                let function_name = node.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Get the parameters
                let params = node.properties.get("params")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|param| param.as_str())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(Vec::new);
                
                // Create a new environment for the function
                let mut function_env = TypeEnvironment::with_parent(env.clone());
                
                // Add parameters to the environment
                let mut param_types = Vec::new();
                for param in &params {
                    let param_type = TypeInfo::Any; // Default to Any for parameters
                    function_env.define(param, param_type.clone());
                    param_types.push(param_type);
                }
                
                // Type check the function body
                let mut return_type = TypeInfo::Void; // Default to Void
                
                for child in &node.children {
                    if child.node_type == "BlockStatement" {
                        let body_types = self.type_check_node(document, child, &mut function_env, errors)?;
                        
                        // Look for return statements
                        let return_statements = AstUtils::collect_nodes(child, |n| n.node_type == "ReturnStatement");
                        
                        if !return_statements.is_empty() {
                            // Infer return type from return statements
                            let mut return_types = Vec::new();
                            
                            for ret_stmt in return_statements {
                                if let Some(value) = ret_stmt.children.first() {
                                    let value_type = self.infer_type(document, value, &function_env);
                                    return_types.push(value_type);
                                } else {
                                    return_types.push(TypeInfo::Void);
                                }
                            }
                            
                            // Combine return types
                            if return_types.len() == 1 {
                                return_type = return_types[0].clone();
                            } else if !return_types.is_empty() {
                                return_type = TypeInfo::Union(return_types);
                            }
                        }
                    } else {
                        let _ = self.type_check_node(document, child, &mut function_env, errors)?;
                    }
                }
                
                // Create a function type
                let function_type = TypeInfo::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                };
                
                // Add the function to the environment
                env.define(function_name, function_type.clone());
                
                // Add the function to the types
                types.insert(function_name.to_string(), function_type);
            }
            
            "VariableDeclaration" => {
                // Get the variable name
                let variable_name = node.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Get the variable type annotation (if any)
                let type_annotation = node.properties.get("typeAnnotation")
                    .and_then(|v| v.as_str())
                    .map(|t| self.parse_type_annotation(t));
                
                // Type check the initializer
                let mut variable_type = type_annotation.unwrap_or(TypeInfo::Unknown);
                
                if let Some(initializer) = node.children.first() {
                    let initializer_type = self.infer_type(document, initializer, env);
                    
                    // Check if the initializer type is compatible with the annotation
                    if let Some(annotation) = &type_annotation {
                        if !annotation.is_assignable_from(&initializer_type) {
                            errors.push(TypeError {
                                range: initializer.range.clone(),
                                message: format!(
                                    "Type '{}' is not assignable to type '{}'",
                                    initializer_type.to_string(),
                                    annotation.to_string()
                                ),
                                code: Some("T001".to_string()),
                                severity: DiagnosticSeverity::Error,
                                expected_type: annotation.clone(),
                                actual_type: initializer_type.clone(),
                            });
                        }
                    }
                    
                    // If no annotation, use the initializer type
                    if variable_type == TypeInfo::Unknown {
                        variable_type = initializer_type;
                    }
                    
                    // Type check the initializer node
                    let initializer_types = self.type_check_node(document, initializer, env, errors)?;
                    types.extend(initializer_types);
                }
                
                // Add the variable to the environment
                env.define(variable_name, variable_type.clone());
                
                // Add the variable to the types
                types.insert(variable_name.to_string(), variable_type);
            }
            
            "BinaryExpression" => {
                // Type check the operands
                if node.children.len() >= 2 {
                    let left = &node.children[0];
                    let right = &node.children[1];
                    
                    let left_types = self.type_check_node(document, left, env, errors)?;
                    let right_types = self.type_check_node(document, right, env, errors)?;
                    
                    types.extend(left_types);
                    types.extend(right_types);
                    
                    // Get the operator
                    let operator = node.properties.get("operator")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    // Infer the types of the operands
                    let left_type = self.infer_type(document, left, env);
                    let right_type = self.infer_type(document, right, env);
                    
                    // Check type compatibility based on the operator
                    match operator {
                        "+" | "-" | "*" | "/" | "%" => {
                            // Arithmetic operators require number operands
                            if left_type != TypeInfo::Number && left_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: left.range.clone(),
                                    message: format!("Expected number for arithmetic operation, got {}", left_type.to_string()),
                                    code: Some("T002".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Number,
                                    actual_type: left_type,
                                });
                            }
                            
                            if right_type != TypeInfo::Number && right_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: right.range.clone(),
                                    message: format!("Expected number for arithmetic operation, got {}", right_type.to_string()),
                                    code: Some("T002".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Number,
                                    actual_type: right_type,
                                });
                            }
                        }
                        
                        "==" | "!=" | "===" | "!==" => {
                            // Equality operators can compare any types
                            // But warn if comparing different types
                            if left_type != right_type && left_type != TypeInfo::Any && right_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: node.range.clone(),
                                    message: format!("Comparing different types: {} and {}", left_type.to_string(), right_type.to_string()),
                                    code: Some("T003".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                    expected_type: left_type.clone(),
                                    actual_type: right_type,
                                });
                            }
                        }
                        
                        "<" | ">" | "<=" | ">=" => {
                            // Comparison operators require comparable types (number or string)
                            if left_type != TypeInfo::Number && left_type != TypeInfo::String && left_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: left.range.clone(),
                                    message: format!("Expected number or string for comparison, got {}", left_type.to_string()),
                                    code: Some("T004".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Union(vec![TypeInfo::Number, TypeInfo::String]),
                                    actual_type: left_type,
                                });
                            }
                            
                            if right_type != TypeInfo::Number && right_type != TypeInfo::String && right_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: right.range.clone(),
                                    message: format!("Expected number or string for comparison, got {}", right_type.to_string()),
                                    code: Some("T004".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Union(vec![TypeInfo::Number, TypeInfo::String]),
                                    actual_type: right_type.clone(),
                                });
                            }
                            
                            // Warn if comparing different types
                            if left_type != right_type && left_type != TypeInfo::Any && right_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: node.range.clone(),
                                    message: format!("Comparing different types: {} and {}", left_type.to_string(), right_type.to_string()),
                                    code: Some("T003".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                    expected_type: left_type,
                                    actual_type: right_type,
                                });
                            }
                        }
                        
                        "&&" | "||" => {
                            // Logical operators prefer boolean operands
                            if left_type != TypeInfo::Boolean && left_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: left.range.clone(),
                                    message: format!("Expected boolean for logical operation, got {}", left_type.to_string()),
                                    code: Some("T005".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                    expected_type: TypeInfo::Boolean,
                                    actual_type: left_type,
                                });
                            }
                            
                            if right_type != TypeInfo::Boolean && right_type != TypeInfo::Any {
                                errors.push(TypeError {
                                    range: right.range.clone(),
                                    message: format!("Expected boolean for logical operation, got {}", right_type.to_string()),
                                    code: Some("T005".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                    expected_type: TypeInfo::Boolean,
                                    actual_type: right_type,
                                });
                            }
                        }
                        
                        _ => {
                            // Unknown operator
                            errors.push(TypeError {
                                range: node.range.clone(),
                                message: format!("Unknown operator: {}", operator),
                                code: Some("T006".to_string()),
                                severity: DiagnosticSeverity::Error,
                                expected_type: TypeInfo::Unknown,
                                actual_type: TypeInfo::Unknown,
                            });
                        }
                    }
                }
            }
            
            "CallExpression" => {
                // Type check the callee and arguments
                if !node.children.is_empty() {
                    let callee = &node.children[0];
                    let callee_types = self.type_check_node(document, callee, env, errors)?;
                    types.extend(callee_types);
                    
                    // Infer the callee type
                    let callee_type = self.infer_type(document, callee, env);
                    
                    // Check if the callee is callable
                    match callee_type {
                        TypeInfo::Function { params, return_type: _ } => {
                            // Check argument count
                            let args = &node.children[1..];
                            if args.len() != params.len() {
                                errors.push(TypeError {
                                    range: node.range.clone(),
                                    message: format!("Expected {} arguments, got {}", params.len(), args.len()),
                                    code: Some("T007".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Unknown,
                                    actual_type: TypeInfo::Unknown,
                                });
                            } else {
                                // Check argument types
                                for (i, (arg, param_type)) in args.iter().zip(params.iter()).enumerate() {
                                    let arg_types = self.type_check_node(document, arg, env, errors)?;
                                    types.extend(arg_types);
                                    
                                    let arg_type = self.infer_type(document, arg, env);
                                    if !param_type.is_assignable_from(&arg_type) {
                                        errors.push(TypeError {
                                            range: arg.range.clone(),
                                            message: format!("Argument {} has type {}, but {} was expected", i + 1, arg_type.to_string(), param_type.to_string()),
                                            code: Some("T008".to_string()),
                                            severity: DiagnosticSeverity::Error,
                                            expected_type: param_type.clone(),
                                            actual_type: arg_type,
                                        });
                                    }
                                }
                            }
                        }
                        
                        TypeInfo::Any => {
                            // Any type is callable, but we can't check arguments
                            // Type check arguments anyway
                            for arg in &node.children[1..] {
                                let arg_types = self.type_check_node(document, arg, env, errors)?;
                                types.extend(arg_types);
                            }
                        }
                        
                        _ => {
                            errors.push(TypeError {
                                range: callee.range.clone(),
                                message: format!("Type {} is not callable", callee_type.to_string()),
                                code: Some("T009".to_string()),
                                severity: DiagnosticSeverity::Error,
                                expected_type: TypeInfo::Function {
                                    params: Vec::new(),
                                    return_type: Box::new(TypeInfo::Any),
                                },
                                actual_type: callee_type,
                            });
                            
                            // Type check arguments anyway
                            for arg in &node.children[1..] {
                                let arg_types = self.type_check_node(document, arg, env, errors)?;
                                types.extend(arg_types);
                            }
                        }
                    }
                }
            }
            
            "MemberExpression" => {
                // Type check the object
                if !node.children.is_empty() {
                    let object = &node.children[0];
                    let object_types = self.type_check_node(document, object, env, errors)?;
                    types.extend(object_types);
                    
                    // Get the property name
                    let property = node.properties.get("property")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    // Infer the object type
                    let object_type = self.infer_type(document, object, env);
                    
                    // Check if the property exists on the object
                    match &object_type {
                        TypeInfo::Object(props) => {
                            if !props.contains_key(property) {
                                errors.push(TypeError {
                                    range: node.range.clone(),
                                    message: format!("Property '{}' does not exist on type {}", property, object_type.to_string()),
                                    code: Some("T010".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Unknown,
                                    actual_type: TypeInfo::Unknown,
                                });
                            }
                        }
                        
                        TypeInfo::Module(exports) => {
                            if !exports.contains_key(property) {
                                errors.push(TypeError {
                                    range: node.range.clone(),
                                    message: format!("Export '{}' does not exist in module", property),
                                    code: Some("T011".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                    expected_type: TypeInfo::Unknown,
                                    actual_type: TypeInfo::Unknown,
                                });
                            }
                        }
                        
                        TypeInfo::Any => {
                            // Any type can have any property
                        }
                        
                        _ => {
                            errors.push(TypeError {
                                range: object.range.clone(),
                                message: format!("Type {} has no properties", object_type.to_string()),
                                code: Some("T012".to_string()),
                                severity: DiagnosticSeverity::Error,
                                expected_type: TypeInfo::Object(HashMap::new()),
                                actual_type: object_type,
                            });
                        }
                    }
                }
            }
            
            "IfStatement" | "WhileStatement" => {
                // Type check the condition
                if !node.children.is_empty() {
                    let condition = &node.children[0];
                    let condition_types = self.type_check_node(document, condition, env, errors)?;
                    types.extend(condition_types);
                    
                    // Infer the condition type
                    let condition_type = self.infer_type(document, condition, env);
                    
                    // Check if the condition is boolean
                    if condition_type != TypeInfo::Boolean && condition_type != TypeInfo::Any {
                        errors.push(TypeError {
                            range: condition.range.clone(),
                            message: format!("Expected boolean condition, got {}", condition_type.to_string()),
                            code: Some("T013".to_string()),
                            severity: DiagnosticSeverity::Warning,
                            expected_type: TypeInfo::Boolean,
                            actual_type: condition_type,
                        });
                    }
                    
                    // Type check the body
                    for child in &node.children[1..] {
                        let child_types = self.type_check_node(document, child, env, errors)?;
                        types.extend(child_types);
                    }
                }
            }
            
            "ReturnStatement" => {
                // Type check the return value
                if !node.children.is_empty() {
                    let value = &node.children[0];
                    let value_types = self.type_check_node(document, value, env, errors)?;
                    types.extend(value_types);
                    
                    // TODO: Check return type against function return type
                }
            }
            
            "Identifier" => {
                // Get the identifier name
                let name = node.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Check if the identifier is defined
                if !env.is_defined(name) {
                    errors.push(TypeError {
                        range: node.range.clone(),
                        message: format!("Cannot find name '{}'", name),
                        code: Some("T014".to_string()),
                        severity: DiagnosticSeverity::Error,
                        expected_type: TypeInfo::Unknown,
                        actual_type: TypeInfo::Unknown,
                    });
                }
            }
            
            "Literal" => {
                // Nothing to type check for literals
            }
            
            "BlockStatement" => {
                // Create a new environment for the block
                let mut block_env = TypeEnvironment::with_parent(env.clone());
                
                // Type check all children
                for child in &node.children {
                    let child_types = self.type_check_node(document, child, &mut block_env, errors)?;
                    types.extend(child_types);
                }
            }
            
            // Other node types...
            _ => {
                // Recursively type check children
                for child in &node.children {
                    let child_types = self.type_check_node(document, child, env, errors)?;
                    types.extend(child_types);
                }
            }
        }
        
        Ok(types)
    }
    
    /// Infer the type of an AST node
    fn infer_type(&self, document: &Document, node: &AstNode, env: &TypeEnvironment) -> TypeInfo {
        match node.node_type.as_str() {
            "Literal" => {
                // Get the literal type
                let literal_type = node.properties.get("literalType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                match literal_type {
                    "number" => TypeInfo::Number,
                    "string" => TypeInfo::String,
                    "boolean" => TypeInfo::Boolean,
                    "null" => TypeInfo::Null,
                    _ => TypeInfo::Unknown,
                }
            }
            
            "Identifier" => {
                // Get the identifier name
                let name = node.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Look up the type in the environment
                env.lookup(name).unwrap_or(TypeInfo::Unknown)
            }
            
            "BinaryExpression" => {
                // Get the operator
                let operator = node.properties.get("operator")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Get the operand types
                let left_type = if node.children.len() > 0 {
                    self.infer_type(document, &node.children[0], env)
                } else {
                    TypeInfo::Unknown
                };
                
                let right_type = if node.children.len() > 1 {
                    self.infer_type(document, &node.children[1], env)
                } else {
                    TypeInfo::Unknown
                };
                
                match operator {
                    "+" => {
                        // Addition can be number + number or string + string
                        if left_type == TypeInfo::String || right_type == TypeInfo::String {
                            TypeInfo::String
                        } else if left_type == TypeInfo::Number && right_type == TypeInfo::Number {
                            TypeInfo::Number
                        } else {
                            TypeInfo::Unknown
                        }
                    }
                    
                    "-" | "*" | "/" | "%" => {
                        // Arithmetic operators return number
                        if left_type == TypeInfo::Number && right_type == TypeInfo::Number {
                            TypeInfo::Number
                        } else {
                            TypeInfo::Unknown
                        }
                    }
                    
                    "==" | "!=" | "===" | "!==" | "<" | ">" | "<=" | ">=" => {
                        // Comparison operators return boolean
                        TypeInfo::Boolean
                    }
                    
                    "&&" | "||" => {
                        // Logical operators return boolean
                        TypeInfo::Boolean
                    }
                    
                    _ => TypeInfo::Unknown,
                }
            }
            
            "CallExpression" => {
                // Get the callee type
                let callee_type = if node.children.len() > 0 {
                    self.infer_type(document, &node.children[0], env)
                } else {
                    TypeInfo::Unknown
                };
                
                // If the callee is a function, return its return type
                match callee_type {
                    TypeInfo::Function { params: _, return_type } => *return_type,
                    _ => TypeInfo::Unknown,
                }
            }
            
            "MemberExpression" => {
                // Get the object type
                let object_type = if node.children.len() > 0 {
                    self.infer_type(document, &node.children[0], env)
                } else {
                    TypeInfo::Unknown
                };
                
                // Get the property name
                let property = node.properties.get("property")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Get the property type
                match object_type {
                    TypeInfo::Object(props) => {
                        props.get(property).cloned().unwrap_or(TypeInfo::Unknown)
                    }
                    
                    TypeInfo::Module(exports) => {
                        exports.get(property).cloned().unwrap_or(TypeInfo::Unknown)
                    }
                    
                    TypeInfo::Any => TypeInfo::Any,
                    
                    _ => TypeInfo::Unknown,
                }
            }
            
            // Other node types...
            _ => TypeInfo::Unknown,
        }
    }
    
    /// Parse a type annotation string
    fn parse_type_annotation(&self, annotation: &str) -> TypeInfo {
        match annotation {
            "number" => TypeInfo::Number,
            "string" => TypeInfo::String,
            "boolean" => TypeInfo::Boolean,
            "any" => TypeInfo::Any,
            "void" => TypeInfo::Void,
            _ if annotation.ends_with("[]") => {
                // Array type
                let element_type = self.parse_type_annotation(&annotation[..annotation.len() - 2]);
                TypeInfo::Array(Box::new(element_type))
            }
            _ => {
                // Look up the type in the global environment
                self.global_env.lookup(annotation).unwrap_or(TypeInfo::Unknown)
            }
        }
    }
    
    /// Get the type of a symbol
    pub fn get_symbol_type(&self, uri: &str, name: &str, position: Position) -> TypeInfo {
        // Look up the type in the type cache
        if let Some((_, types)) = self.type_cache.get(uri) {
            if let Some(type_info) = types.get(name) {
                return type_info.clone();
            }
        }
        
        // If not found, try to get it from the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        if let Some(symbol) = symbol_manager.find_definition(uri, name, position) {
            if let Some(type_str) = &symbol.symbol_type {
                return self.parse_type_annotation(type_str);
            }
        }
        
        TypeInfo::Unknown
    }
    
    /// Get all types in a document
    pub fn get_document_types(&self, uri: &str) -> HashMap<String, TypeInfo> {
        if let Some((_, types)) = self.type_cache.get(uri) {
            types.clone()
        } else {
            HashMap::new()
        }
    }
}

/// Shared type checker that can be used across threads
pub type SharedTypeChecker = Arc<Mutex<TypeChecker>>;

/// Create a new shared type checker
pub fn create_shared_type_checker(symbol_manager: SharedSymbolManager) -> SharedTypeChecker {
    Arc::new(Mutex::new(TypeChecker::new(symbol_manager)))
}
