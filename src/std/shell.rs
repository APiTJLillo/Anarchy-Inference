// src/std/shell.rs
// Shell and OS Process Control for Anarchy-Inference

use std::process::{Command, Output};
use std::env;
use crate::value::Value;
use crate::error::LangError;

/// Execute shell command
/// Symbol: !
/// Usage: !("ls -la") → {o:stdout, e:stderr, c:code}
pub fn execute_shell(command: &str) -> Result<Value, LangError> {
    // Split the command into program and arguments
    let mut parts = command.split_whitespace();
    let program = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    let output = match Command::new(program).args(args).output() {
        Ok(output) => output,
        Err(e) => return Err(LangError::runtime_error(&format!("Failed to execute command '{}': {}", command, e))),
    };

    // Create an object with stdout, stderr, and exit code
    let mut result = Value::empty_object();
    
    // Add stdout
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    result.set_property("o".to_string(), Value::string(stdout))?;
    
    // Add stderr
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    result.set_property("e".to_string(), Value::string(stderr))?;
    
    // Add exit code
    let code = output.status.code().unwrap_or(-1) as f64;
    result.set_property("c".to_string(), Value::number(code))?;

    Ok(result)
}

/// Get current OS
/// Symbol: 🖥 or s
/// Usage: s() → "linux"
pub fn current_os() -> Result<Value, LangError> {
    let os = env::consts::OS;
    Ok(Value::string(os.to_string()))
}

/// Get environment variable
/// Symbol: 🌐 or v
/// Usage: v("VAR_NAME") → "value"
pub fn get_env_var(name: &str) -> Result<Value, LangError> {
    match env::var(name) {
        Ok(value) => Ok(Value::string(value)),
        Err(_) => Ok(Value::null()), // Return null if the variable doesn't exist
    }
}

/// Register all shell functions
pub fn register_shell_functions() {
    // This function will be called from the main module to register all shell functions
    // Implementation will be added when the token registration system is implemented
    // Example:
    // reg("!", execute_shell);
    // reg("🖥", current_os);
    // reg("s", current_os);
    // reg("🌐", get_env_var);
    // reg("v", get_env_var);
}
