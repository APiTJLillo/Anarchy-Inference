// src/ast.rs - Modified to add macro system support
use crate::error::SourceLocation;
use crate::lexer::Token;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub node_type: NodeType,
    pub line: usize,
    pub column: usize,
    pub documentation: Option<String>, // Added for module documentation
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
    // Enhanced module system nodes
    ModuleDeclaration {
        name: String,
        is_public: bool,
        items: Vec<ASTNode>,
        version: Option<String>,           // Added for module versioning
        features: Option<Vec<String>>,     // Added for conditional compilation
        attributes: Option<HashMap<String, String>>, // Added for module metadata
    },
    ModuleImport {
        name: String,
        version_constraint: Option<String>, // Added for module versioning
        features: Option<Vec<String>>,      // Added for conditional compilation
    },
    ImportDeclaration {
        module_path: Vec<String>,
        items: Vec<String>,
        import_all: bool,
        alias: Option<String>,             // Added for module aliases
        re_export: bool,                   // Added for partial re-exports
        item_aliases: Option<HashMap<String, String>>, // Added for item renaming
    },
    ModulePath {
        path: Vec<String>,
        item: Box<ASTNode>,
    },
    // New node for conditional compilation
    ConditionalBlock {
        condition: String,
        items: Vec<ASTNode>,
    },
    // New node for re-exports
    ReExport {
        module_path: Vec<String>,
        items: Vec<String>,
        item_aliases: Option<HashMap<String, String>>,
    },
    // New nodes for macro system
    MacroDefinition {
        name: String,
        pattern: Box<ASTNode>,
        template: Box<ASTNode>,
        is_procedural: bool,
    },
    MacroInvocation {
        name: String,
        arguments: Vec<ASTNode>,
    },
    MacroExpansion {
        original: Box<ASTNode>,
        expanded: Box<ASTNode>,
    },
    MacroPattern {
        variables: Vec<String>,
        pattern: Box<ASTNode>,
    },
    MacroVariable(String),
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
    pub fn new(node_type: NodeType, line: usize, column: usize) -> Self {
        Self {
            node_type,
            line,
            column,
            documentation: None,
        }
    }

    pub fn with_documentation(node_type: NodeType, line: usize, column: usize, documentation: String) -> Self {
        Self {
            node_type,
            line,
            column,
            documentation: Some(documentation),
        }
    }

    pub fn get_location(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    pub fn get_documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }

    pub fn set_documentation(&mut self, documentation: String) {
        self.documentation = Some(documentation);
    }
}

// Version constraint parsing and checking
#[derive(Debug, Clone, PartialEq)]
pub enum VersionConstraint {
    Exact(String),
    GreaterThan(String),
    GreaterThanOrEqual(String),
    LessThan(String),
    LessThanOrEqual(String),
    Caret(String),    // ^1.2.3 -> >=1.2.3 <2.0.0
    Tilde(String),    // ~1.2.3 -> >=1.2.3 <1.3.0
    Range(String, String), // Between two versions
}

impl VersionConstraint {
    pub fn parse(constraint: &str) -> Result<Self, String> {
        if constraint.starts_with("^") {
            let version = constraint[1..].to_string();
            Ok(VersionConstraint::Caret(version))
        } else if constraint.starts_with("~") {
            let version = constraint[1..].to_string();
            Ok(VersionConstraint::Tilde(version))
        } else if constraint.starts_with(">=") {
            let version = constraint[2..].to_string();
            Ok(VersionConstraint::GreaterThanOrEqual(version))
        } else if constraint.starts_with(">") {
            let version = constraint[1..].to_string();
            Ok(VersionConstraint::GreaterThan(version))
        } else if constraint.starts_with("<=") {
            let version = constraint[2..].to_string();
            Ok(VersionConstraint::LessThanOrEqual(version))
        } else if constraint.starts_with("<") {
            let version = constraint[1..].to_string();
            Ok(VersionConstraint::LessThan(version))
        } else if constraint.contains(",") {
            let parts: Vec<&str> = constraint.split(",").collect();
            if parts.len() != 2 {
                return Err(format!("Invalid range constraint: {}", constraint));
            }
            Ok(VersionConstraint::Range(parts[0].to_string(), parts[1].to_string()))
        } else {
            Ok(VersionConstraint::Exact(constraint.to_string()))
        }
    }

