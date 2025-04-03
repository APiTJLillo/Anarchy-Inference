// src/parser.rs - Modified to support Lexer integration
// Parser for the minimal LLM-friendly language

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use crate::lexer::{Token, TokenInfo, Lexer};
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<TokenInfo>>,
    current: Option<TokenInfo>,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            current: None,
        };
        parser.advance();
        parser
    }
    
    // Create a parser from a lexer
    pub fn from_lexer(mut lexer: Lexer) -> Result<Self, LangError> {
        let tokens = lexer.tokenize()?;
        Ok(Self::new(tokens))
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
        self.parse_program()
    }

    pub fn current_token(&self) -> Result<&TokenInfo, LangError> {
        self.current.as_ref().ok_or_else(|| LangError::syntax_error("Unexpected end of input"))
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, LangError> {
        let mut nodes = Vec::new();

        while let Ok(token_info) = self.current_token() {
            match &token_info.token {
                Token::EOF => break,
                Token::SymbolicKeyword('λ') => {
                    // Parse library declaration
                    let line = token_info.line;
                    let column = token_info.column;
                    
                    // Consume the λ token
                    self.advance();

                    // Next token must be an identifier for the library name
                    let name = match self.current_token()? {
                        TokenInfo { token: Token::Identifier(name), .. } => {
                            let name = name.clone();
                            self.advance();
                            name
                        },
                        token_info => {
                            return Err(LangError::syntax_error_with_location(
                                &format!("Expected library name after λ, found {}", token_info.token),
                                token_info.line,
                                token_info.column,
                            ));
                        }
                    };

                    // Expect opening curly brace
                    match self.current_token()? {
                        TokenInfo { token: Token::CurlyBrace('{'), .. } => {
                            self.advance();
                        },
                        token_info => {
                            return Err(LangError::syntax_error_with_location(
                                &format!("Expected '{{' after library name, found {}", token_info.token),
                                token_info.line,
                                token_info.column,
                            ));
                        }
                    }

                    // Parse library contents until closing brace
                    let mut functions = Vec::new();
                    while let Ok(token_info) = self.current_token() {
                        match &token_info.token {
                            Token::CurlyBrace('}') => {
                                self.advance();
                                break;
                            },
                            Token::SymbolicKeyword('ƒ') => {
                                // Parse function declaration
                                let func = self.parse_function_declaration()?;
                                functions.push(func);

                                // Expect semicolon after function declaration
                                match self.current_token()? {
                                    TokenInfo { token: Token::Semicolon, .. } => {
                                        self.advance();
                                    },
                                    token_info => {
                                        return Err(LangError::syntax_error_with_location(
                                            "Expected semicolon after function declaration",
                                            token_info.line,
                                            token_info.column,
                                        ));
                                    }
                                }
                            },
                            _ => {
                                // Skip non-function tokens
                                self.advance();
                            }
                        }
                    }

                    nodes.push(ASTNode {
                        node_type: NodeType::Library {
                            name,
                            functions,
                        },
                        line,
                        column,
                    });
                },
                Token::SymbolicKeyword('ƒ') => {
                    // Parse function declaration
                    let func = self.parse_function_declaration()?;
                    nodes.push(func);
                },
                _ => {
                    let stmt = self.parse_statement()?;
                    nodes.push(stmt);
                }
            }
        }

        Ok(nodes)
    }

    fn parse_statement(&mut self) -> Result<ASTNode, LangError> {
        // Implementation omitted for brevity
        // This is a placeholder to make the code compile
        let token_info = self.current_token()?.clone();
        self.advance();
        
        Ok(ASTNode {
            node_type: NodeType::Null,
            line: token_info.line,
            column: token_info.column,
        })
    }
    
    fn parse_function_declaration(&mut self) -> Result<ASTNode, LangError> {
        // Implementation omitted for brevity
        // This is a placeholder to make the code compile
        let token_info = self.current_token()?.clone();
        self.advance();
        
        Ok(ASTNode {
            node_type: NodeType::Null,
            line: token_info.line,
            column: token_info.column,
        })
    }
}
