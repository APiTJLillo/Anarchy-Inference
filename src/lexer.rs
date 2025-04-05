// Lexer for the minimal LLM-friendly language

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
    StringDictRef(String),  // New token type for string dictionary references
    Parenthesis(char),
    CurlyBrace(char),
    Comma,
    Semicolon,
    Dot,
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
            Token::StringDictRef(key) => write!(f, ":{}", key),  // Format string dictionary reference
            Token::Parenthesis(c) => write!(f, "{}", c),
            Token::CurlyBrace(c) => write!(f, "{}", c),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Dot => write!(f, "."),
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
                // If next_token returned None, we reached EOI. Push EOF if not present.
                if !tokens.iter().any(|t| t.token == Token::EOF) {
                    tokens.push(TokenInfo {
                        token: Token::EOF,
                        line: self.line,
                        column: self.column,
                        start_pos: self.position,
                        end_pos: self.position,
                    });
                }
                break;
            }
        }

        Ok(tokens)
    }

    /// Reads the next token or returns None if at end of input.
    fn next_token(&mut self) -> Result<Option<TokenInfo>, LangError> {
        self.skip_whitespace();

        // End of input?
        if self.position >= self.chars.len() {
            return Ok(Some(TokenInfo {
                token: Token::EOF,
                line: self.line,
                column: self.column,
                start_pos: self.position,
                end_pos: self.position,
            }));
        }

        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.position;

        let c = match self.current_char() {
            Some(ch) => ch,
            None => return Ok(None), // Shouldn't happen if we check length above
        };

        // Match on first character to decide which scanning function to use
        let token = match c {
            '/' => {
                if self.peek_next() == Some('/') {
                    // Skip until end of line
                    while let Some(ch) = self.current_char() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    return self.next_token();
                } else {
                    self.advance();
                    Token::SymbolicOperator(c)
                }
            },
            ':' => {
                // Handle string dictionary reference
                self.advance(); // Skip the colon
                self.read_string_dict_ref()?
            },
            '0'..='9' => self.read_number()?,
            '"' => self.read_string()?,
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier()?,
            '(' | ')' => {
                self.advance();
                Token::Parenthesis(c)
            }
            '{' | '}' => {
                self.advance();
                Token::CurlyBrace(c)
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            ';' => {
                self.advance();
                Token::Semicolon
            }
            '.' => {
                self.advance();
                Token::Dot
            }
            '⊤' => {
                self.advance();
                Token::BooleanLiteral(true)
            }
            '⊥' => {
                self.advance();
                Token::BooleanLiteral(false)
            }
            // Symbolic keywords
            '⬢' | '□' | '⬚' | '⚈' | '⟳'
            | 'λ' | 'ƒ' | 'ι' | '⟼' | '⌽' | '∞' | 'ξ' | 'σ' | '∇' | '⟑'
            | 'α' | 'β' | 'γ' | 'δ' | 'ε' | 'ζ' | 'η' | 'θ' | 'κ' | 'ν'
            | 'ο' | 'π' | 'ρ' | 'τ' | 'υ' | 'φ' | 'χ' | 'ψ' | 'ω'
            | '∅' | '＋' | '∑' | '∀' | '⊳' | '⊢' | '⊣'
            | '÷' | '⚠'
            | '↯' | '↱' | '⌸'
            | '#' | '🔒' | '🔑' | '⚿'
            | '⏰'
            | '🔢' | '🔤'
            | '⇪'
            | '⚡' | '⊲' | '⇉' | '⇓' | '⇑' | '⥮'
            | '⟿' | '⇢' | '⇠' | '⟰' | '⇡' | '⇣'
            | '✎' | '⌨'
            // File System Operations Emoji
            | '📂' | '📖' | '✍' | '✂' | '⧉' | '↷' | '?'
            // Shell & OS Process Control Emoji
            | '🖥' | '🌐'
            // String Dictionary Operations Emoji
            | '📝' | '🔠' | '💾' | '🔄'
            // Agent Memory Emoji
            | '🗑'
            // Browser Automation Emoji
            | '🖱' | '👁' | '🧠' | '❌'
            => {
                self.advance();
                Token::SymbolicKeyword(c)
            }
            // Symbolic operators
            '+' | '-' | '*' | '=' | '!' | '>' | '<' | '&'
            | '|' | '^' | '%' | '~' | '≥' | '≤' | '≠' | '≈' => {
                self.advance();
                Token::SymbolicOperator(c)
            }
            _ if c.is_whitespace() => {
                // We skip these in skip_whitespace(), so we usually won't get here.
                self.advance();
                return self.next_token();
            }
            _ => return Err(LangError::syntax_error(&format!("Unexpected character: {}", c))),
        };

        Ok(Some(TokenInfo {
            token,
            line: start_line,
            column: start_column,
            start_pos,
            end_pos: self.position,
        }))
    }

    fn current_char(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Reads consecutive digits as a Number token.
    fn read_number(&mut self) -> Result<Token, LangError> {
        let mut number = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        match number.parse::<i64>() {
            Ok(n) => Ok(Token::Number(n)),
            Err(e) => Err(LangError::syntax_error(&format!("Invalid number literal: {}", e))),
        }
    }

    /// Reads a quoted string `"..."`. Handles basic escapes like `\"` or `\\`.
    fn read_string(&mut self) -> Result<Token, LangError> {
        // Skip the opening quote
        self.advance();
        let mut result = String::new();

        while let Some(ch) = self.current_char() {
            match ch {
                '"' => {
                    self.advance(); // Skip closing quote
                    return Ok(Token::StringLiteral(result));
                }
                '\\' => {
                    self.advance();
                    if let Some(next) = self.current_char() {
                        match next {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            'r' => result.push('\r'),
                            '\\' => result.push('\\'),
                            '"' => result.push('"'),
                            _ => return Err(LangError::syntax_error(&format!("Invalid escape sequence: \\{}", next))),
                        }
                        self.advance();
                    } else {
                        return Err(LangError::syntax_error("Unterminated escape sequence"));
                    }
                }
                _ => {
                    result.push(ch);
                    self.advance();
                }
            }
        }
        Err(LangError::syntax_error("Unterminated string literal"))
    }

    /// Reads an identifier (letter or underscore followed by letters, digits, or underscores).
    fn read_identifier(&mut self) -> Result<Token, LangError> {
        let mut identifier = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Ok(Token::Identifier(identifier))
    }

    /// Reads a string dictionary reference key (after the colon).
    fn read_string_dict_ref(&mut self) -> Result<Token, LangError> {
        let mut key = String::new();
        
        // First character must be a letter or underscore
        if let Some(ch) = self.current_char() {
            if ch.is_alphabetic() || ch == '_' {
                key.push(ch);
                self.advance();
            } else {
                return Err(LangError::syntax_error(&format!(
                    "Invalid string dictionary key: must start with a letter or underscore, found '{}'", 
                    ch
                )));
            }
        } else {
            return Err(LangError::syntax_error("Unexpected end of input after ':'"));
        }
        
        // Rest of the key can include letters, digits, or underscores
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                key.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(Token::StringDictRef(key))
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.position + 1).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_number() {
        let mut lexer = Lexer::new("123".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // 123 + EOF
        assert_eq!(tokens[0].token, Token::Number(123));
        assert_eq!(tokens[0].start_pos, 0);
        assert_eq!(tokens[0].end_pos, 3);
        assert!(matches!(tokens[1].token, Token::EOF));
    }

    #[test]
    fn test_multi_byte_symbols() {
        let mut lexer = Lexer::new("λ ⬢".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::SymbolicKeyword('λ'));
        assert_eq!(tokens[1].token, Token::SymbolicKeyword('⬢'));
        assert!(matches!(tokens[2].token, Token::EOF));
    }
    
    #[test]
    fn test_string_dict_ref() {
        let mut lexer = Lexer::new(":hello :world123 :_test".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::StringDictRef("hello".to_string()));
        assert_eq!(tokens[1].token, Token::StringDictRef("world123".to_string()));
        assert_eq!(tokens[2].token, Token::StringDictRef("_test".to_string()));
        assert!(matches!(tokens[3].token, Token::EOF));
    }
}
