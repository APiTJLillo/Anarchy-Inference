// Semantic analyzer module for LSP-like Component
//
// This module provides semantic analysis of Anarchy Inference code,
// including type checking, symbol resolution, and semantic validation.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, Location};
use crate::language_hub_server::lsp::document::Document;
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager, SymbolInformation, SymbolKind};
use crate::language_hub_server::lsp::parser_integration::{AstNode, SyntaxError, DiagnosticSeverity};

/// Semantic error type
#[derive(Debug, Clone)]
pub struct SemanticError {
    /// The range where the error occurred
    pub range: Range,
    
    /// The error message
    pub message: String,
    
    /// The error code (if available)
    pub code: Option<String>,
    
    /// The severity of the error
    pub severity: DiagnosticSeverity,
}

/// Type information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    /// Unknown type
    Unknown,
    
    /// Any type
    Any,
    
    /// Void type
    Void,
    
    /// Boolean type
    Boolean,
    
    /// Number type
    Number,
    
    /// String type
    String,
    
    /// Array type
    Array(Box<TypeInfo>),
    
    /// Object type
    Object(HashMap<String, TypeInfo>),
    
    /// Function type
    Function {
        /// Parameter types
        params: Vec<TypeInfo>,
        
        /// Return type
        return_type: Box<TypeInfo>,
    },
    
    /// Module type
    Module(HashMap<String, TypeInfo>),
    
    /// Union type
    Union(Vec<TypeInfo>),
}

