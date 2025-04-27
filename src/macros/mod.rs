// src/macros/mod.rs - Macro system implementation for Anarchy Inference

mod expander;
mod hygiene;
mod pattern;

pub use expander::MacroExpander;
pub use pattern::MacroPattern;

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use std::collections::HashMap;

/// Represents a macro definition in the language
#[derive(Debug, Clone)]
pub struct MacroDefinition {
    /// Name of the macro
    pub name: String,
    /// Pattern to match against
    pub pattern: MacroPattern,
    /// Template to expand to
    pub template: ASTNode,
    /// Whether this is a procedural macro
    pub is_procedural: bool,
    /// Environment for procedural macros
    pub environment: Option<HashMap<String, ASTNode>>,
}

impl MacroDefinition {
    /// Create a new declarative macro
    pub fn new_declarative(name: String, pattern: MacroPattern, template: ASTNode) -> Self {
        Self {
            name,
            pattern,
            template,
            is_procedural: false,
            environment: None,
        }
    }
    
    /// Create a new procedural macro
    pub fn new_procedural(name: String, pattern: MacroPattern, template: ASTNode) -> Self {
        Self {
            name,
            pattern,
            template,
            is_procedural: true,
            environment: Some(HashMap::new()),
        }
    }
    
    /// Expand a macro invocation
    pub fn expand(&self, arguments: &[ASTNode], expander: &MacroExpander) -> Result<ASTNode, LangError> {
        if self.is_procedural {
            self.expand_procedural(arguments, expander)
        } else {
            self.expand_declarative(arguments, expander)
        }
    }
    
    /// Expand a declarative macro
    fn expand_declarative(&self, arguments: &[ASTNode], expander: &MacroExpander) -> Result<ASTNode, LangError> {
        // Match the pattern against the arguments
        let bindings = self.pattern.match_arguments(arguments)?;
        
        // Apply the bindings to the template
        let expanded = self.apply_bindings(&self.template, &bindings, expander)?;
        
        Ok(expanded)
    }
    
    /// Expand a procedural macro
    fn expand_procedural(&self, arguments: &[ASTNode], expander: &MacroExpander) -> Result<ASTNode, LangError> {
        // Match the pattern against the arguments
        let bindings = self.pattern.match_arguments(arguments)?;
        
        // Create an environment for the procedural macro
        let mut env = self.environment.clone().unwrap_or_default();
        
        // Add the bindings to the environment
        for (name, node) in bindings {
            env.insert(name, node);
        }
        
        // Execute the procedural macro
        // This would normally involve interpreting the macro body,
        // but for now we'll just return the template as a placeholder
        let expanded = self.template.clone();
        
        Ok(expanded)
    }
    
    /// Apply bindings to a template
    fn apply_bindings(&self, template: &ASTNode, bindings: &HashMap<String, ASTNode>, expander: &MacroExpander) -> Result<ASTNode, LangError> {
        match &template.node_type {
            NodeType::MacroVariable(name) => {
                // Replace macro variables with their bindings
                if let Some(binding) = bindings.get(name) {
                    Ok(binding.clone())
                } else {
                    Err(LangError::runtime_error(&format!("Macro variable '{}' not found", name)))
                }
            },
            NodeType::Block(nodes) => {
                // Apply bindings to each node in the block
                let mut expanded_nodes = Vec::new();
                for node in nodes {
                    let expanded = self.apply_bindings(node, bindings, expander)?;
                    expanded_nodes.push(expanded);
                }
                
                Ok(ASTNode::new(
                    NodeType::Block(expanded_nodes),
                    template.line,
                    template.column,
                ))
            },
            NodeType::MacroInvocation { name, arguments } => {
                // Expand nested macro invocations
                let mut expanded_args = Vec::new();
                for arg in arguments {
                    let expanded = self.apply_bindings(arg, bindings, expander)?;
                    expanded_args.push(expanded);
                }
                
                expander.expand_macro(name, &expanded_args)
            },
            // Handle other node types recursively
            NodeType::Binary { left, operator, right } => {
                let expanded_left = self.apply_bindings(left, bindings, expander)?;
                let expanded_right = self.apply_bindings(right, bindings, expander)?;
                
                Ok(ASTNode::new(
                    NodeType::Binary {
                        left: Box::new(expanded_left),
                        operator: operator.clone(),
                        right: Box::new(expanded_right),
                    },
                    template.line,
                    template.column,
                ))
            },
            NodeType::Unary { operator, operand } => {
                let expanded_operand = self.apply_bindings(operand, bindings, expander)?;
                
                Ok(ASTNode::new(
                    NodeType::Unary {
                        operator: operator.clone(),
                        operand: Box::new(expanded_operand),
                    },
                    template.line,
                    template.column,
                ))
            },
            NodeType::FunctionCall { callee, arguments } => {
                let expanded_callee = self.apply_bindings(callee, bindings, expander)?;
                
                let mut expanded_args = Vec::new();
                for arg in arguments {
                    let expanded = self.apply_bindings(arg, bindings, expander)?;
                    expanded_args.push(expanded);
                }
                
                Ok(ASTNode::new(
                    NodeType::FunctionCall {
                        callee: Box::new(expanded_callee),
                        arguments: expanded_args,
                    },
                    template.line,
                    template.column,
                ))
            },
            // For other node types, just clone them
            _ => Ok(template.clone()),
        }
    }
}
