# Module System and User Input Emoji Tests

This file contains tests for the newly implemented module system and user input emoji (🎤) support in Anarchy Inference.

```rust
#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Token};
    use crate::parser::Parser;
    use crate::ast::{ASTNode, NodeType};

    #[test]
    fn test_user_input_emoji_lexer() {
        let mut lexer = Lexer::new("🎤".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // UserInput + EOF
        assert!(matches!(tokens[0].token, Token::UserInput));
        assert!(matches!(tokens[1].token, Token::EOF));
    }

    #[test]
    fn test_user_input_emoji_parser() {
        let mut lexer = Lexer::new("🎤".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].node_type, NodeType::UserInput));
    }

    #[test]
    fn test_user_input_emoji_in_expression() {
        let mut lexer = Lexer::new("x = 🎤".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::Assignment { name, value } = &ast[0].node_type {
            assert_eq!(name, "x");
            assert!(matches!(value.node_type, NodeType::UserInput));
        } else {
            panic!("Expected Assignment node");
        }
    }

    #[test]
    fn test_basic_module_declaration() {
        let mut lexer = Lexer::new("λ math { }".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleDeclaration { name, is_public, items } = &ast[0].node_type {
            assert_eq!(name, "math");
            assert_eq!(*is_public, false);
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected ModuleDeclaration node");
        }
    }

    #[test]
    fn test_public_module_declaration() {
        let mut lexer = Lexer::new("⊢ λ math { }".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleDeclaration { name, is_public, items } = &ast[0].node_type {
            assert_eq!(name, "math");
            assert_eq!(*is_public, true);
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected ModuleDeclaration node");
        }
    }

    #[test]
    fn test_nested_module_declaration() {
        let mut lexer = Lexer::new("λ geometry { λ shapes { } }".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleDeclaration { name, items, .. } = &ast[0].node_type {
            assert_eq!(name, "geometry");
            assert_eq!(items.len(), 1);
            if let NodeType::ModuleDeclaration { name, .. } = &items[0].node_type {
                assert_eq!(name, "shapes");
            } else {
                panic!("Expected nested ModuleDeclaration node");
            }
        } else {
            panic!("Expected ModuleDeclaration node");
        }
    }

    #[test]
    fn test_file_based_module_import() {
        let mut lexer = Lexer::new("λ⟨ math ⟩".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ModuleImport { name } = &ast[0].node_type {
            assert_eq!(name, "math");
        } else {
            panic!("Expected ModuleImport node");
        }
    }

    #[test]
    fn test_import_specific_items() {
        let mut lexer = Lexer::new("⟑ math::{add, subtract}".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ImportDeclaration { module_path, items, import_all } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &vec!["add".to_string(), "subtract".to_string()]);
            assert_eq!(*import_all, false);
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }

    #[test]
    fn test_import_all_items() {
        let mut lexer = Lexer::new("⟑ math::*".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ImportDeclaration { module_path, items, import_all } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items.len(), 0);
            assert_eq!(*import_all, true);
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }

    #[test]
    fn test_nested_module_path() {
        let mut lexer = Lexer::new("⟑ geometry::shapes::circle".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.len(), 1);
        if let NodeType::ImportDeclaration { module_path, items, import_all } = &ast[0].node_type {
            assert_eq!(module_path, &vec!["geometry".to_string(), "shapes".to_string()]);
            assert_eq!(items, &vec!["circle".to_string()]);
            assert_eq!(*import_all, false);
        } else {
            panic!("Expected ImportDeclaration node");
        }
    }

    #[test]
    fn test_complex_module_system() {
        let code = r#"
        λ⟨ math ⟩
        λ⟨ geometry ⟩
        
        ⟑ math::{add, subtract}
        ⟑ geometry::shapes::*
        
        λ app {
            ⊢ λ utils {
                ⊢ ƒ helper() {
                    ⟼ math::add(5, 3)
                }
            }
            
            ⊢ ƒ main() {
                ⌽ utils::helper()
                ⌽ 🎤
            }
        }
        "#;
        
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        // Verify the AST structure
        assert!(ast.len() >= 5); // 2 module imports, 2 import declarations, 1 module declaration
        
        // Check for module imports
        let mut found_math_import = false;
        let mut found_geometry_import = false;
        
        // Check for import declarations
        let mut found_math_items_import = false;
        let mut found_geometry_shapes_import = false;
        
        // Check for app module declaration
        let mut found_app_module = false;
        
        for node in &ast {
            match &node.node_type {
                NodeType::ModuleImport { name } => {
                    if name == "math" {
                        found_math_import = true;
                    } else if name == "geometry" {
                        found_geometry_import = true;
                    }
                },
                NodeType::ImportDeclaration { module_path, items, import_all } => {
                    if module_path == &vec!["math".to_string()] && 
                       items == &vec!["add".to_string(), "subtract".to_string()] && 
                       !*import_all {
                        found_math_items_import = true;
                    } else if module_path == &vec!["geometry".to_string(), "shapes".to_string()] && 
                              *import_all {
                        found_geometry_shapes_import = true;
                    }
                },
                NodeType::ModuleDeclaration { name, .. } => {
                    if name == "app" {
                        found_app_module = true;
                    }
                },
                _ => {}
            }
        }
        
        assert!(found_math_import, "Missing math module import");
        assert!(found_geometry_import, "Missing geometry module import");
        assert!(found_math_items_import, "Missing math items import");
        assert!(found_geometry_shapes_import, "Missing geometry shapes import");
        assert!(found_app_module, "Missing app module declaration");
    }
}
```
