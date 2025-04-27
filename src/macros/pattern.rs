// src/macros/pattern.rs - Pattern matching for macros in Anarchy Inference

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use std::collections::HashMap;

/// Represents a pattern for matching macro arguments
#[derive(Debug, Clone)]
pub struct MacroPattern {
    /// Variables in the pattern
    pub variables: Vec<String>,
    /// Pattern structure
    pub pattern: ASTNode,
}

impl MacroPattern {
    /// Create a new macro pattern
    pub fn new(variables: Vec<String>, pattern: ASTNode) -> Self {
        Self {
            variables,
            pattern,
        }
    }
    
    /// Create a macro pattern from an AST node
    pub fn from_ast(node: &ASTNode) -> Result<Self, LangError> {
        match &node.node_type {
            NodeType::MacroPattern { variables, pattern } => {
                Ok(Self {
                    variables: variables.clone(),
                    pattern: (**pattern).clone(),
                })
            },
            _ => {
                // For simple patterns, extract variables from parameter list
                match &node.node_type {
                    NodeType::FunctionCall { callee, arguments } => {
                        if let NodeType::Identifier(name) = &callee.node_type {
                            let mut variables = Vec::new();
                            for arg in arguments {
                                if let NodeType::Identifier(var_name) = &arg.node_type {
                                    variables.push(var_name.clone());
                                } else {
                                    return Err(LangError::runtime_error(
                                        "Pattern parameters must be identifiers"
                                    ));
                                }
                            }
                            
                            Ok(Self {
                                variables,
                                pattern: node.clone(),
                            })
                        } else {
                            Err(LangError::runtime_error(
                                "Pattern callee must be an identifier"
                            ))
                        }
                    },
                    _ => Err(LangError::runtime_error(
                        "Invalid macro pattern, expected MacroPattern or function call"
                    )),
                }
            }
        }
    }
    
    /// Match a pattern against arguments
    pub fn match_arguments(&self, arguments: &[ASTNode]) -> Result<HashMap<String, ASTNode>, LangError> {
        // Simple parameter matching for now
        if arguments.len() != self.variables.len() {
            return Err(LangError::runtime_error(&format!(
                "Macro expected {} arguments, got {}",
                self.variables.len(), arguments.len()
            )));
        }
        
        let mut bindings = HashMap::new();
        for (i, var) in self.variables.iter().enumerate() {
            bindings.insert(var.clone(), arguments[i].clone());
        }
        
        Ok(bindings)
    }
    
    /// Match a pattern against a node
    pub fn match_node(&self, node: &ASTNode) -> Result<HashMap<String, ASTNode>, LangError> {
        // For more complex pattern matching in the future
        // Currently just delegates to match_arguments for function call patterns
        match &self.pattern.node_type {
            NodeType::FunctionCall { callee: _, arguments: _ } => {
                // Extract arguments from the node
                match &node.node_type {
                    NodeType::FunctionCall { callee: _, arguments } => {
                        self.match_arguments(arguments)
                    },
                    _ => Err(LangError::runtime_error(
                        "Pattern expects a function call"
                    )),
                }
            },
            _ => Err(LangError::runtime_error(
                "Complex pattern matching not yet implemented"
            )),
        }
    }
}
