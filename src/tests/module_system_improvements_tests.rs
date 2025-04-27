# Module System Improvements Tests

This file contains comprehensive tests for the improved module system in Anarchy Inference.

```rust
#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ast::{ASTNode, NodeType, VersionConstraint};
    use std::collections::HashMap;

    #[test]
    fn test_module_versioning() {
        // Test module declaration with version
        let input = "λ math v\"1.0.0\" { }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleDeclaration { name, version, .. } = &ast[0].node_type {
            assert_eq!(name, "math");
            assert_eq!(version, &Some("1.0.0".to_string()));
        } else {
            panic!("Expected ModuleDeclaration node");
        }
        
        // Test module import with version constraint
        let input = "λ⟨ math v\"^1.0.0\" ⟩";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleImport { name, version_constraint, .. } = &ast[0].node_type {
            assert_eq!(name, "math");
            assert_eq!(version_constraint, &Some("^1.0.0".to_string()));
        } else {
            panic!("Expected ModuleImport node");
        }
        
        // Test import declaration with version constraint
        let input = "⟑ math v\">=1.0.0,<2.0.0\"::{add, subtract}";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ImportDeclaration { module_path, items, .. } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &vec!["add".to_string(), "subtract".to_string()]);
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }
    
    #[test]
    fn test_version_constraint_parsing() {
        // Test exact version
        let constraint = VersionConstraint::parse("1.0.0").unwrap();
        assert_eq!(constraint, VersionConstraint::Exact("1.0.0".to_string()));
        assert!(constraint.satisfies("1.0.0"));
        assert!(!constraint.satisfies("1.0.1"));
        
        // Test caret version
        let constraint = VersionConstraint::parse("^1.0.0").unwrap();
        assert_eq!(constraint, VersionConstraint::Caret("1.0.0".to_string()));
        assert!(constraint.satisfies("1.0.0"));
        assert!(constraint.satisfies("1.9.9"));
        assert!(!constraint.satisfies("2.0.0"));
        
        // Test tilde version
        let constraint = VersionConstraint::parse("~1.2.0").unwrap();
        assert_eq!(constraint, VersionConstraint::Tilde("1.2.0".to_string()));
        assert!(constraint.satisfies("1.2.0"));
        assert!(constraint.satisfies("1.2.9"));
        assert!(!constraint.satisfies("1.3.0"));
        
        // Test greater than or equal
        let constraint = VersionConstraint::parse(">=1.0.0").unwrap();
        assert_eq!(constraint, VersionConstraint::GreaterThanOrEqual("1.0.0".to_string()));
        assert!(constraint.satisfies("1.0.0"));
        assert!(constraint.satisfies("2.0.0"));
        assert!(!constraint.satisfies("0.9.9"));
        
        // Test less than
        let constraint = VersionConstraint::parse("<2.0.0").unwrap();
        assert_eq!(constraint, VersionConstraint::LessThan("2.0.0".to_string()));
        assert!(constraint.satisfies("1.9.9"));
        assert!(!constraint.satisfies("2.0.0"));
        
        // Test range
        let constraint = VersionConstraint::parse(">=1.0.0,<2.0.0").unwrap();
        assert_eq!(constraint, VersionConstraint::Range(">=1.0.0".to_string(), "<2.0.0".to_string()));
        assert!(constraint.satisfies("1.0.0"));
        assert!(constraint.satisfies("1.9.9"));
        assert!(!constraint.satisfies("0.9.9"));
        assert!(!constraint.satisfies("2.0.0"));
    }
    
    #[test]
    fn test_module_aliases() {
        // Test import with alias
        let input = "⟑ long_module_name as short::{item1, item2}";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ImportDeclaration { module_path, alias, items, .. } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["long_module_name".to_string()]);
            assert_eq!(alias, &Some("short".to_string()));
            assert_eq!(items, &vec!["item1".to_string(), "item2".to_string()]);
        } else {
            panic!("Expected ImportDeclaration node");
        }
        
        // Test complex import with alias and version
        let input = "⟑ module_name v\"^1.2.3\" as m::{item1, item2}";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ImportDeclaration { module_path, alias, .. } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["module_name".to_string()]);
            assert_eq!(alias, &Some("m".to_string()));
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }
    
    #[test]
    fn test_partial_re_exports() {
        // Test re-export
        let input = "⊢ ⟑ math::{add, subtract}";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ReExport { module_path, items, .. } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &vec!["add".to_string(), "subtract".to_string()]);
        } else {
            panic!("Expected ReExport node");
        }
        
        // Test re-export with item aliases (not fully implemented in parser yet)
        let mut item_aliases = HashMap::new();
        item_aliases.insert("add".to_string(), "plus".to_string());
        
        let node = ASTNode::new(
            NodeType::ReExport {
                module_path: vec!["math".to_string()],
                items: vec!["add".to_string(), "subtract".to_string()],
                item_aliases: Some(item_aliases),
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
    fn test_conditional_compilation() {
        // Test conditional block
        let input = "#[if(feature=\"web\")] { }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ConditionalBlock { condition, items } = &ast[0].node_type {
            assert_eq!(condition, "feature=\"web\"");
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected ConditionalBlock node");
        }
        
        // Test module with feature
        let input = "λ math #[feature=\"web\"] { }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.set_enabled_features(vec!["web".to_string()]);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleDeclaration { name, features, .. } = &ast[0].node_type {
            assert_eq!(name, "math");
            assert!(features.is_some());
            assert_eq!(features.as_ref().unwrap(), &vec!["web".to_string()]);
        } else {
            panic!("Expected ModuleDeclaration node");
        }
        
        // Test feature evaluation
        let mut parser = Parser::new(vec![]);
        parser.set_enabled_features(vec!["web".to_string()]);
        assert!(parser.is_feature_enabled("web"));
        assert!(!parser.is_feature_enabled("native"));
        assert!(parser.evaluate_condition("feature=\"web\""));
        assert!(!parser.evaluate_condition("feature=\"native\""));
        assert!(!parser.evaluate_condition("!feature=\"web\""));
        assert!(parser.evaluate_condition("!feature=\"native\""));
    }
    
    #[test]
    fn test_module_documentation() {
        // Test module with documentation
        let input = "/// This is a math module\nλ math { }";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        assert_eq!(ast[0].documentation, Some("This is a math module".to_string()));
        
        // Test function with documentation
        let node = ASTNode::with_documentation(
            NodeType::FunctionDeclaration {
                name: "add".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: Box::new(ASTNode::new(NodeType::Null, 1, 1)),
            },
            1,
            1,
            "Adds two numbers".to_string()
        );
        
        assert_eq!(node.documentation, Some("Adds two numbers".to_string()));
    }
    
    #[test]
    fn test_circular_dependency_resolution() {
        // This test would require integration with the module loader
        // For now, we'll just test the AST structure
        
        // Module A imports from Module B
        let input_a = "λ module_a { ⟑ module_b::item_b }";
        let mut lexer = Lexer::new(input_a.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast_a = parser.parse().unwrap();
        
        // Module B imports from Module A
        let input_b = "λ module_b { ⟑ module_a::item_a }";
        let mut lexer = Lexer::new(input_b.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast_b = parser.parse().unwrap();
        
        // Verify the structure
        assert_eq!(ast_a.len(), 1);
        assert_eq!(ast_b.len(), 1);
        
        if let NodeType::ModuleDeclaration { name: name_a, .. } = &ast_a[0].node_type {
            assert_eq!(name_a, "module_a");
        } else {
            panic!("Expected ModuleDeclaration node");
        }
        
        if let NodeType::ModuleDeclaration { name: name_b, .. } = &ast_b[0].node_type {
            assert_eq!(name_b, "module_b");
        } else {
            panic!("Expected ModuleDeclaration node");
        }
    }
    
    #[test]
    fn test_complex_module_system() {
        let code = r#"
        /// Math module with versioning
        λ math v"1.0.0" #[feature="math"] {
            /// Add two numbers
            ⊢ ƒ add(a, b) {
                ⟼ a + b
            }
            
            /// Subtract two numbers
            ⊢ ƒ subtract(a, b) {
                ⟼ a - b
            }
            
            #[if(feature="advanced")]
            λ advanced {
                ⊢ ƒ multiply(a, b) {
                    ⟼ a * b
                }
            }
        }
        
        λ⟨ geometry v"^2.0.0" ⟩
        
        ⟑ math as m::{add, subtract}
        ⟑ geometry::shapes::*
        
        ⊢ ⟑ math::add as plus
        
        λ app {
            ⊢ λ utils {
                ⊢ ƒ helper() {
                    ⟼ m::add(5, 3)
                }
            }
            
            #[if(feature="web")]
            λ web {
                ⊢ ƒ render() {
                    // Web-specific implementation
                }
            }
            
            #[if(feature="native")]
            λ native {
                ⊢ ƒ render() {
                    // Native-specific implementation
                }
            }
        }
        "#;
        
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.set_enabled_features(vec!["math".to_string(), "advanced".to_string(), "web".to_string()]);
        let ast = parser.parse().unwrap();
        
        // Verify the AST structure
        assert!(ast.len() >= 5); // Math module, geometry import, 2 imports, re-export, app module
        
        // Check for math module with version
        let mut found_math_module = false;
        let mut found_geometry_import = false;
        let mut found_math_import = false;
        let mut found_geometry_shapes_import = false;
        let mut found_re_export = false;
        let mut found_app_module = false;
        
        for node in &ast {
            match &node.node_type {
                NodeType::ModuleDeclaration { name, version, features, .. } if name == "math" => {
                    found_math_module = true;
                    assert_eq!(version, &Some("1.0.0".to_string()));
                    assert!(features.is_some());
                    assert_eq!(features.as_ref().unwrap(), &vec!["math".to_string()]);
                    assert_eq!(node.documentation, Some("Math module with versioning".to_string()));
                },
                NodeType::ModuleImport { name, version_constraint, .. } if name == "geometry" => {
                    found_geometry_import = true;
                    assert_eq!(version_constraint, &Some("^2.0.0".to_string()));
                },
                NodeType::ImportDeclaration { module_path, alias, .. } if module_path == &vec!["math".to_string()] => {
                    found_math_import = true;
                    assert_eq!(alias, &Some("m".to_string()));
                },
                NodeType::ImportDeclaration { module_path, import_all, .. } 
                    if module_path == &vec!["geometry".to_string(), "shapes".to_string()] => {
                    found_geometry_shapes_import = true;
                    assert!(*import_all);
                },
                NodeType::ReExport { module_path, items, .. } 
                    if module_path == &vec!["math".to_string()] && items == &vec!["add".to_string()] => {
                    found_re_export = true;
                },
                NodeType::ModuleDeclaration { name, .. } if name == "app" => {
                    found_app_module = true;
                },
                _ => {}
            }
        }
        
        assert!(found_math_module, "Missing math module");
        assert!(found_geometry_import, "Missing geometry import");
        assert!(found_math_import, "Missing math import");
        assert!(found_geometry_shapes_import, "Missing geometry shapes import");
        assert!(found_re_export, "Missing re-export");
        assert!(found_app_module, "Missing app module");
    }
}
```
