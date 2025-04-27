// src/lexer.rs - Modified to add macro system support
// Lexer for the minimal LLM-friendly language with module system and macro support

use crate::error::LangError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    StringLiteral(String),
    BooleanLiteral(bool),
    Identifier(String),
    SymbolicOperator(char),
    SymbolicKeyword(char),
    StringDictRef(String),  // String dictionary references
    UserInput,              // User input emoji (🎤)
    Parenthesis(char),
    CurlyBrace(char),
    SquareBracket(char),    // Added for attribute syntax
    AngleBracket(char),     // Added for file-based module imports
    Comma,
    Semicolon,
    Dot,
    DoubleColon,            // Added for module path resolution (::)
    As,                     // Added for module aliases
    Version(String),        // Added for module versioning (v"1.0.0")
    Attribute(String),      // Added for conditional compilation (#[feature="web"])
    MacroKeyword,           // Added for macro definition (ℳ)
    ProceduralMacroKeyword, // Added for procedural macro definition (ℳƒ)
    EOF,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::BooleanLiteral(b) => write!(f, "{}", if *b { "⊤" } else { "⊥" }),
            Token::Identifier(name) => write!(f, "{}", name),
            Token::SymbolicOperator(c) => write!(f, "{}", c),
            Token::SymbolicKeyword(c) => write!(f, "{}", c),
            Token::StringDictRef(key) => write!(f, ":{}", key),
            Token::UserInput => write!(f, "🎤"),
            Token::Parenthesis(c) => write!(f, "{}", c),
            Token::CurlyBrace(c) => write!(f, "{}", c),
            Token::SquareBracket(c) => write!(f, "{}", c),
            Token::AngleBracket(c) => write!(f, "{}", c),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Dot => write!(f, "."),
            Token::DoubleColon => write!(f, "::"),
            Token::As => write!(f, "as"),
            Token::Version(v) => write!(f, "v\"{}\"", v),
            Token::Attribute(a) => write!(f, "#[{}]", a),
            Token::MacroKeyword => write!(f, "ℳ"),
            Token::ProceduralMacroKeyword => write!(f, "ℳƒ"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// A safer Lexer that stores the entire input as a `Vec<char>` and tracks
/// position by "characters", not by UTF‑8 byte indices. This prevents
/// partial slicing errors when multi‑byte symbols appear.
pub struct Lexer {
    chars: Vec<char>,     // All characters in the input.
    position: usize,      // Current index in `chars`, not bytes
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        // Convert the entire input into a char vector:
        let chars: Vec<char> = input.chars().collect();

        Self {
            chars,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Turn the entire input into a list of TokenInfo.
    pub fn tokenize(&mut self) -> Result<Vec<TokenInfo>, LangError> {
        let mut tokens = Vec::new();

        loop {
            if let Some(tok) = self.next_token()? {
                if tok.token == Token::EOF {
                    tokens.push(tok);
                    break;
                } else {
                    tokens.push(tok);
                }
            } else {
                break;
            }
        }

        Ok(tokens)
    }

    /// Get the next token from the input.
    pub fn next_token(&mut self) -> Result<Option<TokenInfo>, LangError> {
        self.skip_whitespace();

        if self.position >= self.chars.len() {
            return Ok(Some(TokenInfo {
                token: Token::EOF,
                line: self.line,
                column: self.column,
                start_pos: self.position,
                end_pos: self.position,
            }));
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;

        let c = self.chars[self.position];
        let token = match c {
            '0'..='9' => {
                let num = self.read_number()?;
                Token::Number(num)
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "as" => Token::As,
                    _ => Token::Identifier(ident),
                }
            },
            '"' => {
                let s = self.read_string()?;
                Token::StringLiteral(s)
            },
            ':' => {
                self.advance();
                let key = self.read_identifier();
                Token::StringDictRef(key)
            },
            '(' | ')' => {
                self.advance();
                Token::Parenthesis(c)
            },
            '{' | '}' => {
                self.advance();
                Token::CurlyBrace(c)
            },
            '[' | ']' => {
                self.advance();
                Token::SquareBracket(c)
            },
            '<' | '>' => {
                self.advance();
                Token::AngleBracket(c)
            },
            ',' => {
                self.advance();
                Token::Comma
            },
            ';' => {
                self.advance();
                Token::Semicolon
            },
            '.' => {
                self.advance();
                // Check for double colon
                if self.peek() == Some('.') {
                    self.advance();
                    Token::DoubleColon
                } else {
                    Token::Dot
                }
            },
            '#' => {
                self.advance();
                if self.peek() == Some('[') {
                    self.advance();
                    let attr = self.read_until(']');
                    if self.peek() == Some(']') {
                        self.advance();
                        Token::Attribute(attr)
                    } else {
                        return Err(LangError::syntax_error_with_location(
                            "Unterminated attribute",
                            start_line,
                            start_column,
                        ));
                    }
                } else {
                    return Err(LangError::syntax_error_with_location(
                        "Expected '[' after '#'",
                        start_line,
                        start_column,
                    ));
                }
            },
            'v' => {
                self.advance();
                if self.peek() == Some('"') {
                    self.advance();
                    let version = self.read_until('"');
                    if self.peek() == Some('"') {
                        self.advance();
                        Token::Version(version)
                    } else {
                        return Err(LangError::syntax_error_with_location(
                            "Unterminated version string",
                            start_line,
                            start_column,
                        ));
                    }
                } else {
                    // Rewind and treat as identifier
                    self.position = start_pos;
                    self.column = start_column;
                    let ident = self.read_identifier();
                    Token::Identifier(ident)
                }
            },
            '🎤' => {
                self.advance();
                Token::UserInput
            },
            'ℳ' => {
                self.advance();
                // Check if this is a procedural macro (ℳƒ)
                if self.peek() == Some('ƒ') {
                    self.advance();
                    Token::ProceduralMacroKeyword
                } else {
                    Token::MacroKeyword
                }
            },
            // Symbolic operators
            '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' | '&' | '|' => {
                self.advance();
                Token::SymbolicOperator(c)
            },
            // Symbolic keywords
            '⊤' | '⊥' | 'ι' | 'ƒ' | 'λ' | '⟼' | '⌽' | '⊲' | '⇉' | '⇓' | '⇑' | '⥮' | '□' | '⬚' | '✎' | '⌨' | '⟑' | '⊢' => {
                self.advance();
                match c {
                    '⊤' => Token::BooleanLiteral(true),
                    '⊥' => Token::BooleanLiteral(false),
                    _ => Token::SymbolicKeyword(c),
                }
            },
            _ => {
                return Err(LangError::syntax_error_with_location(
                    &format!("Unexpected character: {}", c),
                    self.line,
                    self.column,
                ));
            }
        };

        let end_pos = self.position;

        Ok(Some(TokenInfo {
            token,
            line: start_line,
            column: start_column,
            start_pos,
            end_pos,
        }))
    }

    /// Advance the position by one character.
    fn advance(&mut self) {
        if self.position < self.chars.len() {
            if self.chars[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    /// Peek at the next character without advancing.
    fn peek(&self) -> Option<char> {
        if self.position < self.chars.len() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }

    /// Skip whitespace characters.
    fn skip_whitespace(&mut self) {
        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' && self.position + 1 < self.chars.len() && self.chars[self.position + 1] == '/' {
                // Skip single-line comments
                while self.position < self.chars.len() && self.chars[self.position] != '\n' {
                    self.advance();
                }
            } else if c == '/' && self.position + 1 < self.chars.len() && self.chars[self.position + 1] == '*' {
                // Skip multi-line comments
                self.advance(); // Skip '/'
                self.advance(); // Skip '*'
                while self.position + 1 < self.chars.len() {
                    if self.chars[self.position] == '*' && self.chars[self.position + 1] == '/' {
                        self.advance(); // Skip '*'
                        self.advance(); // Skip '/'
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// Read a number from the input.
    fn read_number(&mut self) -> Result<i64, LangError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut num_str = String::new();

        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c.is_digit(10) {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        num_str.parse::<i64>().map_err(|_| {
            LangError::syntax_error_with_location(
                &format!("Invalid number: {}", num_str),
                start_line,
                start_column,
            )
        })
    }

    /// Read an identifier from the input.
    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();

        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        ident
    }

    /// Read a string from the input.
    fn read_string(&mut self) -> Result<String, LangError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut s = String::new();

        // Skip the opening quote
        self.advance();

        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c == '"' {
                // Skip the closing quote
                self.advance();
                return Ok(s);
            } else if c == '\\' {
                // Handle escape sequences
                self.advance();
                if self.position < self.chars.len() {
                    let escape_char = self.chars[self.position];
                    match escape_char {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        _ => {
                            return Err(LangError::syntax_error_with_location(
                                &format!("Invalid escape sequence: \\{}", escape_char),
                                self.line,
                                self.column,
                            ));
                        }
                    }
                    self.advance();
                } else {
                    return Err(LangError::syntax_error_with_location(
                        "Unterminated string",
                        start_line,
                        start_column,
                    ));
                }
            } else {
                s.push(c);
                self.advance();
            }
        }

        Err(LangError::syntax_error_with_location(
            "Unterminated string",
            start_line,
            start_column,
        ))
    }

    /// Read until a specific character is encountered.
    fn read_until(&mut self, end_char: char) -> String {
        let mut s = String::new();

        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c == end_char {
                break;
            } else {
                s.push(c);
                self.advance();
            }
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_number() {
        let mut lexer = Lexer::new("42".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Number(42));
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new("\"hello\"".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::StringLiteral("hello".to_string()));
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_identifier() {
        let mut lexer = Lexer::new("foo".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Identifier("foo".to_string()));
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_boolean() {
        let mut lexer = Lexer::new("⊤ ⊥".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::BooleanLiteral(true));
        assert_eq!(tokens[1].token, Token::BooleanLiteral(false));
        assert_eq!(tokens[2].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_symbolic_keywords() {
        let mut lexer = Lexer::new("ι ƒ λ ⟼ ⌽".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token, Token::SymbolicKeyword('ι'));
        assert_eq!(tokens[1].token, Token::SymbolicKeyword('ƒ'));
        assert_eq!(tokens[2].token, Token::SymbolicKeyword('λ'));
        assert_eq!(tokens[3].token, Token::SymbolicKeyword('⟼'));
        assert_eq!(tokens[4].token, Token::SymbolicKeyword('⌽'));
        assert_eq!(tokens[5].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_symbolic_operators() {
        let mut lexer = Lexer::new("+ - * / = ! < > & |".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[0].token, Token::SymbolicOperator('+'));
        assert_eq!(tokens[1].token, Token::SymbolicOperator('-'));
        assert_eq!(tokens[2].token, Token::SymbolicOperator('*'));
        assert_eq!(tokens[3].token, Token::SymbolicOperator('/'));
        assert_eq!(tokens[4].token, Token::SymbolicOperator('='));
        assert_eq!(tokens[5].token, Token::SymbolicOperator('!'));
        assert_eq!(tokens[6].token, Token::SymbolicOperator('<'));
        assert_eq!(tokens[7].token, Token::SymbolicOperator('>'));
        assert_eq!(tokens[8].token, Token::SymbolicOperator('&'));
        assert_eq!(tokens[9].token, Token::SymbolicOperator('|'));
        assert_eq!(tokens[10].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_string_dict_ref() {
        let mut lexer = Lexer::new(":hello".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::StringDictRef("hello".to_string()));
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_user_input() {
        let mut lexer = Lexer::new("🎤".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::UserInput);
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_version() {
        let mut lexer = Lexer::new("v\"1.0.0\"".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Version("1.0.0".to_string()));
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_attribute() {
        let mut lexer = Lexer::new("#[feature=\"web\"]".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Attribute("feature=\"web\"".to_string()));
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_module_import() {
        let mut lexer = Lexer::new("⟑ math::functions".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token, Token::SymbolicKeyword('⟑'));
        assert_eq!(tokens[1].token, Token::Identifier("math".to_string()));
        assert_eq!(tokens[2].token, Token::DoubleColon);
        assert_eq!(tokens[3].token, Token::Identifier("functions".to_string()));
        assert_eq!(tokens[4].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_module_alias() {
        let mut lexer = Lexer::new("⟑ math as m".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token, Token::SymbolicKeyword('⟑'));
        assert_eq!(tokens[1].token, Token::Identifier("math".to_string()));
        assert_eq!(tokens[2].token, Token::As);
        assert_eq!(tokens[3].token, Token::Identifier("m".to_string()));
        assert_eq!(tokens[4].token, Token::EOF);
    }

    // Tests for macro system tokens
    #[test]
    fn test_tokenize_macro_keyword() {
        let mut lexer = Lexer::new("ℳ".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::MacroKeyword);
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_procedural_macro_keyword() {
        let mut lexer = Lexer::new("ℳƒ".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::ProceduralMacroKeyword);
        assert_eq!(tokens[1].token, Token::EOF);
    }

    #[test]
    fn test_tokenize_macro_definition() {
        let mut lexer = Lexer::new("ℳ unless(condition, body) ⟼ if (!condition) { body }".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::MacroKeyword);
        assert_eq!(tokens[1].token, Token::Identifier("unless".to_string()));
        assert_eq!(tokens[2].token, Token::Parenthesis('('));
        assert_eq!(tokens[3].token, Token::Identifier("condition".to_string()));
        assert_eq!(tokens[4].token, Token::Comma);
        assert_eq!(tokens[5].token, Token::Identifier("body".to_string()));
        assert_eq!(tokens[6].token, Token::Parenthesis(')'));
        assert_eq!(tokens[7].token, Token::SymbolicKeyword('⟼'));
    }

    #[test]
    fn test_tokenize_procedural_macro_definition() {
        let mut lexer = Lexer::new("ℳƒ debug_print(expr) ⟼ { ⟼ expr }".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::ProceduralMacroKeyword);
        assert_eq!(tokens[1].token, Token::Identifier("debug_print".to_string()));
        assert_eq!(tokens[2].token, Token::Parenthesis('('));
        assert_eq!(tokens[3].token, Token::Identifier("expr".to_string()));
        assert_eq!(tokens[4].token, Token::Parenthesis(')'));
        assert_eq!(tokens[5].token, Token::SymbolicKeyword('⟼'));
        assert_eq!(tokens[6].token, Token::CurlyBrace('{'));
        assert_eq!(tokens[7].token, Token::SymbolicKeyword('⟼'));
        assert_eq!(tokens[8].token, Token::Identifier("expr".to_string()));
        assert_eq!(tokens[9].token, Token::CurlyBrace('}'));
    }
}
