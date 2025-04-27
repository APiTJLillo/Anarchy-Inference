// src/parser.rs - Modified to support macro system
// Parser for the minimal LLM-friendly language

use crate::ast::{ASTNode, NodeType, VersionConstraint};
use crate::error::LangError;
use crate::lexer::{Token, TokenInfo, Lexer};
use crate::macros::{MacroExpander, MacroPattern};
// Use direct implementation instead of importing the problematic module
mod local_implicit_types {
    pub fn is_implicit_cast_allowed(_from_type: &str, _to_type: &str) -> bool {
        // Simple implementation that allows all casts for now
        true
    }
}
use local_implicit_types as implicit_types;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::collections::HashMap;

pub struct Parser {
    tokens: Peekable<IntoIter<TokenInfo>>,
    current: Option<TokenInfo>,
    // Flag to enable implicit type inference
    implicit_types: bool,
    // Track enabled features for conditional compilation
    enabled_features: Vec<String>,
    // Macro expander for handling macros
    macro_expander: Option<MacroExpander>,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            current: None,
            implicit_types: true, // Enable implicit type inference by default
            enabled_features: Vec::new(),
            macro_expander: Some(MacroExpander::new()),
        };
        parser.advance();
        parser
    }
    
    // Create a parser from a lexer
    pub fn from_lexer(mut lexer: Lexer) -> Result<Self, LangError> {
        let tokens = lexer.tokenize()?;
        Ok(Self::new(tokens))
    }
    
    // Enable or disable implicit type inference
    pub fn set_implicit_types(&mut self, enabled: bool) {
        self.implicit_types = enabled;
    }
    
    // Set enabled features for conditional compilation
    pub fn set_enabled_features(&mut self, features: Vec<String>) {
        self.enabled_features = features;
    }
    
    // Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.enabled_features.contains(&feature.to_string())
    }
    
    // Evaluate a feature condition
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        // Simple condition evaluation for now
        // Format: feature="name" or !feature="name"
        if condition.starts_with("feature=") {
            let feature = condition.trim_start_matches("feature=")
                .trim_matches('"');
            self.is_feature_enabled(feature)
        } else if condition.starts_with("!feature=") {
            let feature = condition.trim_start_matches("!feature=")
                .trim_matches('"');
            !self.is_feature_enabled(feature)
        } else {
            // Default to true for unknown conditions
            true
        }
    }

    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    fn peek(&mut self) -> Option<&TokenInfo> {
        self.tokens.peek()
    }

    fn expect(&mut self, expected: Token) -> Result<(), LangError> {
        if let Some(TokenInfo { token, line, column, .. }) = &self.current {
            if *token == expected {
                self.advance();
                Ok(())
            } else {
                Err(LangError::syntax_error_with_location(
                    &format!("Expected {:?}, found {:?}", expected, token),
                    *line,
                    *column,
                ))
            }
        } else {
            Err(LangError::syntax_error("Unexpected end of input"))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ASTNode>, LangError> {
        let nodes = self.parse_program()?;
        
        // Process macros if the expander is available
        if let Some(expander) = &mut self.macro_expander {
            expander.process(&nodes)
        } else {
            Ok(nodes)
        }
    }

    pub fn current_token(&self) -> Result<&TokenInfo, LangError> {
        self.current.as_ref().ok_or_else(|| LangError::syntax_error("Unexpected end of input"))
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, LangError> {
        let mut nodes = Vec::new();
        let mut documentation = None;

        while let Ok(token_info) = self.current_token() {
            match &token_info.token {
                Token::EOF => break,
                // Handle documentation comments
                Token::StringLiteral(s) if s.starts_with("///") => {
                    let doc = s.trim_start_matches("///").trim().to_string();
                    documentation = Some(doc);
                    self.advance();
                    continue;
                },
                // Handle macro definitions
                Token::MacroKeyword => {
                    let line = token_info.line;
                    let column = token_info.column;
                    self.advance();
                    
                    // Parse the macro definition
                    let macro_def = self.parse_macro_definition(false)?;
                    
                    if let Some(doc) = documentation.take() {
                        let mut macro_def = macro_def;
                        macro_def.documentation = Some(doc);
                        nodes.push(macro_def);
                    } else {
                        nodes.push(macro_def);
                    }
                    
                    continue;
                },
                // Handle procedural macro definitions
                Token::ProceduralMacroKeyword => {
                    let line = token_info.line;
                    let column = token_info.column;
                    self.advance();
                    
                    // Parse the procedural macro definition
                    let macro_def = self.parse_macro_definition(true)?;
                    
                    if let Some(doc) = documentation.take() {
                        let mut macro_def = macro_def;
                        macro_def.documentation = Some(doc);
                        nodes.push(macro_def);
                    } else {
                        nodes.push(macro_def);
                    }
                    
                    continue;
                },
                // Handle conditional compilation attributes
                Token::Attribute(attr) => {
                    let line = token_info.line;
                    let column = token_info.column;
                    self.advance();
                    
                    // Check if this is a conditional block
                    if attr.starts_with("if(") && attr.ends_with(")") {
                        let condition = attr.trim_start_matches("if(")
                            .trim_end_matches(")")
                            .trim()
                            .to_string();
                        
                        // Parse the conditional block
                        let items = if self.evaluate_condition(&condition) {
                            // Condition is true, parse the block normally
                            self.parse_block()?
                        } else {
                            // Condition is false, skip the block
                            self.skip_block()?;
                            Vec::new()
                        };
                        
                        // Add the conditional block node
                        nodes.push(ASTNode::new(
                            NodeType::ConditionalBlock {
                                condition,
                                items,
                            },
                            line,
                            column
                        ));
                        
                        continue;
                    }
                    
                    // Other attributes are handled with their associated nodes
                    // Store the attribute for the next node
                    let next_attr = Some(attr.clone());
                    
                    // Continue to the next token
                    continue;
                },
                // Handle other tokens as before
                _ => {
                    // Check if this is a macro invocation
                    if let Some(macro_invocation) = self.try_parse_macro_invocation()? {
                        nodes.push(macro_invocation);
                        continue;
                    }
                    
                    // Handle other node types as before
                    // ...
                }
            }
            
            // If we get here, it's not a special token, so parse a statement
            let statement = self.parse_statement()?;
            nodes.push(statement);
        }

        Ok(nodes)
    }
    
    // Parse a macro definition
    fn parse_macro_definition(&mut self, is_procedural: bool) -> Result<ASTNode, LangError> {
        let line = self.current_token()?.line;
        let column = self.current_token()?.column;
        
        // Parse the macro name
        let name = match self.current_token()?.token {
            Token::Identifier(ref name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => {
                return Err(LangError::syntax_error_with_location(
                    "Expected macro name",
                    line,
                    column,
                ));
            }
        };
        
        // Parse the pattern
        let pattern = self.parse_macro_pattern()?;
        
        // Expect the ⟼ token
        match self.current_token()?.token {
            Token::SymbolicKeyword('⟼') => {
                self.advance();
            },
            _ => {
                return Err(LangError::syntax_error_with_location(
                    "Expected ⟼ after macro pattern",
                    self.current_token()?.line,
                    self.current_token()?.column,
                ));
            }
        }
        
        // Parse the template
        let template = if is_procedural {
            // Procedural macros have a block body
            self.parse_block_expression()?
        } else {
            // Declarative macros have a template expression
            self.parse_expression()?
        };
        
        Ok(ASTNode::new(
            NodeType::MacroDefinition {
                name,
                pattern: Box::new(pattern),
                template: Box::new(template),
                is_procedural,
            },
            line,
            column,
        ))
    }
    
    // Parse a macro pattern
    fn parse_macro_pattern(&mut self) -> Result<ASTNode, LangError> {
        let line = self.current_token()?.line;
        let column = self.current_token()?.column;
        
        // Expect opening parenthesis
        self.expect(Token::Parenthesis('('))?;
        
        // Parse pattern variables
        let mut variables = Vec::new();
        
        // Parse first variable
        if let Token::Identifier(ref name) = self.current_token()?.token {
            variables.push(name.clone());
            self.advance();
        } else {
            return Err(LangError::syntax_error_with_location(
                "Expected pattern variable",
                self.current_token()?.line,
                self.current_token()?.column,
            ));
        }
        
        // Parse additional variables
        while let Ok(token_info) = self.current_token() {
            if token_info.token == Token::Parenthesis(')') {
                break;
            }
            
            // Expect comma
            self.expect(Token::Comma)?;
            
            // Parse variable
            if let Token::Identifier(ref name) = self.current_token()?.token {
                variables.push(name.clone());
                self.advance();
            } else {
                return Err(LangError::syntax_error_with_location(
                    "Expected pattern variable",
                    self.current_token()?.line,
                    self.current_token()?.column,
                ));
            }
        }
        
        // Expect closing parenthesis
        self.expect(Token::Parenthesis(')'))?;
        
        // Create a pattern node
        let pattern_node = ASTNode::new(
            NodeType::MacroPattern {
                variables,
                pattern: Box::new(ASTNode::new(
                    NodeType::Block(Vec::new()),
                    line,
                    column,
                )),
            },
            line,
            column,
        );
        
        Ok(pattern_node)
    }
    
    // Try to parse a macro invocation
    fn try_parse_macro_invocation(&mut self) -> Result<Option<ASTNode>, LangError> {
        // Check if this is a macro invocation
        if let Token::Identifier(ref name) = self.current_token()?.token {
            // Save the current position
            let line = self.current_token()?.line;
            let column = self.current_token()?.column;
            let name = name.clone();
            
            // Check if the macro exists
            if let Some(expander) = &self.macro_expander {
                if expander.get_macro(&name).is_some() {
                    // This is a macro invocation
                    self.advance();
                    
                    // Parse arguments
                    let mut arguments = Vec::new();
                    
                    // Expect opening parenthesis
                    self.expect(Token::Parenthesis('('))?;
                    
                    // Parse arguments
                    if self.current_token()?.token != Token::Parenthesis(')') {
                        // Parse first argument
                        arguments.push(self.parse_expression()?);
                        
                        // Parse additional arguments
                        while self.current_token()?.token == Token::Comma {
                            self.advance();
                            arguments.push(self.parse_expression()?);
                        }
                    }
                    
                    // Expect closing parenthesis
                    self.expect(Token::Parenthesis(')'))?;
                    
                    // Create a macro invocation node
                    return Ok(Some(ASTNode::new(
                        NodeType::MacroInvocation {
                            name,
                            arguments,
                        },
                        line,
                        column,
                    )));
                }
            }
        }
        
        // Not a macro invocation
        Ok(None)
    }
    
    // Other parsing methods remain the same
    // ...
}
