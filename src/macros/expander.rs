// src/macros/expander.rs - Macro expansion system for Anarchy Inference

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use super::MacroDefinition;
use std::collections::HashMap;

/// Responsible for expanding macros in the AST
#[derive(Debug, Clone)]
pub struct MacroExpander {
    /// Map of macro names to their definitions
    macros: HashMap<String, MacroDefinition>,
    /// Maximum expansion depth to prevent infinite recursion
    max_depth: usize,
    /// Current expansion depth
    current_depth: usize,
}

impl MacroExpander {
    /// Create a new macro expander
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            max_depth: 100, // Default max depth
            current_depth: 0,
        }
    }
    
    /// Create a new macro expander with a specific max depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            macros: HashMap::new(),
            max_depth,
            current_depth: 0,
        }
    }
    
    /// Register a macro definition
    pub fn register_macro(&mut self, definition: MacroDefinition) {
        self.macros.insert(definition.name.clone(), definition);
    }
    
    /// Get a macro definition by name
    pub fn get_macro(&self, name: &str) -> Option<&MacroDefinition> {
        self.macros.get(name)
    }
    
    /// Expand a macro invocation
    pub fn expand_macro(&self, name: &str, arguments: &[ASTNode]) -> Result<ASTNode, LangError> {
        // Check expansion depth
        if self.current_depth >= self.max_depth {
            return Err(LangError::runtime_error(&format!(
                "Maximum macro expansion depth ({}) exceeded, possible infinite recursion",
                self.max_depth
            )));
        }
        
        // Get the macro definition
        let definition = self.get_macro(name)
            .ok_or_else(|| LangError::runtime_error(&format!("Macro '{}' not found", name)))?;
        
        // Create a new expander with increased depth
        let mut expander = self.clone();
        expander.current_depth += 1;
        
        // Expand the macro
        definition.expand(arguments, &expander)
    }
    
    /// Expand all macros in an AST
    pub fn expand_all(&self, node: &ASTNode) -> Result<ASTNode, LangError> {
        match &node.node_type {
            NodeType::MacroInvocation { name, arguments } => {
                // Expand arguments first
                let mut expanded_args = Vec::new();
                for arg in arguments {
                    let expanded = self.expand_all(arg)?;
                    expanded_args.push(expanded);
                }
                
                // Expand the macro
                let expanded = self.expand_macro(name, &expanded_args)?;
                
                // Wrap in a MacroExpansion node for debugging
                Ok(ASTNode::new(
                    NodeType::MacroExpansion {
                        original: Box::new(node.clone()),
                        expanded: Box::new(expanded),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::Block(nodes) => {
                // Expand each node in the block
                let mut expanded_nodes = Vec::new();
                for node in nodes {
                    let expanded = self.expand_all(node)?;
                    expanded_nodes.push(expanded);
                }
                
                Ok(ASTNode::new(
                    NodeType::Block(expanded_nodes),
                    node.line,
                    node.column,
                ))
            },
            NodeType::Binary { left, operator, right } => {
                let expanded_left = self.expand_all(left)?;
                let expanded_right = self.expand_all(right)?;
                
                Ok(ASTNode::new(
                    NodeType::Binary {
                        left: Box::new(expanded_left),
                        operator: operator.clone(),
                        right: Box::new(expanded_right),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::Unary { operator, operand } => {
                let expanded_operand = self.expand_all(operand)?;
                
                Ok(ASTNode::new(
                    NodeType::Unary {
                        operator: operator.clone(),
                        operand: Box::new(expanded_operand),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::FunctionCall { callee, arguments } => {
                let expanded_callee = self.expand_all(callee)?;
                
                let mut expanded_args = Vec::new();
                for arg in arguments {
                    let expanded = self.expand_all(arg)?;
                    expanded_args.push(expanded);
                }
                
                Ok(ASTNode::new(
                    NodeType::FunctionCall {
                        callee: Box::new(expanded_callee),
                        arguments: expanded_args,
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::If { condition, then_branch, else_branch } => {
                let expanded_condition = self.expand_all(condition)?;
                let expanded_then = self.expand_all(then_branch)?;
                
                let expanded_else = if let Some(else_branch) = else_branch {
                    Some(Box::new(self.expand_all(else_branch)?))
                } else {
                    None
                };
                
                Ok(ASTNode::new(
                    NodeType::If {
                        condition: Box::new(expanded_condition),
                        then_branch: Box::new(expanded_then),
                        else_branch: expanded_else,
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::While { condition, body } => {
                let expanded_condition = self.expand_all(condition)?;
                let expanded_body = self.expand_all(body)?;
                
                Ok(ASTNode::new(
                    NodeType::While {
                        condition: Box::new(expanded_condition),
                        body: Box::new(expanded_body),
                    },
                    node.line,
                    node.column,
                ))
            },
            NodeType::For { initializer, condition, increment, body } => {
                let expanded_initializer = self.expand_all(initializer)?;
                let expanded_condition = self.expand_all(condition)?;
                let expanded_increment = self.expand_all(increment)?;
                let expanded_body = self.expand_all(body)?;
                
                Ok(ASTNode::new(
                    NodeType::For {
                        initializer: Box::new(expanded_initializer),
                        condition: Box::new(expanded_condition),
                        increment: Box::new(expanded_increment),
                        body: Box::new(expanded_body),
                    },
                    node.line,
                    node.column,
                ))
            },
            // For other node types, just clone them
            _ => Ok(node.clone()),
        }
    }
    
    /// Process an AST, registering macro definitions and expanding macro invocations
    pub fn process(&mut self, nodes: &[ASTNode]) -> Result<Vec<ASTNode>, LangError> {
        let mut result = Vec::new();
        
        for node in nodes {
            match &node.node_type {
                NodeType::MacroDefinition { name, pattern, template, is_procedural } => {
                    // Register the macro definition
                    let pattern_obj = super::MacroPattern::from_ast(pattern)?;
                    let definition = if *is_procedural {
                        MacroDefinition::new_procedural(name.clone(), pattern_obj, (**template).clone())
                    } else {
                        MacroDefinition::new_declarative(name.clone(), pattern_obj, (**template).clone())
                    };
                    
                    self.register_macro(definition);
                    
                    // Don't include the definition in the result
                    continue;
                },
                _ => {
                    // Expand macros in the node
                    let expanded = self.expand_all(node)?;
                    result.push(expanded);
                }
            }
        }
        
        Ok(result)
    }
}