impl TypeInfo {
    /// Check if this type is assignable from another type
    pub fn is_assignable_from(&self, other: &TypeInfo) -> bool {
        match (self, other) {
            // Any type can be assigned from any other type
            (TypeInfo::Any, _) => true,
            
            // Unknown type is not assignable
            (_, TypeInfo::Unknown) => false,
            
            // Same types are assignable
            (a, b) if a == b => true,
            
            // Number can be assigned from Number
            (TypeInfo::Number, TypeInfo::Number) => true,
            
            // String can be assigned from String
            (TypeInfo::String, TypeInfo::String) => true,
            
            // Boolean can be assigned from Boolean
            (TypeInfo::Boolean, TypeInfo::Boolean) => true,
            
            // Array can be assigned if element types are assignable
            (TypeInfo::Array(a), TypeInfo::Array(b)) => a.is_assignable_from(b),
            
            // Object can be assigned if all properties are assignable
            (TypeInfo::Object(a_props), TypeInfo::Object(b_props)) => {
                // All properties in a must be assignable from corresponding properties in b
                for (name, a_type) in a_props {
                    if let Some(b_type) = b_props.get(name) {
                        if !a_type.is_assignable_from(b_type) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            
            // Function can be assigned if parameter and return types are compatible
            (TypeInfo::Function { params: a_params, return_type: a_return }, 
             TypeInfo::Function { params: b_params, return_type: b_return }) => {
                // Return type must be assignable
                if !a_return.is_assignable_from(b_return) {
                    return false;
                }
                
                // Parameter types must be compatible (contravariant)
                if a_params.len() != b_params.len() {
                    return false;
                }
                
                for (a_param, b_param) in a_params.iter().zip(b_params.iter()) {
                    if !b_param.is_assignable_from(a_param) {
                        return false;
                    }
                }
                
                true
            }
            
            // Union type is assignable if any of its members is assignable
            (TypeInfo::Union(types), other) => {
                types.iter().any(|t| t.is_assignable_from(other))
            }
            
            // Other type is assignable to union if it's assignable to any member
            (other, TypeInfo::Union(types)) => {
                types.iter().any(|t| other.is_assignable_from(t))
            }
            
            // Other combinations are not assignable
            _ => false,
        }
    }
    
    /// Get a string representation of the type
    pub fn to_string(&self) -> String {
        match self {
            TypeInfo::Unknown => "unknown".to_string(),
            TypeInfo::Any => "any".to_string(),
            TypeInfo::Void => "void".to_string(),
            TypeInfo::Boolean => "boolean".to_string(),
            TypeInfo::Number => "number".to_string(),
            TypeInfo::String => "string".to_string(),
            TypeInfo::Array(elem_type) => format!("{}[]", elem_type.to_string()),
            TypeInfo::Object(props) => {
                let props_str = props.iter()
                    .map(|(name, type_info)| format!("{}: {}", name, type_info.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", props_str)
            }
            TypeInfo::Function { params, return_type } => {
                let params_str = params.iter()
                    .map(|param| param.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) => {}", params_str, return_type.to_string())
            }
            TypeInfo::Module(exports) => {
                let exports_str = exports.iter()
                    .map(|(name, type_info)| format!("{}: {}", name, type_info.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("module {{ {} }}", exports_str)
            }
            TypeInfo::Union(types) => {
                let types_str = types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("({})", types_str)
            }
        }
    }
}

/// Semantic analyzer for Anarchy Inference code
pub struct SemanticAnalyzer {
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// Type information for symbols
    type_info: HashMap<String, TypeInfo>,
    
    /// Cache of analyzed documents
    analyzed_documents: HashMap<String, (i64, Vec<SemanticError>)>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new(symbol_manager: SharedSymbolManager) -> Self {
        SemanticAnalyzer {
            symbol_manager,
            type_info: HashMap::new(),
            analyzed_documents: HashMap::new(),
        }
    }
    
    /// Analyze a document
    pub fn analyze_document(&mut self, document: &Document, ast: &AstNode) -> Result<Vec<SemanticError>, String> {
        // Check if we have already analyzed this version of the document
        if let Some((version, errors)) = self.analyzed_documents.get(&document.uri) {
            if *version == document.version {
                return Ok(errors.clone());
            }
        }
        
        // Update the symbol table
        {
            let mut symbol_manager = self.symbol_manager.lock().unwrap();
            symbol_manager.update_document(document)?;
        }
        
        // Analyze the AST
        let errors = self.analyze_ast(document, ast);
        
        // Cache the results
        self.analyzed_documents.insert(document.uri.clone(), (document.version, errors.clone()));
        
        Ok(errors)
    }
    
    /// Analyze an AST node
    fn analyze_ast(&mut self, document: &Document, ast: &AstNode) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        
        // Analyze the node based on its type
        match ast.node_type.as_str() {
            "Program" => {
                // Analyze all children
                for child in &ast.children {
                    let child_errors = self.analyze_ast(document, child);
                    errors.extend(child_errors);
                }
            }
            
            "ModuleDeclaration" => {
                // Get the module name
                let module_name = ast.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Analyze all children
                for child in &ast.children {
                    let child_errors = self.analyze_ast(document, child);
                    errors.extend(child_errors);
                }
                
                // Create a module type
                let mut exports = HashMap::new();
                
                // Add exports from children
                for child in &ast.children {
                    if child.node_type == "FunctionDeclaration" || child.node_type == "VariableDeclaration" {
                        let name = child.properties.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        
                        let type_info = self.infer_type(document, child);
                        exports.insert(name.to_string(), type_info);
                    }
                }
                
                // Store the module type
                self.type_info.insert(module_name.to_string(), TypeInfo::Module(exports));
            }
            
            "FunctionDeclaration" => {
                // Get the function name
                let function_name = ast.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Analyze the function body
                for child in &ast.children {
                    let child_errors = self.analyze_ast(document, child);
                    errors.extend(child_errors);
                }
                
                // Infer the function type
                let function_type = self.infer_type(document, ast);
                
                // Store the function type
                self.type_info.insert(function_name.to_string(), function_type);
            }
            
            "VariableDeclaration" => {
                // Get the variable name
                let variable_name = ast.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Analyze the initializer
                if let Some(initializer) = ast.children.first() {
                    let initializer_errors = self.analyze_ast(document, initializer);
                    errors.extend(initializer_errors);
                    
                    // Infer the variable type
                    let variable_type = self.infer_type(document, initializer);
                    
                    // Store the variable type
                    self.type_info.insert(variable_name.to_string(), variable_type);
                } else {
                    // No initializer, assume unknown type
                    self.type_info.insert(variable_name.to_string(), TypeInfo::Unknown);
                }
            }
            
            "BinaryExpression" => {
                // Analyze the left and right operands
                if ast.children.len() >= 2 {
                    let left = &ast.children[0];
                    let right = &ast.children[1];
                    
                    let left_errors = self.analyze_ast(document, left);
                    let right_errors = self.analyze_ast(document, right);
                    
                    errors.extend(left_errors);
                    errors.extend(right_errors);
                    
                    // Get the operator
                    let operator = ast.properties.get("operator")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    // Infer the types of the operands
                    let left_type = self.infer_type(document, left);
                    let right_type = self.infer_type(document, right);
                    
                    // Check type compatibility based on the operator
                    match operator {
                        "+" | "-" | "*" | "/" | "%" => {
                            // Arithmetic operators require number operands
                            if left_type != TypeInfo::Number && left_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: left.range.clone(),
                                    message: format!("Expected number for arithmetic operation, got {}", left_type.to_string()),
                                    code: Some("E1001".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            }
                            
                            if right_type != TypeInfo::Number && right_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: right.range.clone(),
                                    message: format!("Expected number for arithmetic operation, got {}", right_type.to_string()),
                                    code: Some("E1001".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            }
                        }
                        
                        "==" | "!=" | "===" | "!==" => {
                            // Equality operators can compare any types
                            // But warn if comparing different types
                            if left_type != right_type && left_type != TypeInfo::Any && right_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: ast.range.clone(),
                                    message: format!("Comparing different types: {} and {}", left_type.to_string(), right_type.to_string()),
                                    code: Some("W1001".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                        }
                        
                        "<" | ">" | "<=" | ">=" => {
                            // Comparison operators require comparable types (number or string)
                            if left_type != TypeInfo::Number && left_type != TypeInfo::String && left_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: left.range.clone(),
                                    message: format!("Expected number or string for comparison, got {}", left_type.to_string()),
                                    code: Some("E1002".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            }
                            
                            if right_type != TypeInfo::Number && right_type != TypeInfo::String && right_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: right.range.clone(),
                                    message: format!("Expected number or string for comparison, got {}", right_type.to_string()),
                                    code: Some("E1002".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            }
                            
                            // Warn if comparing different types
                            if left_type != right_type && left_type != TypeInfo::Any && right_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: ast.range.clone(),
                                    message: format!("Comparing different types: {} and {}", left_type.to_string(), right_type.to_string()),
                                    code: Some("W1001".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                        }
                        
                        "&&" | "||" => {
                            // Logical operators prefer boolean operands
                            if left_type != TypeInfo::Boolean && left_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: left.range.clone(),
                                    message: format!("Expected boolean for logical operation, got {}", left_type.to_string()),
                                    code: Some("W1002".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                            
                            if right_type != TypeInfo::Boolean && right_type != TypeInfo::Any {
                                errors.push(SemanticError {
                                    range: right.range.clone(),
                                    message: format!("Expected boolean for logical operation, got {}", right_type.to_string()),
                                    code: Some("W1002".to_string()),
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                        }
                        
                        _ => {
                            // Unknown operator
                            errors.push(SemanticError {
                                range: ast.range.clone(),
                                message: format!("Unknown operator: {}", operator),
                                code: Some("E1003".to_string()),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                    }
                }
            }
            
            "CallExpression" => {
                // Analyze the callee and arguments
                if !ast.children.is_empty() {
                    let callee = &ast.children[0];
                    let callee_errors = self.analyze_ast(document, callee);
                    errors.extend(callee_errors);
                    
                    // Infer the callee type
                    let callee_type = self.infer_type(document, callee);
                    
                    // Check if the callee is callable
                    match callee_type {
                        TypeInfo::Function { params, return_type: _ } => {
                            // Check argument count
                            let args = &ast.children[1..];
                            if args.len() != params.len() {
                                errors.push(SemanticError {
                                    range: ast.range.clone(),
                                    message: format!("Expected {} arguments, got {}", params.len(), args.len()),
                                    code: Some("E1004".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            } else {
                                // Check argument types
                                for (i, (arg, param_type)) in args.iter().zip(params.iter()).enumerate() {
                                    let arg_errors = self.analyze_ast(document, arg);
                                    errors.extend(arg_errors);
                                    
                                    let arg_type = self.infer_type(document, arg);
                                    if !param_type.is_assignable_from(&arg_type) {
                                        errors.push(SemanticError {
                                            range: arg.range.clone(),
                                            message: format!("Argument {} has type {}, but {} was expected", i + 1, arg_type.to_string(), param_type.to_string()),
                                            code: Some("E1005".to_string()),
                                            severity: DiagnosticSeverity::Error,
                                        });
                                    }
                                }
                            }
                        }
                        
                        TypeInfo::Any => {
                            // Any type is callable, but we can't check arguments
                            // Analyze arguments anyway
                            for arg in &ast.children[1..] {
                                let arg_errors = self.analyze_ast(document, arg);
                                errors.extend(arg_errors);
                            }
                        }
                        
                        _ => {
                            errors.push(SemanticError {
                                range: callee.range.clone(),
                                message: format!("Type {} is not callable", callee_type.to_string()),
                                code: Some("E1006".to_string()),
                                severity: DiagnosticSeverity::Error,
                            });
                            
                            // Analyze arguments anyway
                            for arg in &ast.children[1..] {
                                let arg_errors = self.analyze_ast(document, arg);
                                errors.extend(arg_errors);
                            }
                        }
                    }
                }
            }
            
            "MemberExpression" => {
                // Analyze the object
                if !ast.children.is_empty() {
                    let object = &ast.children[0];
                    let object_errors = self.analyze_ast(document, object);
                    errors.extend(object_errors);
                    
                    // Get the property name
                    let property = ast.properties.get("property")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    // Infer the object type
                    let object_type = self.infer_type(document, object);
                    
                    // Check if the property exists on the object
                    match &object_type {
                        TypeInfo::Object(props) => {
                            if !props.contains_key(property) {
                                errors.push(SemanticError {
                                    range: ast.range.clone(),
                                    message: format!("Property '{}' does not exist on type {}", property, object_type.to_string()),
                                    code: Some("E1007".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            }
                        }
                        
                        TypeInfo::Module(exports) => {
                            if !exports.contains_key(property) {
                                errors.push(SemanticError {
                                    range: ast.range.clone(),
                                    message: format!("Export '{}' does not exist in module", property),
                                    code: Some("E1008".to_string()),
                                    severity: DiagnosticSeverity::Error,
                                });
                            }
                        }
                        
                        TypeInfo::Any => {
                            // Any type can have any property
                        }
                        
                        _ => {
                            errors.push(SemanticError {
                                range: object.range.clone(),
                                message: format!("Type {} has no properties", object_type.to_string()),
                                code: Some("E1009".to_string()),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                    }
                }
            }
            
            "IfStatement" | "WhileStatement" => {
                // Analyze the condition
                if !ast.children.is_empty() {
                    let condition = &ast.children[0];
                    let condition_errors = self.analyze_ast(document, condition);
                    errors.extend(condition_errors);
                    
                    // Infer the condition type
                    let condition_type = self.infer_type(document, condition);
                    
                    // Check if the condition is boolean
                    if condition_type != TypeInfo::Boolean && condition_type != TypeInfo::Any {
                        errors.push(SemanticError {
                            range: condition.range.clone(),
                            message: format!("Expected boolean condition, got {}", condition_type.to_string()),
                            code: Some("W1003".to_string()),
                            severity: DiagnosticSeverity::Warning,
                        });
                    }
                    
                    // Analyze the body
                    for child in &ast.children[1..] {
                        let child_errors = self.analyze_ast(document, child);
                        errors.extend(child_errors);
                    }
                }
            }
            
            "ReturnStatement" => {
                // Analyze the return value
                if !ast.children.is_empty() {
                    let value = &ast.children[0];
                    let value_errors = self.analyze_ast(document, value);
                    errors.extend(value_errors);
                    
                    // TODO: Check return type against function return type
                }
            }
            
            "Identifier" => {
                // Get the identifier name
                let name = ast.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Check if the identifier is defined
                let symbol_manager = self.symbol_manager.lock().unwrap();
                let position = Position {
                    line: ast.range.start.line,
                    character: ast.range.start.character,
                };
                
                if symbol_manager.find_definition(&document.uri, name, position).is_none() {
                    errors.push(SemanticError {
                        range: ast.range.clone(),
                        message: format!("Undefined identifier: {}", name),
                        code: Some("E1010".to_string()),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            }
            
            // Other node types...
            _ => {
                // Recursively analyze children
                for child in &ast.children {
                    let child_errors = self.analyze_ast(document, child);
                    errors.extend(child_errors);
                }
            }
        }
        
        errors
    }
    
    /// Infer the type of an AST node
    fn infer_type(&self, document: &Document, ast: &AstNode) -> TypeInfo {
        match ast.node_type.as_str() {
            "Literal" => {
                // Get the literal type
                let literal_type = ast.properties.get("literalType")
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
                let name = ast.properties.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Look up the type in our type info
                if let Some(type_info) = self.type_info.get(name) {
                    return type_info.clone();
                }
                
                // If not found, try to get it from the symbol manager
                let symbol_manager = self.symbol_manager.lock().unwrap();
                let position = Position {
                    line: ast.range.start.line,
                    character: ast.range.start.character,
                };
                
                if let Some(symbol) = symbol_manager.find_definition(&document.uri, name, position) {
                    if let Some(type_str) = &symbol.symbol_type {
                        match type_str.as_str() {
                            "number" => return TypeInfo::Number,
                            "string" => return TypeInfo::String,
                            "boolean" => return TypeInfo::Boolean,
                            _ => {}
                        }
                    }
                }
                
                TypeInfo::Unknown
            }
            
            "BinaryExpression" => {
                // Get the operator
                let operator = ast.properties.get("operator")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                // Get the operand types
                let left_type = if ast.children.len() > 0 {
                    self.infer_type(document, &ast.children[0])
                } else {
                    TypeInfo::Unknown
                };
                
                let right_type = if ast.children.len() > 1 {
                    self.infer_type(document, &ast.children[1])
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
                let callee_type = if ast.children.len() > 0 {
                    self.infer_type(document, &ast.children[0])
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
                let object_type = if ast.children.len() > 0 {
                    self.infer_type(document, &ast.children[0])
                } else {
                    TypeInfo::Unknown
                };
                
                // Get the property name
                let property = ast.properties.get("property")
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
            
            "FunctionDeclaration" => {
                // Get parameter types
                let mut params = Vec::new();
                
                // In a real implementation, we would extract parameter types from the AST
                // For now, we'll assume all parameters are of type Any
                
                // Get the return type
                // In a real implementation, we would analyze the function body
                // For now, we'll assume the return type is Any
                
                TypeInfo::Function {
                    params,
                    return_type: Box::new(TypeInfo::Any),
                }
            }
            
            // Other node types...
            _ => TypeInfo::Unknown,
        }
    }
    
    /// Get the type of a symbol
    pub fn get_symbol_type(&self, uri: &str, name: &str, position: Position) -> TypeInfo {
        // Look up the type in our type info
        if let Some(type_info) = self.type_info.get(name) {
            return type_info.clone();
        }
        
        // If not found, try to get it from the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        if let Some(symbol) = symbol_manager.find_definition(uri, name, position) {
            if let Some(type_str) = &symbol.symbol_type {
                match type_str.as_str() {
                    "number" => return TypeInfo::Number,
                    "string" => return TypeInfo::String,
                    "boolean" => return TypeInfo::Boolean,
                    _ => {}
                }
            }
        }
        
        TypeInfo::Unknown
    }
}

/// Shared semantic analyzer that can be used across threads
pub type SharedSemanticAnalyzer = Arc<Mutex<SemanticAnalyzer>>;

/// Create a new shared semantic analyzer
pub fn create_shared_semantic_analyzer(symbol_manager: SharedSymbolManager) -> SharedSemanticAnalyzer {
    Arc::new(Mutex::new(SemanticAnalyzer::new(symbol_manager)))
}
