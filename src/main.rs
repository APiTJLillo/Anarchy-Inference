use crate::error::LangError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use std::fs;
use log::debug;
use anarchy_inference::ui::YewApp;

mod ast;
mod lexer;
mod parser;
mod interpreter;
mod error;
mod network;
mod concurrency;
mod lsp;
mod ui;
mod semantic;

fn main() -> Result<(), LangError> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1])?;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    
    debug!("Token stream: {:?}", tokens);
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program()?;
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast)?;

    yew::Renderer::<YewApp>::new().render();
    Ok(())
}
