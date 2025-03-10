// Parser for the minimal LLM-friendly language

use crate::ast::{ASTNode, NodeType};
use crate::error::LangError;
use crate::lexer::{Token, TokenInfo};
use std::iter::Peekable;
use std::vec::IntoIter;
use log::{debug, info, trace};

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

    pub fn parse(&mut self) -> Result<ASTNode, LangError> {
        let nodes = self.parse_program()?;
        // Return first node since our main.rs expects single AST node
        nodes.into_iter().next().ok_or_else(|| LangError::syntax_error("Empty program"))
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
        let token_info = self.current_token()?.clone();
        let line = token_info.line;
        let column = token_info.column;

        match &token_info.token {
            Token::SymbolicKeyword('λ') => {
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

                Ok(ASTNode {
                    node_type: NodeType::Library {
                        name,
                        functions,
                    },
                    line,
                    column,
                })
            },
            Token::Identifier(_) => self.parse_identifier_statement(),
            Token::SymbolicKeyword('ι') => self.parse_variable_declaration(),
            Token::SymbolicKeyword('⟼') => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(ASTNode {
                    node_type: NodeType::Return(Some(Box::new(expr))),
                    line,
                    column,
                })
            },
            Token::SymbolicKeyword('⌽') => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(ASTNode {
                    node_type: NodeType::Print(Box::new(expr)),
                    line,
                    column,
                })
            },
            _ => self.parse_expression(),
        }
    }

    fn parse_identifier_statement(&mut self) -> Result<ASTNode, LangError> {
        let token_info = self.current.as_ref().unwrap();
        let line = token_info.line;
        let column = token_info.column;

        let name = match &token_info.token {
            Token::Identifier(name) => name.clone(),
            Token::SymbolicKeyword('∇') => "core".to_string(),
            Token::SymbolicKeyword('⌽') => "print".to_string(),
            Token::SymbolicKeyword(c) => c.to_string(),
            _ => return Err(LangError::syntax_error("Expected identifier or symbolic keyword")),
        };
        self.advance();

        let mut current_node = ASTNode {
            node_type: NodeType::Identifier(name),
            line,
            column,
        };

        // Handle chained method calls and property access
        loop {
            let (token, line, column) = match &self.current {
                Some(token_info) => (token_info.token.clone(), token_info.line, token_info.column),
                None => break,
            };

            match token {
                Token::Dot => {
                    self.advance(); // Skip the dot
                    
                    let property = match &self.current {
                        Some(TokenInfo { token: Token::Identifier(name), .. }) => name.clone(),
                        Some(TokenInfo { token: Token::SymbolicKeyword(sym), .. }) => sym.to_string(),
                        _ => return Err(LangError::syntax_error("Expected identifier or symbolic keyword after dot")),
                    };
                    self.advance();

                    current_node = ASTNode {
                        node_type: NodeType::PropertyAccess {
                            object: Box::new(current_node),
                            property: property.clone(),
                        },
                        line,
                        column,
                    };

                    // Check if this is a method call
                    if matches!(self.current.as_ref().map(|t| &t.token), Some(Token::Parenthesis('('))) {
                        debug!("Found method call to {}", property);
                        debug!("Current token before parsing method args: {:?}", self.current);
                        let arguments = self.parse_method_args()?;
                        debug!("Successfully parsed method call arguments");
                        current_node = ASTNode {
                            node_type: NodeType::MethodCall {
                                object: Box::new(current_node),
                                method: property,
                                arguments,
                            },
                            line,
                            column,
                        };
                    }
                },
                Token::Parenthesis('(') => {
                    // Direct function call without dot
                    let args = self.parse_arguments()?;
                    current_node = ASTNode {
                        node_type: NodeType::FunctionCall {
                            callee: Box::new(current_node),
                            arguments: args,
                        },
                        line,
                        column,
                    };
                },
                Token::SymbolicOperator('=') => break,
                Token::Semicolon => {
                    self.advance();
                    break;
                },
                _ => break,
            }
        }

        // Handle assignment if present
        if let Some(TokenInfo { token: Token::SymbolicOperator('='), .. }) = self.current {
            self.advance();
            let value = self.parse_expression()?;
            Ok(ASTNode {
                node_type: NodeType::Assignment {
                    name: match current_node.node_type {
                        NodeType::Identifier(name) => name,
                        _ => return Err(LangError::syntax_error("Invalid assignment target")),
                    },
                    value: Box::new(value),
                },
                line,
                column,
            })
        } else {
            Ok(current_node)
        }
    }

    fn parse_expression(&mut self) -> Result<ASTNode, LangError> {
        debug!("Entering parse_expression");
        let mut left = self.parse_primary()?;
        
        loop {
            let token_info = match &self.current {
                Some(info) => info.clone(),
                None => break,
            };

            match token_info.token {
                Token::Dot => {
                    debug!("Found dot operator, parsing method call");
                    self.advance(); // Skip the dot
                    
                    // Get the method name
                    let method = match &self.current {
                        Some(TokenInfo { token: Token::Identifier(name), .. }) => name.clone(),
                        Some(TokenInfo { token: Token::SymbolicKeyword(sym), .. }) => sym.to_string(),
                        _ => return Err(LangError::syntax_error("Expected method name after dot")),
                    };
                    self.advance();
                    
                    // Parse method arguments if present
                    if let Some(TokenInfo { token: Token::Parenthesis('('), .. }) = &self.current {
                        debug!("Found opening parenthesis for method arguments");
                        let args = self.parse_method_args()?;
                        left = ASTNode {
                            node_type: NodeType::MethodCall {
                                object: Box::new(left),
                                method,
                                arguments: args,
                            },
                            line: token_info.line,
                            column: token_info.column,
                        };
                    } else {
                        left = ASTNode {
                            node_type: NodeType::PropertyAccess {
                                object: Box::new(left),
                                property: method,
                            },
                            line: token_info.line,
                            column: token_info.column,
                        };
                    }
                },
                Token::SymbolicOperator(op) => {
                    debug!("Found operator {}, parsing binary expression", op);
                    self.advance();
                    let right = self.parse_primary()?;
                    left = ASTNode {
                        node_type: NodeType::Binary {
                            left: Box::new(left),
                            operator: Token::SymbolicOperator(op),
                            right: Box::new(right),
                        },
                        line: token_info.line,
                        column: token_info.column,
                    };
                },
                Token::Comma | Token::Parenthesis(')') | Token::Semicolon => {
                    // Let the caller handle commas, closing parentheses, and semicolons
                    debug!("Found comma, closing parenthesis, or semicolon in expression");
                    break;
                },
                _ => break,
            }
        }
        
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ASTNode, LangError> {
        debug!("Entering parse_primary");
        let token_info = match &self.current {
            Some(info) => info.clone(),
            None => return Err(LangError::syntax_error("Unexpected end of input")),
        };

        let node = match &token_info.token {
            Token::Number(n) => {
                self.advance();
                ASTNode {
                    node_type: NodeType::Number(*n),
                    line: token_info.line,
                    column: token_info.column,
                }
            },
            Token::StringLiteral(s) => {
                self.advance();
                ASTNode {
                    node_type: NodeType::String(s.clone()),
                    line: token_info.line,
                    column: token_info.column,
                }
            },
            Token::BooleanLiteral(b) => {
                self.advance();
                ASTNode {
                    node_type: NodeType::Boolean(*b),
                    line: token_info.line,
                    column: token_info.column,
                }
            },
            Token::Identifier(name) => {
                self.advance();
                ASTNode {
                    node_type: NodeType::Identifier(name.clone()),
                    line: token_info.line,
                    column: token_info.column,
                }
            },
            Token::SymbolicKeyword(c) => {
                self.advance();
                ASTNode {
                    node_type: NodeType::SymbolicKeyword(c.to_string()),
                    line: token_info.line,
                    column: token_info.column,
                }
            },
            Token::Parenthesis('(') => {
                debug!("Found opening parenthesis in primary expression");
                self.advance();
                let expr = self.parse_expression()?;
                
                // Handle comma-separated expressions in parentheses
                let mut exprs = vec![expr];
                while let Some(TokenInfo { token: Token::Comma, .. }) = &self.current {
                    debug!("Found comma in parenthesized expression");
                    self.advance(); // Skip comma
                    exprs.push(self.parse_expression()?);
                }
                
                match &self.current {
                    Some(TokenInfo { token: Token::Parenthesis(')'), .. }) => {
                        self.advance();
                        if exprs.len() == 1 {
                            exprs.remove(0)
                        } else {
                            ASTNode {
                                node_type: NodeType::FunctionCall {
                                    callee: Box::new(exprs[0].clone()),
                                    arguments: exprs[1..].to_vec(),
                                },
                                line: token_info.line,
                                column: token_info.column,
                            }
                        }
                    },
                    Some(token_info) => {
                        return Err(LangError::syntax_error_with_location(
                            "Expected closing parenthesis",
                            token_info.line,
                            token_info.column,
                        ));
                    },
                    None => return Err(LangError::syntax_error("Unexpected end of input")),
                }
            },
            _ => return Err(LangError::syntax_error_with_location(
                &format!("Unexpected token: {:?}", token_info.token),
                token_info.line,
                token_info.column,
            )),
        };

        Ok(node)
    }

    fn parse_variable_declaration(&mut self) -> Result<ASTNode, LangError> {
        // Skip the 'ι' token
        self.advance();

        let token_info = self.current.as_ref()
            .ok_or_else(|| LangError::syntax_error("Unexpected end of input"))?;
        let line = token_info.line;
        let column = token_info.column;

        let name = match &token_info.token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(LangError::syntax_error_with_location(
                "Expected variable name",
                line,
                column,
            )),
        };
        self.advance();

        self.expect(Token::SymbolicOperator('='))?;
        let value = self.parse_expression()?;

        if let Some(TokenInfo { token: Token::Semicolon, .. }) = self.current {
            self.advance();
        }

        Ok(ASTNode {
            node_type: NodeType::Assignment {
                name,
                value: Box::new(value),
            },
            line,
            column,
        })
    }

    fn parse_block(&mut self) -> Result<ASTNode, LangError> {
        debug!("Entering parse_block, current token: {:?}", self.current);
        
        // Get position info from the current token but don't consume it yet
        let token_info = match &self.current {
            Some(info) => info.clone(),
            None => return Err(LangError::syntax_error("Unexpected end of input")),
        };
        
        let line = token_info.line;
        let column = token_info.column;
        
        debug!("Token at start of block: {:?} at line {} column {}", token_info.token, line, column);
        
        let mut statements = Vec::new();

        // Check for opening brace and consume it
        match token_info.token {
            Token::CurlyBrace('{') => {
                debug!("Found opening brace at line {} column {}", line, column);
                self.advance();
            },
            _ => {
                debug!("Missing opening brace, found: {:?}", token_info.token);
                return Err(LangError::syntax_error_with_location(
                    &format!("Expected '{{', found {:?}", token_info.token),
                    line,
                    column,
                ));
            }
        };

        loop {
            match &self.current {
                Some(TokenInfo { token: Token::CurlyBrace('}'), .. }) => {
                    self.advance();  // Skip the closing brace
                    break;
                },
                Some(TokenInfo { token, .. }) => {
                    // Parse any valid statement or expression
                    let statement = match token {
                        Token::SymbolicKeyword('λ') => self.parse_lambda()?,
                        Token::SymbolicKeyword('ƒ') => self.parse_function_declaration()?,
                        Token::Identifier(_) | Token::SymbolicKeyword(_) => {
                            let expr = self.parse_expression()?;
                            // Check for semicolon after expression
                            if let Some(TokenInfo { token: Token::Semicolon, .. }) = &self.current {
                                self.advance();
                            }
                            expr
                        },
                        _ => {
                            let expr = self.parse_expression()?;
                            // Check for semicolon after expression
                            if let Some(TokenInfo { token: Token::Semicolon, .. }) = &self.current {
                                self.advance();
                            }
                            expr
                        },
                    };
                    statements.push(statement);
                },
                None => return Err(LangError::syntax_error("Unexpected end of input")),
            }
        }

        Ok(ASTNode {
            node_type: NodeType::Block(statements),
            line,
            column,
        })
    }

    fn parse_method_args(&mut self) -> Result<Vec<ASTNode>, LangError> {
        debug!("Entering parse_method_args");
        let mut args = Vec::new();
        
        // Expect and consume opening parenthesis
        debug!("Expecting opening parenthesis for method args");
        match &self.current {
            Some(TokenInfo { token: Token::Parenthesis('('), .. }) => {
                self.advance();
            },
            _ => return Err(LangError::syntax_error("Expected '(' for method arguments")),
        }
        
        // Handle empty argument list
        if let Some(TokenInfo { token: Token::Parenthesis(')'), .. }) = &self.current {
            debug!("Found empty argument list");
            self.advance();
            return Ok(args);
        }
        
        // Parse first argument
        debug!("Parsing first argument");
        args.push(self.parse_expression()?);
        
        // Parse remaining arguments
        loop {
            match &self.current {
                Some(TokenInfo { token: Token::Parenthesis(')'), .. }) => {
                    debug!("Found closing parenthesis - end of argument list");
                    self.advance();
                    break;
                },
                Some(TokenInfo { token: Token::Comma, .. }) => {
                    debug!("Found comma - parsing next argument");
                    self.advance(); // Skip the comma
                    
                    // Check for trailing comma
                    if let Some(TokenInfo { token: Token::Parenthesis(')'), .. }) = &self.current {
                        debug!("Found trailing comma");
                        return Err(LangError::syntax_error("Unexpected trailing comma in argument list"));
                    }
                    
                    // Parse next argument
                    debug!("Parsing next argument after comma");
                    args.push(self.parse_expression()?);
                },
                Some(token_info) => {
                    return Err(LangError::syntax_error_with_location(
                        "Expected comma or closing parenthesis after argument",
                        token_info.line,
                        token_info.column,
                    ));
                },
                None => {
                    return Err(LangError::syntax_error("Unexpected end of input in argument list"));
                }
            }
        }
        
        debug!("Successfully parsed {} method arguments", args.len());
        Ok(args)
    }

    fn parse_arguments(&mut self) -> Result<Vec<ASTNode>, LangError> {
        self.parse_method_args()
    }

    fn current_token(&mut self) -> Result<&TokenInfo, LangError> {
        self.current.as_ref()
            .ok_or_else(|| LangError::syntax_error("Unexpected end of input"))
    }

    fn parse_lambda(&mut self) -> Result<ASTNode, LangError> {
        let start_pos = self.current.as_ref().map(|t| t.line).unwrap_or(0);
        let start_col = self.current.as_ref().map(|t| t.column).unwrap_or(0);
        
        // Skip the lambda symbol
        self.advance();
        
        // Parse parameters
        let mut params = Vec::new();
        
        // Handle both single parameter and multiple parameter cases
        match &self.current {
            Some(t) => match &t.token {
                Token::Identifier(name) if name == "_" => {
                    // Anonymous parameter case
                    self.advance();
                },
                Token::Identifier(name) => {
                    // Single parameter case
                    params.push(name.clone());
                    self.advance();
                },
                Token::SymbolicKeyword(c) => {
                    // Single symbolic parameter case
                    params.push(c.to_string());
                    self.advance();
                },
                Token::Parenthesis('(') => {
                    self.advance(); // Skip opening parenthesis
                    let mut expecting_parameter = true;

                    loop {
                        let (token, line, column) = match &self.current {
                            Some(token_info) => (token_info.token.clone(), token_info.line, token_info.column),
                            None => break,
                        };

                        match token {
                            Token::Parenthesis(')') if !expecting_parameter => {
                                self.advance();
                                break;
                            },
                            Token::Identifier(name) if expecting_parameter => {
                                params.push(name);
                                expecting_parameter = false;
                                self.advance();
                            },
                            Token::SymbolicKeyword(c) if expecting_parameter => {
                                params.push(c.to_string());
                                expecting_parameter = false;
                                self.advance();
                            },
                            Token::Comma if !expecting_parameter => {
                                expecting_parameter = true;
                                self.advance();
                            },
                            _ => {
                                return Err(LangError::syntax_error_with_location(
                                    if expecting_parameter {
                                        "Expected parameter name"
                                    } else {
                                        "Expected comma or closing parenthesis"
                                    },
                                    line,
                                    column,
                                ));
                            }
                        }
                    }
                },
                _ => return Err(LangError::syntax_error("Expected parameter name or opening parenthesis after lambda")),
            },
            None => return Err(LangError::syntax_error("Unexpected end of input after lambda")),
        }
        
        // Expect opening brace
        match &self.current {
            Some(t) => match &t.token {
                Token::CurlyBrace('{') => self.advance(),
                _ => return Err(LangError::syntax_error("Expected { after lambda parameters")),
            },
            None => return Err(LangError::syntax_error("Unexpected end of input")),
        }
        
        // Parse the block
        let body = self.parse_block()?;
        
        // Create the lambda node with all parameters
        let lambda = ASTNode {
            node_type: NodeType::Lambda {
                params,
                body: Box::new(body),
            },
            line: start_pos,
            column: start_col,
        };
        
        Ok(lambda)
    }

    fn parse_function_declaration(&mut self) -> Result<ASTNode, LangError> {
        // Start position is from the current token
        let start_token = self.current_token()?.clone();

        // First token should be 'ƒ'
        match &start_token.token {
            Token::SymbolicKeyword('ƒ') => self.advance(),
            _ => return Err(LangError::syntax_error("Expected 'ƒ' at start of function declaration")),
        }

        // Parse function name
        let name_token = self.current_token()?.clone();

        let func_name = match name_token.token {
            Token::Identifier(id) => id,
            Token::SymbolicKeyword(ch) => ch.to_string(),
            _ => return Err(LangError::syntax_error("Invalid function name")),
        };
        self.advance(); // Move past function name

        // Parse parameters: (p1, p2, ...)
        let mut params = Vec::new();
        if let Some(TokenInfo { token: Token::Parenthesis('('), .. }) = &self.current {
            self.advance(); // Skip opening parenthesis
        } else {
            return Err(LangError::syntax_error("Expected '(' after function name"));
        }
        
        if let Some(TokenInfo { token: Token::Parenthesis(')'), .. }) = &self.current {
            self.advance(); // Empty parameter list
        } else {
            loop {
                match &self.current {
                    Some(TokenInfo { token: Token::Identifier(name), .. }) => {
                        params.push(name.clone());
                        self.advance();
                    },
                    Some(TokenInfo { token: Token::SymbolicKeyword(c), .. }) => {
                        params.push(c.to_string());
                        self.advance();
                    },
                    _ => return Err(LangError::syntax_error("Expected parameter name")),
                }

                match &self.current {
                    Some(TokenInfo { token: Token::Parenthesis(')'), .. }) => {
                        self.advance();
                        break;
                    },
                    Some(TokenInfo { token: Token::Comma, .. }) => {
                        self.advance();
                        continue;
                    },
                    _ => return Err(LangError::syntax_error("Expected ',' or ')' in parameter list")),
                }
            }
        }

        // Parse the function body
        match &self.current {
            Some(token_info) => {
                debug!("Current token before block: {:?}", token_info.token);
                
                // Make sure we have a curly brace
                if !matches!(token_info.token, Token::CurlyBrace('{')) {
                    return Err(LangError::syntax_error_with_location(
                        &format!("Expected '{{' before function body, found {:?}", token_info.token),
                        token_info.line,
                        token_info.column,
                    ));
                }
                
                let body = self.parse_block()?;
                Ok(ASTNode {
                    node_type: NodeType::FunctionDeclaration {
                        name: func_name,
                        parameters: params,
                        body: Box::new(body),
                    },
                    line: start_token.line,
                    column: start_token.column,
                })
            },
            None => Err(LangError::syntax_error("Unexpected end of input")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let tokens = vec![
            TokenInfo {
                token: Token::Number(42),
                line: 1,
                column: 1,
                start_pos: 0,
                end_pos: 2,
            },
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse_program().unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].node_type {
            NodeType::Number(n) => assert_eq!(*n, 42),
            _ => panic!("Expected number node"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let tokens = vec![
            TokenInfo {
                token: Token::Number(1),
                line: 1,
                column: 1,
                start_pos: 0,
                end_pos: 1,
            },
            TokenInfo {
                token: Token::SymbolicOperator('+'),
                line: 1,
                column: 2,
                start_pos: 1,
                end_pos: 2,
            },
            TokenInfo {
                token: Token::Number(2),
                line: 1,
                column: 3,
                start_pos: 2,
                end_pos: 3,
            },
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse_program().unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].node_type {
            NodeType::Binary { .. } => (),
            _ => panic!("Expected binary node"),
        }
    }

    #[test]
    fn test_parse_variable_assignment() {
        let tokens = vec![
            TokenInfo {
                token: Token::SymbolicKeyword('ι'),
                line: 1,
                column: 1,
                start_pos: 0,
                end_pos: 1,
            },
            TokenInfo {
                token: Token::Identifier("x".to_string()),
                line: 1,
                column: 2,
                start_pos: 1,
                end_pos: 2,
            },
            TokenInfo {
                token: Token::SymbolicOperator('='),
                line: 1,
                column: 3,
                start_pos: 2,
                end_pos: 3,
            },
            TokenInfo {
                token: Token::Number(42),
                line: 1,
                column: 4,
                start_pos: 3,
                end_pos: 5,
            },
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse_program().unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].node_type {
            NodeType::Assignment { name, .. } => assert_eq!(name, "x"),
            _ => panic!("Expected assignment node"),
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let tokens = vec![
            TokenInfo { token: Token::SymbolicKeyword('ƒ'), line: 1, column: 1, start_pos: 0, end_pos: 1 },
            TokenInfo { token: Token::Identifier("test".to_string()), line: 1, column: 3, start_pos: 2, end_pos: 6 },
            TokenInfo { token: Token::Parenthesis('('), line: 1, column: 7, start_pos: 6, end_pos: 7 },
            TokenInfo { token: Token::Parenthesis(')'), line: 1, column: 8, start_pos: 7, end_pos: 8 },
            TokenInfo { token: Token::CurlyBrace('{'), line: 1, column: 10, start_pos: 9, end_pos: 10 },
            TokenInfo { token: Token::CurlyBrace('}'), line: 1, column: 11, start_pos: 10, end_pos: 11 },
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse_program().unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].node_type {
            NodeType::FunctionDeclaration { name, .. } => assert_eq!(name, "test"),
            _ => panic!("Expected function declaration node"),
        }
    }
}