    pub fn satisfies(&self, version: &str) -> bool {
        match self {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::GreaterThan(v) => version_compare(version, v) > 0,
            VersionConstraint::GreaterThanOrEqual(v) => version_compare(version, v) >= 0,
            VersionConstraint::LessThan(v) => version_compare(version, v) < 0,
            VersionConstraint::LessThanOrEqual(v) => version_compare(version, v) <= 0,
            VersionConstraint::Caret(v) => {
                let parts: Vec<&str> = v.split(".").collect();
                if parts.len() < 1 {
                    return false;
                }
                let major = parts[0].parse::<i32>().unwrap_or(0);
                let next_major = format!("{}.0.0", major + 1);
                version_compare(version, v) >= 0 && version_compare(version, &next_major) < 0
            },
            VersionConstraint::Tilde(v) => {
                let parts: Vec<&str> = v.split(".").collect();
                if parts.len() < 2 {
                    return false;
                }
                let major = parts[0].parse::<i32>().unwrap_or(0);
                let minor = parts[1].parse::<i32>().unwrap_or(0);
                let next_minor = format!("{}.{}.0", major, minor + 1);
                version_compare(version, v) >= 0 && version_compare(version, &next_minor) < 0
            },
            VersionConstraint::Range(v1, v2) => {
                version_compare(version, v1) >= 0 && version_compare(version, v2) < 0
            },
        }
    }
}

