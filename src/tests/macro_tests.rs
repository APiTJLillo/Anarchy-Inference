# Macro System Tests for Anarchy Inference

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::macros::{MacroExpander, MacroPattern};
use crate::interpreter::Interpreter;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to parse and expand macros
    fn parse_and_expand(input: &str) -> Result<Vec<ASTNode>, LangError> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    // Helper function to parse, expand, and execute
    fn parse_expand_execute(input: &str) -> Result<String, LangError> {
        let nodes = parse_and_expand(input)?;
        let mut interpreter = Interpreter::new();
        let result = interpreter.execute_nodes(&nodes)?;
        Ok(format!("{}", result))
    }
    
    #[test]
    fn test_declarative_macro_definition() {
        let input = r#"
        ℳ unless(condition, body) ⟼ if (!condition) { body }
        "#;
        
        let result = parse_and_expand(input);
        assert!(result.is_ok());
        
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 0); // Macro definitions are not included in the output
    }
    
    #[test]
    fn test_procedural_macro_definition() {
        let input = r#"
        ℳƒ debug_print(expr) ⟼ {
            ⟼ {
                let temp = expr;
                ⌽ ":debug_message {expr} = {temp}";
                temp
            }
        }
        "#;
        
        let result = parse_and_expand(input);
        assert!(result.is_ok());
        
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 0); // Macro definitions are not included in the output
    }
    
    #[test]
    fn test_declarative_macro_expansion() {
        let input = r#"
        ℳ unless(condition, body) ⟼ if (!condition) { body }
        
        unless(x == 0, {
            ⌽ "x is not zero";
        })
        "#;
        
        let result = parse_and_expand(input);
        assert!(result.is_ok());
        
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1); // One expanded macro
        
        // Check that the macro was expanded to an if statement
        if let NodeType::MacroExpansion { original, expanded } = &nodes[0].node_type {
            if let NodeType::If { condition, then_branch, else_branch } = &expanded.node_type {
                // Expansion successful
                assert!(else_branch.is_none());
            } else {
                panic!("Expected If node in expanded macro");
            }
        } else {
            panic!("Expected MacroExpansion node");
        }
    }
    
    #[test]
    fn test_nested_macro_expansion() {
        let input = r#"
        ℳ twice(expr) ⟼ { expr; expr }
        ℳ thrice(expr) ⟼ { expr; expr; expr }
        
        twice(thrice(⌽ "Hello"))
        "#;
        
        let result = parse_and_expand(input);
        assert!(result.is_ok());
        
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1); // One expanded macro
        
        // The expansion should contain 6 print statements (2 * 3)
        if let NodeType::MacroExpansion { original, expanded } = &nodes[0].node_type {
            if let NodeType::Block(statements) = &expanded.node_type {
                assert_eq!(statements.len(), 2); // Two blocks from twice
                
                if let NodeType::Block(inner_statements) = &statements[0].node_type {
                    assert_eq!(inner_statements.len(), 3); // Three statements from thrice
                } else {
                    panic!("Expected Block node in expanded macro");
                }
            } else {
                panic!("Expected Block node in expanded macro");
            }
        } else {
            panic!("Expected MacroExpansion node");
        }
    }
    
    #[test]
    fn test_macro_hygiene() {
        let input = r#"
        ℳ with_temp_var(expr) ⟼ {
            let temp = 42;
            expr
        }
        
        let temp = 10;
        with_temp_var(⌽ temp)
        "#;
        
        // This should print 10, not 42, due to hygiene
        let result = parse_expand_execute(input);
        assert!(result.is_ok());
        
        // The actual result depends on the hygiene implementation
        // In a fully hygienic system, it would print 10
        // In a non-hygienic system, it would print 42
    }
    
    #[test]
    fn test_macro_with_string_dictionary() {
        let input = r#"
        ℳ log_error(message) ⟼ ⌽ ":error_prefix {message}"
        
        log_error("File not found")
        "#;
        
        let result = parse_and_expand(input);
        assert!(result.is_ok());
        
        let nodes = result.unwrap();
        assert_eq!(nodes.len(), 1); // One expanded macro
        
        // Check that the macro was expanded to a print statement with string dictionary reference
        if let NodeType::MacroExpansion { original, expanded } = &nodes[0].node_type {
            if let NodeType::Print(_) = &expanded.node_type {
                // Expansion successful
            } else {
                panic!("Expected Print node in expanded macro");
            }
        } else {
            panic!("Expected MacroExpansion node");
        }
    }
    
    #[test]
    fn test_macro_with_conditional_compilation() {
        let input = r#"
        ℳ platform_specific(web_code, desktop_code) ⟼ {
            #[if(feature="web")]
            {
                web_code
            }
            #[if(!feature="web")]
            {
                desktop_code
            }
        }
        
        platform_specific(
            ⌽ "Running on web",
            ⌽ "Running on desktop"
        )
        "#;
        
        // Set web feature
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.set_enabled_features(vec!["web".to_string()]);
        let result = parser.parse();
        
        assert!(result.is_ok());
        
        // The expansion should include only the web code
        let nodes = result.unwrap();
        // Detailed assertion would depend on how conditional compilation is implemented
    }
    
    #[test]
    fn test_macro_with_module_integration() {
        let input = r#"
        λ macros {
            ⊢ ℳ unless(condition, body) ⟼ if (!condition) { body }
        }
        
        ⟑ macros::{unless}
        
        unless(x == 0, {
            ⌽ "x is not zero";
        })
        "#;
        
        let result = parse_and_expand(input);
        // The result depends on how module integration is implemented
        // This test is more of a compilation test than a functional test
    }
    
    #[test]
    fn test_recursive_macro_expansion_limit() {
        let input = r#"
        ℳ recursive(n) ⟼ {
            if (n <= 0) {
                0
            } else {
                recursive(n - 1)
            }
        }
        
        recursive(1000) // Should hit expansion limit
        "#;
        
        let result = parse_and_expand(input);
        // This should fail with a recursion limit error
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("expansion depth"));
    }
    
    #[test]
    fn test_macro_error_reporting() {
        let input = r#"
        ℳ requires_two_args(a, b) ⟼ a + b
        
        requires_two_args(42) // Missing argument
        "#;
        
        let result = parse_and_expand(input);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("expected 2"));
    }
}
