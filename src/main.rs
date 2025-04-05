#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
use crate::error::LangError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use std::fs;
use log::debug;

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
mod value;

// Helper function to run code
fn run_code(input: &str, interpreter: &mut Interpreter) -> Result<String, LangError> {
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize()?;
    
    debug!("Token stream: {:?}", tokens);
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program()?;
    
    // Execute each node in the AST
    let mut result = String::new();
    for node in &ast {
        let value = interpreter.execute(node)?;
        result = format!("{}", value);
    }
    
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), LangError> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    
    // Handle REPL mode
    if args.len() == 2 && args[1] == "repl" {
        println!("Anarchy-Inference REPL Mode");
        println!("Type 'exit' to quit");
        
        let mut interpreter = Interpreter::new();
        
        loop {
            use std::io::{self, Write};
            
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            let input = input.trim();
            if input == "exit" {
                break;
            }
            
            match run_code(input, &mut interpreter) {
                Ok(result) => println!("{}", result),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        
        return Ok(());
    }
    
    // Normal file execution mode
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file> or {} repl", args[0], args[0]);
        std::process::exit(1);
    }
    
    let input = fs::read_to_string(&args[1])?;
    let mut interpreter = Interpreter::new();
    
    match run_code(&input, &mut interpreter) {
        Ok(_) => {},
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Only initialize Yew app when targeting wasm32
    #[cfg(target_arch = "wasm32")]
    {
        use crate::ui::App;
        yew::Renderer::<App>::new().render();
    }
    
    Ok(())
}
