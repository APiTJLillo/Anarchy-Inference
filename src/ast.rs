// src/ast.rs - Modified to add Null variant
use crate::error::SourceLocation;
use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub node_type: NodeType,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Null,  // Added Null variant for empty/null values
    Number(i64),
    String(String),
    Boolean(bool),
    Variable(String),
    Binary {
        left: Box<ASTNode>,
        operator: Token,
        right: Box<ASTNode>,
    },
    Unary {
        operator: Token,
        operand: Box<ASTNode>,
    },
    Assignment {
        name: String,
        value: Box<ASTNode>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Box<ASTNode>,
    },
    FunctionCall {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    PropertyAccess {
        object: Box<ASTNode>,
        property: String,
    },
    MethodCall {
        object: Box<ASTNode>,
        method: String,
        arguments: Vec<ASTNode>,
    },
    Block(Vec<ASTNode>),
    Library {
        name: String,
        functions: Vec<ASTNode>,
    },
    Return(Option<Box<ASTNode>>),
    If {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    For {
        initializer: Box<ASTNode>,
        condition: Box<ASTNode>,
        increment: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    Break,
    Continue,
    Channel(Box<ASTNode>),
    Send {
        channel: Box<ASTNode>,
        value: Box<ASTNode>,
    },
    Receive(Box<ASTNode>),
    SharedState {
        name: String,
        value: Box<ASTNode>,
    },
    SetSharedState {
        name: String,
        value: Box<ASTNode>,
    },
    GetSharedState {
        name: String,
    },
    Identifier(String),
    SymbolicKeyword(String),
    Lambda {
        params: Vec<String>,
        body: Box<ASTNode>,
    },
    Print(Box<ASTNode>),
}

impl ASTNode {
    pub fn get_location(&self) -> (usize, usize) {
        (self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let node = ASTNode {
            node_type: NodeType::Number(42),
            line: 1,
            column: 1,
        };
        assert!(matches!(node.node_type, NodeType::Number(42)));
        assert_eq!(node.get_location(), (1, 1));
    }

    #[test]
    fn test_binary_node_creation() {
        let left = Box::new(ASTNode {
            node_type: NodeType::Number(1),
            line: 1,
            column: 1,
        });
        let right = Box::new(ASTNode {
            node_type: NodeType::Number(2),
            line: 1,
            column: 3,
        });
        let node = ASTNode {
            node_type: NodeType::Binary {
                left,
                operator: Token::SymbolicOperator('+'),
                right,
            },
            line: 1,
            column: 2,
        };
        assert!(matches!(node.node_type, NodeType::Binary { .. }));
    }

    #[test]
    fn test_function_declaration_node() {
        let body = Box::new(ASTNode {
            node_type: NodeType::Block(vec![]),
            line: 1,
            column: 1,
        });
        let node = ASTNode {
            node_type: NodeType::FunctionDeclaration {
                name: "test".to_string(),
                parameters: vec!["x".to_string(), "y".to_string()],
                body,
            },
            line: 1,
            column: 1,
        };
        if let NodeType::FunctionDeclaration { name, parameters, .. } = node.node_type {
            assert_eq!(name, "test");
            assert_eq!(parameters, vec!["x", "y"]);
        } else {
            panic!("Expected function declaration node");
        }
    }

    #[test]
    fn test_concurrency_nodes() {
        let channel = Box::new(ASTNode {
            node_type: NodeType::Number(5),
            line: 1,
            column: 1,
        });
        let node = ASTNode {
            node_type: NodeType::Channel(channel),
            line: 1,
            column: 1,
        };
        assert!(matches!(node.node_type, NodeType::Channel(_)));

        let _state = Box::new(ASTNode {
            node_type: NodeType::Variable("state".to_string()),
            line: 1,
            column: 1,
        });
        let value = Box::new(ASTNode {
            node_type: NodeType::Number(42),
            line: 1,
            column: 1,
        });
        let node = ASTNode {
            node_type: NodeType::SetSharedState {
                name: "counter".to_string(),
                value,
            },
            line: 1,
            column: 1,
        };
        assert!(matches!(node.node_type, NodeType::SetSharedState { .. }));
    }
}
