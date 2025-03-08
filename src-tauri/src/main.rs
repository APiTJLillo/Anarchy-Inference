#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use std::sync::Mutex;
use anarchy_inference::interpreter::Interpreter;
use anarchy_inference::parser::Parser;
use anarchy_inference::lexer::Lexer;
use anarchy_inference::error::LangError;

// State to hold our interpreter
struct InterpreterState(Mutex<Interpreter>);

#[tauri::command]
async fn execute_code(
    code: String,
    state: tauri::State<'_, InterpreterState>
) -> Result<String, String> {
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.lex();
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            let mut interpreter = state.0.lock().unwrap();
            match interpreter.eval(&ast) {
                Ok(result) => Ok(result.to_string()),
                Err(e) => Err(format!("Runtime error: {}", e))
            }
        }
        Err(e) => Err(format!("Parse error: {}", e))
    }
}

#[tauri::command]
async fn run_shell_command(command: String) -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(path, contents)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_window(
    window: tauri::Window,
    title: String,
    width: f64,
    height: f64
) -> Result<(), String> {
    tauri::WindowBuilder::new(
        &window.app_handle(),
        title.clone(), /* the window label */
        tauri::WindowUrl::App("index.html".into())
    )
    .title(title)
    .inner_size(width, height)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .manage(InterpreterState(Mutex::new(Interpreter::new())))
        .invoke_handler(tauri::generate_handler![
            execute_code,
            run_shell_command,
            read_file,
            write_file,
            create_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 