// Simple semantic version comparison
fn version_compare(a: &str, b: &str) -> i32 {
    let a_parts: Vec<i32> = a.split(".")
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect();
    let b_parts: Vec<i32> = b.split(".")
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect();
    
    let max_len = std::cmp::max(a_parts.len(), b_parts.len());
    
    for i in 0..max_len {
        let a_val = if i < a_parts.len() { a_parts[i] } else { 0 };
        let b_val = if i < b_parts.len() { b_parts[i] } else { 0 };
        
        if a_val > b_val {
            return 1;
        } else if a_val < b_val {
            return -1;
        }
    }
    
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let node = ASTNode::new(NodeType::Number(42), 1, 1);
        assert!(matches!(node.node_type, NodeType::Number(42)));
        assert_eq!(node.get_location(), (1, 1));
    }

    #[test]
    fn test_ast_node_with_documentation() {
        let node = ASTNode::with_documentation(
            NodeType::Number(42), 
            1, 
            1, 
            "This is a number".to_string()
        );
        assert!(matches!(node.node_type, NodeType::Number(42)));
        assert_eq!(node.get_documentation(), Some(&"This is a number".to_string()));
    }

    #[test]
    fn test_binary_node_creation() {
        let left = Box::new(ASTNode::new(NodeType::Number(1), 1, 1));
        let right = Box::new(ASTNode::new(NodeType::Number(2), 1, 3));
        let node = ASTNode::new(
            NodeType::Binary {
                left,
                operator: Token::SymbolicOperator('+'),
                right,
            },
            1, 
            2
        );
        assert!(matches!(node.node_type, NodeType::Binary { .. }));
    }

    #[test]
    fn test_function_declaration_node() {
        let body = Box::new(ASTNode::new(NodeType::Block(vec![]), 1, 1));
        let node = ASTNode::new(
            NodeType::FunctionDeclaration {
                name: "test".to_string(),
                parameters: vec!["x".to_string(), "y".to_string()],
                body,
            },
            1,
            1
        );
        if let NodeType::FunctionDeclaration { name, parameters, .. } = node.node_type {
            assert_eq!(name, "test");
            assert_eq!(parameters, vec!["x", "y"]);
        } else {
            panic!("Expected function declaration node");
        }
    }

    #[test]
    fn test_module_declaration_with_version() {
        let node = ASTNode::new(
            NodeType::ModuleDeclaration {
                name: "test_module".to_string(),
                is_public: true,
                items: vec![],
                version: Some("1.0.0".to_string()),
                features: None,
                attributes: None,
            },
            1,
            1
        );
        if let NodeType::ModuleDeclaration { name, is_public, version, .. } = &node.node_type {
            assert_eq!(name, "test_module");
            assert_eq!(*is_public, true);
            assert_eq!(version, &Some("1.0.0".to_string()));
        } else {
            panic!("Expected ModuleDeclaration node");
        }
    }

    #[test]
    fn test_import_declaration_with_alias() {
        let node = ASTNode::new(
            NodeType::ImportDeclaration {
                module_path: vec!["math".to_string()],
                items: vec!["add".to_string(), "subtract".to_string()],
                import_all: false,
                alias: Some("m".to_string()),
                re_export: false,
                item_aliases: None,
            },
            1,
            1
        );
        if let NodeType::ImportDeclaration { module_path, items, alias, .. } = &node.node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &vec!["add".to_string(), "subtract".to_string()]);
            assert_eq!(alias, &Some("m".to_string()));
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }

    #[test]
    fn test_re_export_node() {
        let node = ASTNode::new(
            NodeType::ReExport {
                module_path: vec!["math".to_string()],
                items: vec!["add".to_string(), "subtract".to_string()],
                item_aliases: Some({
                    let mut map = HashMap::new();
                    map.insert("add".to_string(), "plus".to_string());
                    map
                }),
            },
            1,
            1
        );
        if let NodeType::ReExport { module_path, items, item_aliases } = &node.node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &vec!["add".to_string(), "subtract".to_string()]);
            assert_eq!(
                item_aliases.as_ref().unwrap().get("add"), 
                Some(&"plus".to_string())
            );
        } else {
            panic!("Expected ReExport node");
        }
    }

    #[test]
    fn test_conditional_block() {
        let node = ASTNode::new(
            NodeType::ConditionalBlock {
                condition: "feature=\"web\"".to_string(),
                items: vec![],
            },
            1,
            1
        );
        if let NodeType::ConditionalBlock { condition, .. } = &node.node_type {
            assert_eq!(condition, "feature=\"web\"");
        } else {
            panic!("Expected ConditionalBlock node");
        }
    }

    // Tests for macro system nodes
    #[test]
    fn test_macro_definition() {
        let pattern = Box::new(ASTNode::new(
            NodeType::MacroPattern {
                variables: vec!["condition".to_string(), "body".to_string()],
                pattern: Box::new(ASTNode::new(NodeType::Block(vec![]), 1, 1)),
            },
            1, 
            1
        ));
        
        let template = Box::new(ASTNode::new(NodeType::Block(vec![]), 1, 1));
        
        let node = ASTNode::new(
            NodeType::MacroDefinition {
                name: "unless".to_string(),
                pattern,
                template,
                is_procedural: false,
            },
            1,
            1
        );
        
        if let NodeType::MacroDefinition { name, is_procedural, .. } = &node.node_type {
            assert_eq!(name, "unless");
            assert_eq!(*is_procedural, false);
        } else {
            panic!("Expected MacroDefinition node");
        }
    }
    
    #[test]
    fn test_macro_invocation() {
        let node = ASTNode::new(
            NodeType::MacroInvocation {
                name: "unless".to_string(),
                arguments: vec![
                    ASTNode::new(NodeType::Boolean(false), 1, 10),
                    ASTNode::new(NodeType::Block(vec![]), 1, 15),
                ],
            },
            1,
            1
        );
        
        if let NodeType::MacroInvocation { name, arguments } = &node.node_type {
            assert_eq!(name, "unless");
            assert_eq!(arguments.len(), 2);
        } else {
            panic!("Expected MacroInvocation node");
        }
    }
    
    #[test]
    fn test_macro_expansion() {
        let original = Box::new(ASTNode::new(
            NodeType::MacroInvocation {
                name: "unless".to_string(),
                arguments: vec![
                    ASTNode::new(NodeType::Boolean(false), 1, 10),
                    ASTNode::new(NodeType::Block(vec![]), 1, 15),
                ],
            },
            1,
            1
        ));
        
        let expanded = Box::new(ASTNode::new(
            NodeType::If {
                condition: Box::new(ASTNode::new(
                    NodeType::Unary {
                        operator: Token::SymbolicOperator('!'),
                        operand: Box::new(ASTNode::new(NodeType::Boolean(false), 1, 10)),
                    },
                    1,
                    9
                )),
                then_branch: Box::new(ASTNode::new(NodeType::Block(vec![]), 1, 15)),
                else_branch: None,
            },
            1,
            1
        ));
        
        let node = ASTNode::new(
            NodeType::MacroExpansion {
                original,
                expanded,
            },
            1,
            1
        );
        
        assert!(matches!(node.node_type, NodeType::MacroExpansion { .. }));
    }
}
