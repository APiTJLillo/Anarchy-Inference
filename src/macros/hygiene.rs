// src/macros/hygiene.rs - Hygiene mechanisms for macros in Anarchy Inference

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use std::collections::{HashMap, HashSet};

/// Responsible for maintaining hygiene in macro expansions
#[derive(Debug, Clone)]
pub struct MacroHygiene {
    /// Counter for generating unique identifiers
    counter: usize,
    /// Map of original names to renamed identifiers
    renames: HashMap<String, String>,
    /// Set of captured variables from outer scopes
    captures: HashSet<String>,
}

impl MacroHygiene {
    /// Create a new macro hygiene context
    pub fn new() -> Self {
        Self {
            counter: 0,
            renames: HashMap::new(),
            captures: HashSet::new(),
        }
    }
    
    /// Generate a unique identifier
    pub fn generate_unique_name(&mut self, base: &str) -> String {
        self.counter += 1;
        format!("{}_{}", base, self.counter)
    }
    
    /// Register a variable capture
    pub fn capture_variable(&mut self, name: &str) {
        self.captures.insert(name.to_string());
    }
    
    /// Check if a variable is captured
    pub fn is_captured(&self, name: &str) -> bool {
        self.captures.contains(name)
    }
    
    /// Rename a variable
    pub fn rename_variable(&mut self, original: &str) -> String {
        if let Some(renamed) = self.renames.get(original) {
            renamed.clone()
        } else {
            let renamed = self.generate_unique_name(original);
            self.renames.insert(original.to_string(), renamed.clone());
            renamed
        }
    }
    
    /// Get the renamed version of a variable
    pub fn get_renamed(&self, original: &str) -> Option<&String> {
        self.renames.get(original)
    }
    
    /// Apply hygiene to an AST node
    pub fn apply_hygiene(&mut self, node: &ASTNode) -> Result<ASTNode, LangError> {
        match &node.node_type {
            NodeType::Variable(name) => {
                // If the variable is captured, use the original name
                // Otherwise, rename it to ensure hygiene
                let var_name = if self.is_captured(name) {
                    name.clone()
                } else if let Some(renamed) = self.get_renamed(name) {
                    renamed.clone()
                } else {
                    name.clone()
                };
                
                Ok(ASTNode::new(
                    NodeType::Variable(var_name),
                    node.line,
                    node.column,
                ))
            },
            NodeType::Assignment { name, value } => {
                // Rename the variable being assigned
                let var_name = self.rename_variable(name);
                
                // Apply hygiene to the value
                let hygienic_value = self.apply_hygiene(value)?;
                
                Ok(ASTNode::new(
                    NodeType::Assignment {
                        name: var_name,
                        value: Box::new(hygienic_value),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::FunctionDeclaration { name, parameters, body } => {
                // Rename the function
                let func_name = self.rename_variable(name);
                
                // Create a new hygiene context for the function body
                let mut body_hygiene = self.clone();
                
                // Rename parameters
                let mut renamed_params = Vec::new();
                for param in parameters {
                    let renamed = body_hygiene.rename_variable(param);
                    renamed_params.push(renamed);
                }
                
                // Apply hygiene to the body
                let hygienic_body = body_hygiene.apply_hygiene(body)?;
                
                Ok(ASTNode::new(
                    NodeType::FunctionDeclaration {
                        name: func_name,
                        parameters: renamed_params,
                        body: Box::new(hygienic_body),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::Block(nodes) => {
                // Apply hygiene to each node in the block
                let mut hygienic_nodes = Vec::new();
                for node in nodes {
                    let hygienic = self.apply_hygiene(node)?;
                    hygienic_nodes.push(hygienic);
                }
                
                Ok(ASTNode::new(
                    NodeType::Block(hygienic_nodes),
                    node.line,
                    node.column,
                ))
            },
            // For other node types, recursively apply hygiene to children
            NodeType::Binary { left, operator, right } => {
                let hygienic_left = self.apply_hygiene(left)?;
                let hygienic_right = self.apply_hygiene(right)?;
                
                Ok(ASTNode::new(
                    NodeType::Binary {
                        left: Box::new(hygienic_left),
                        operator: operator.clone(),
                        right: Box::new(hygienic_right),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::Unary { operator, operand } => {
                let hygienic_operand = self.apply_hygiene(operand)?;
                
                Ok(ASTNode::new(
                    NodeType::Unary {
                        operator: operator.clone(),
                        operand: Box::new(hygienic_operand),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::FunctionCall { callee, arguments } => {
                let hygienic_callee = self.apply_hygiene(callee)?;
                
                let mut hygienic_args = Vec::new();
                for arg in arguments {
                    let hygienic = self.apply_hygiene(arg)?;
                    hygienic_args.push(hygienic);
                }
                
                Ok(ASTNode::new(
                    NodeType::FunctionCall {
                        callee: Box::new(hygienic_callee),
                        arguments: hygienic_args,
                    },
                    node.line,
                    node.column,
                ))
            },
            // For other node types, just clone them
            _ => Ok(node.clone()),
        }
    }
}
