// Semantic analyzer for the minimal LLM-friendly language

use std::collections::HashMap;
use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Collection(Box<Type>),
    Boolean,
    Void,
    Unknown,
    Connection,
    Function(Vec<Type>, Box<Type>),
    Channel(Box<Type>),
    SharedState(Box<Type>),
    Any,
}

pub struct FunctionType {
    parameters: Vec<Type>,
    return_type: Type,
}

pub struct SemanticAnalyzer {
    symbols: HashMap<String, String>, // Variable name -> Type
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbols: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, ast: &[ASTNode]) -> Result<(), LangError> {
        for node in ast {
            self.analyze_node(node)?;
        }
        Ok(())
    }

    fn analyze_node(&mut self, node: &ASTNode) -> Result<(), LangError> {
        match &node.node_type {
            NodeType::Library { name: _, functions } => {
                for func in functions {
                    self.analyze_node(func)?;
                }
            },
            NodeType::FunctionDeclaration { name: _, parameters: _, body } => {
                self.analyze_node(body)?;
            },
            NodeType::Block(statements) => {
                for stmt in statements {
                    self.analyze_node(stmt)?;
                }
            },
            NodeType::Assignment { name, value } => {
                self.analyze_node(value)?;
                self.symbols.insert(name.clone(), "dynamic".to_string());
            },
            NodeType::Binary { left, operator: _, right } => {
                self.analyze_node(left)?;
                self.analyze_node(right)?;
            },
            NodeType::FunctionCall { callee, arguments } => {
                self.analyze_node(callee)?;
                for arg in arguments {
                    self.analyze_node(arg)?;
                }
            },
            NodeType::MethodCall { object, method: _, arguments } => {
                self.analyze_node(object)?;
                for arg in arguments {
                    self.analyze_node(arg)?;
                }
            },
            _ => (), // Other node types don't require semantic analysis for now
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;

    #[test]
    fn test_basic_types() {
        let mut analyzer = SemanticAnalyzer::new();
        let nodes = vec![
            ASTNode {
                node_type: NodeType::Number(42),
                line: 1,
                column: 1,
            },
            ASTNode {
                node_type: NodeType::String("test".to_string()),
                line: 1,
                column: 1,
            },
            ASTNode {
                node_type: NodeType::Boolean(true),
                line: 1,
                column: 1,
            },
        ];
        assert!(analyzer.analyze(&nodes).is_ok());
    }

    #[test]
    fn test_binary_operations() {
        let mut analyzer = SemanticAnalyzer::new();
        let nodes = vec![
            ASTNode {
                node_type: NodeType::Binary {
                    left: Box::new(ASTNode {
                        node_type: NodeType::Number(1),
                        line: 1,
                        column: 1,
                    }),
                    operator: Token::SymbolicOperator('+'),
                    right: Box::new(ASTNode {
                        node_type: NodeType::Number(2),
                        line: 1,
                        column: 3,
                    }),
                },
                line: 1,
                column: 2,
            },
        ];
        assert!(analyzer.analyze(&nodes).is_ok());
    }

    #[test]
    fn test_variable_assignment() {
        let mut analyzer = SemanticAnalyzer::new();
        let nodes = vec![
            ASTNode {
                node_type: NodeType::Assignment {
                    name: "x".to_string(),
                    value: Box::new(ASTNode {
                        node_type: NodeType::Number(42),
                        line: 1,
                        column: 5,
                    }),
                },
                line: 1,
                column: 1,
            },
            ASTNode {
                node_type: NodeType::Variable("x".to_string()),
                line: 1,
                column: 10,
            },
        ];
        assert!(analyzer.analyze(&nodes).is_ok());
    }

    #[test]
    fn test_function_declaration() {
        let mut analyzer = SemanticAnalyzer::new();
        let nodes = vec![
            ASTNode {
                node_type: NodeType::FunctionDeclaration {
                    name: "test".to_string(),
                    parameters: vec!["x".to_string()],
                    body: Box::new(ASTNode {
                        node_type: NodeType::Return(Some(Box::new(ASTNode {
                            node_type: NodeType::Variable("x".to_string()),
                            line: 2,
                            column: 5,
                        }))),
                        line: 2,
                        column: 1,
                    }),
                },
                line: 1,
                column: 1,
            },
        ];
        assert!(analyzer.analyze(&nodes).is_ok());
    }
}
