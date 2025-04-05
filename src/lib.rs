// src/lib.rs - Modified to include string dictionary support
// This file is the main entry point for the library

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod value;
pub mod core;
pub mod gc;
pub mod std_lib;
pub mod concurrency;
pub mod network;
pub mod security;
pub mod semantic;
pub mod lsp;
pub mod ui;

// Re-export commonly used types
pub use ast::{ASTNode, NodeType};
pub use error::LangError;
pub use lexer::{Lexer, Token, TokenInfo};
pub use parser::Parser;
pub use interpreter::Interpreter;
pub use value::Value;
pub use core::string_dict::{StringDictionary, StringDictionaryManager};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the language runtime
pub fn init() -> Interpreter {
    let mut interpreter = Interpreter::new();
    
    // Initialize standard library
    std_lib::init(&mut interpreter);
    
    // Initialize default string dictionary
    let dict_manager = interpreter.get_string_dict_manager_mut();
    dict_manager.set_string("hello".to_string(), "Hello, world!".to_string());
    dict_manager.set_string("error".to_string(), "Error: {}".to_string());
    dict_manager.set_string("success".to_string(), "Operation completed successfully: {}".to_string());
    
    interpreter
}

/// Parse and execute a program
pub fn run(source: &str) -> Result<Value, LangError> {
    let lexer = Lexer::new(source.to_string());
    let mut parser = Parser::from_lexer(lexer)?;
    let nodes = parser.parse()?;
    
    let mut interpreter = init();
    interpreter.execute_nodes(&nodes)
}

/// Parse a program
pub fn parse(source: &str) -> Result<Vec<ASTNode>, LangError> {
    let lexer = Lexer::new(source.to_string());
    let mut parser = Parser::from_lexer(lexer)?;
    parser.parse()
}

/// Load and execute a program from a file
pub fn run_file(path: &str) -> Result<Value, LangError> {
    use std::fs;
    let source = fs::read_to_string(path)
        .map_err(|e| LangError::io_error(&format!("Failed to read file: {}", e)))?;
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::from_lexer(lexer)?;
    let nodes = parser.parse()?;
    
    let mut interpreter = init();
    interpreter.set_current_file(path.to_string());
    interpreter.execute_nodes(&nodes)
}

/// Load a string dictionary from a file
pub fn load_string_dictionary(interpreter: &mut Interpreter, path: &str) -> Result<(), LangError> {
    interpreter.load_string_dictionary(path)
}
