// src/ast.rs - Modified to add module system support
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
    StringDictRef(String), // New variant for string dictionary references
    UserInput, // New variant for user input emoji (🎤)
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
    // New module system nodes
    ModuleDeclaration {
        name: String,
        is_public: bool,
        items: Vec<ASTNode>,
    },
    ModuleImport {
        name: String,
    },
    ImportDeclaration {
        module_path: Vec<String>,
        items: Vec<String>,
        import_all: bool,
    },
    ModulePath {
        path: Vec<String>,
        item: Box<ASTNode>,
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
    
    #[test]
    fn test_string_dict_ref_node() {
        let node = ASTNode {
            node_type: NodeType::StringDictRef("hello".to_string()),
            line: 1,
            column: 1,
        };
        if let NodeType::StringDictRef(key) = &node.node_type {
            assert_eq!(key, "hello");
        } else {
            panic!("Expected StringDictRef node");
        }
    }
    
    #[test]
    fn test_user_input_node() {
        let node = ASTNode {
            node_type: NodeType::UserInput,
            line: 1,
            column: 1,
        };
        assert!(matches!(node.node_type, NodeType::UserInput));
    }
    
    #[test]
    fn test_module_declaration_node() {
        let node = ASTNode {
            node_type: NodeType::ModuleDeclaration {
                name: "test_module".to_string(),
                is_public: true,
                items: vec![],
            },
            line: 1,
            column: 1,
        };
        if let NodeType::ModuleDeclaration { name, is_public, items } = &node.node_type {
            assert_eq!(name, "test_module");
            assert_eq!(*is_public, true);
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected ModuleDeclaration node");
        }
    }
    
    #[test]
    fn test_module_import_node() {
        let node = ASTNode {
            node_type: NodeType::ModuleImport {
                name: "math".to_string(),
            },
            line: 1,
            column: 1,
        };
        if let NodeType::ModuleImport { name } = &node.node_type {
            assert_eq!(name, "math");
        } else {
            panic!("Expected ModuleImport node");
        }
    }
    
    #[test]
    fn test_import_declaration_node() {
        let node = ASTNode {
            node_type: NodeType::ImportDeclaration {
                module_path: vec!["math".to_string()],
                items: vec!["add".to_string(), "subtract".to_string()],
                import_all: false,
            },
            line: 1,
            column: 1,
        };
        if let NodeType::ImportDeclaration { module_path, items, import_all } = &node.node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &vec!["add".to_string(), "subtract".to_string()]);
            assert_eq!(*import_all, false);
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }
}
