// Execution engine module for Advanced REPL Service
//
// This module provides functionality for executing Anarchy Inference code
// within the REPL environment, with support for capturing output, timing,
// and error handling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout, ChildStderr};
use std::thread;

use crate::language_hub_server::repl::session::{Session, ExecutionHistoryEntry};
use crate::language_hub_server::repl::types::{ExecutionResult, ExecutionStatus, ErrorType, ErrorInfo, ErrorLocation};
use chrono::Utc;
use uuid::Uuid;
use serde_json::{json, Value};

/// Execution engine configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum execution time in milliseconds
    pub max_execution_time: u64,
    
    /// Maximum memory usage in megabytes
    pub max_memory_usage: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            max_execution_time: 5000, // 5 seconds
            max_memory_usage: 100, // 100 MB
        }
    }
}

/// Execution engine
pub struct ExecutionEngine {
    /// Execution configuration
    config: ExecutionConfig,
    
    /// Anarchy Inference interpreter process
    interpreter: Option<AnarchyInterpreter>,
    
    /// Active executions
    active_executions: HashMap<String, ExecutionInfo>,
}

/// Execution information
struct ExecutionInfo {
    /// Execution ID
    id: String,
    
    /// Session ID
    session_id: String,
    
    /// Code being executed
    code: String,
    
    /// Start time
    start_time: Instant,
    
    /// Timeout
    timeout: Duration,
    
    /// Whether to capture output
    capture_output: bool,
}

/// Anarchy Inference interpreter
struct AnarchyInterpreter {
    /// Child process
    process: Child,
    
    /// Standard input
    stdin: ChildStdin,
    
    /// Standard output
    stdout: ChildStdout,
    
    /// Standard error
    stderr: ChildStderr,
    
    /// Whether the interpreter is ready
    ready: bool,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(config: ExecutionConfig) -> Self {
        ExecutionEngine {
            config,
            interpreter: None,
            active_executions: HashMap::new(),
        }
    }
    
    /// Execute code in a session
    pub fn execute(
        &mut self,
        session: &mut Session,
        code: &str,
        timeout_ms: u64,
        capture_output: bool
    ) -> Result<ExecutionResult, String> {
        // Initialize the interpreter if needed
        if self.interpreter.is_none() {
            self.initialize_interpreter()?;
        }
        
        // Generate an execution ID
        let execution_id = Uuid::new_v4().to_string();
        
        // Create execution info
        let execution_info = ExecutionInfo {
            id: execution_id.clone(),
            session_id: session.id.clone(),
            code: code.to_string(),
            start_time: Instant::now(),
            timeout: Duration::from_millis(timeout_ms),
            capture_output,
        };
        
        // Add to active executions
        self.active_executions.insert(execution_id.clone(), execution_info);
        
        // Prepare the execution context
        let context = self.prepare_execution_context(session)?;
        
        // Execute the code
        let result = self.execute_code(code, context, timeout_ms, capture_output);
        
        // Remove from active executions
        self.active_executions.remove(&execution_id);
        
        // Process the result
        let execution_result = match result {
            Ok(result) => {
                // Update session variables with any new variables from the execution
                if let Some(variables) = result.get("variables").and_then(|v| v.as_object()) {
                    for (name, value) in variables {
                        session.variables.insert(name.clone(), value.clone());
                    }
                }
                
                // Create the execution result
                let execution_result = ExecutionResult {
                    result: result.get("result").cloned().unwrap_or(json!(null)),
                    output: result.get("output").and_then(|o| o.as_str()).map(|s| s.to_string()),
                    duration: result.get("duration").and_then(|d| d.as_u64()).unwrap_or(0),
                    status: result.get("status").and_then(|s| s.as_str()).unwrap_or("success").to_string(),
                };
                
                // Add to execution history
                let history_entry = ExecutionHistoryEntry {
                    id: execution_id,
                    code: code.to_string(),
                    result: Some(execution_result.result.clone()),
                    output: execution_result.output.clone(),
                    duration: execution_result.duration,
                    status: execution_result.status.clone(),
                    timestamp: Utc::now(),
                };
                
                session.history.push(history_entry);
                
                // Limit history size
                if session.history.len() > 100 {
                    session.history.remove(0);
                }
                
                execution_result
            }
            Err(e) => {
                // Create an error result
                let execution_result = ExecutionResult {
                    result: json!({
                        "error": {
                            "message": e,
                            "type": "runtime"
                        }
                    }),
                    output: None,
                    duration: execution_info.start_time.elapsed().as_millis() as u64,
                    status: "error".to_string(),
                };
                
                // Add to execution history
                let history_entry = ExecutionHistoryEntry {
                    id: execution_id,
                    code: code.to_string(),
                    result: Some(execution_result.result.clone()),
                    output: None,
                    duration: execution_result.duration,
                    status: execution_result.status.clone(),
                    timestamp: Utc::now(),
                };
                
                session.history.push(history_entry);
                
                // Limit history size
                if session.history.len() > 100 {
                    session.history.remove(0);
                }
                
                execution_result
            }
        };
        
        Ok(execution_result)
    }
    
    /// Initialize the Anarchy Inference interpreter
    fn initialize_interpreter(&mut self) -> Result<(), String> {
        // Start the Anarchy Inference interpreter process
        let mut process = match Command::new("python3")
            .arg("-c")
            .arg(r#"
import sys
import json
import time
import traceback

# This is a simplified interpreter for demonstration purposes
# In a real implementation, this would use the actual Anarchy Inference interpreter

class AnarchyInterpreter:
    def __init__(self):
        self.variables = {}
        self.ready = True
        print("READY", flush=True)
    
    def execute(self, code, context=None):
        start_time = time.time()
        output = ""
        result = None
        status = "success"
        error = None
        
        if context:
            # Load variables from context
            self.variables.update(context.get("variables", {}))
        
        try:
            # Capture stdout
            original_stdout = sys.stdout
            sys.stdout = OutputCapture()
            
            # Execute the code
            # In a real implementation, this would use the Anarchy Inference parser and interpreter
            # For demonstration, we'll use Python's exec() with some basic parsing
            
            # Check for variable assignment
            if "=" in code and not "==" in code:
                parts = code.split("=", 1)
                var_name = parts[0].strip()
                var_expr = parts[1].strip()
                
                # Execute the right side
                var_value = eval(var_expr, {"__builtins__": __builtins__}, self.variables)
                
                # Store the variable
                self.variables[var_name] = var_value
                result = var_value
            else:
                # Execute as expression or statement
                try:
                    # Try as expression
                    result = eval(code, {"__builtins__": __builtins__}, self.variables)
                except SyntaxError:
                    # Try as statement
                    exec(code, {"__builtins__": __builtins__}, self.variables)
                    result = None
            
            # Get captured output
            output = sys.stdout.getvalue()
            
            # Restore stdout
            sys.stdout = original_stdout
            
        except Exception as e:
            # Restore stdout
            sys.stdout = original_stdout
            
            # Get error details
            error_type = type(e).__name__
            error_message = str(e)
            
            tb = traceback.extract_tb(sys.exc_info()[2])
            error_line = tb[-1].lineno if tb else 0
            error_column = 0  # Not available in Python's traceback
            
            error = {
                "type": error_type,
                "message": error_message,
                "location": {
                    "line": error_line,
                    "column": error_column
                }
            }
            
            status = "error"
        
        duration = int((time.time() - start_time) * 1000)  # Convert to milliseconds
        
        return {
            "result": result,
            "output": output,
            "duration": duration,
            "status": status,
            "error": error,
            "variables": self.variables
        }

class OutputCapture:
    def __init__(self):
        self.value = ""
    
    def write(self, text):
        self.value += text
    
    def flush(self):
        pass
    
    def getvalue(self):
        return self.value

# Create the interpreter
interpreter = AnarchyInterpreter()

# Main loop
while True:
    try:
        # Read a command
        command_line = input()
        command = json.loads(command_line)
        
        if command["type"] == "execute":
            # Execute code
            code = command["code"]
            context = command.get("context")
            
            result = interpreter.execute(code, context)
            
            # Convert result to JSON-serializable format
            if result["result"] is not None:
                try:
                    # Try to convert to a simple type
                    json.dumps(result["result"])
                except:
                    # If not serializable, convert to string
                    result["result"] = str(result["result"])
            
            # Convert variables to JSON-serializable format
            for var_name, var_value in list(result["variables"].items()):
                try:
                    # Try to convert to a simple type
                    json.dumps(var_value)
                except:
                    # If not serializable, convert to string
                    result["variables"][var_name] = str(var_value)
            
            # Send the result
            print(json.dumps(result), flush=True)
        
        elif command["type"] == "exit":
            # Exit the interpreter
            break
        
        else:
            # Unknown command
            print(json.dumps({
                "error": {
                    "message": f"Unknown command: {command['type']}",
                    "type": "command"
                }
            }), flush=True)
    
    except Exception as e:
        # Send error
        print(json.dumps({
            "error": {
                "message": str(e),
                "type": "internal"
            }
        }), flush=True)

# Exit
sys.exit(0)
            "#)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() {
            Ok(process) => process,
            Err(e) => return Err(format!("Failed to start Anarchy Inference interpreter: {}", e)),
        };
        
        // Get stdin, stdout, and stderr
        let stdin = process.stdin.take().ok_or("Failed to open stdin")?;
        let mut stdout = process.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = process.stderr.take().ok_or("Failed to open stderr")?;
        
        // Wait for the interpreter to be ready
        let mut ready_line = String::new();
        match stdout.read_to_string(&mut ready_line) {
            Ok(_) => {
                if !ready_line.trim().ends_with("READY") {
                    return Err(format!("Interpreter failed to initialize: {}", ready_line));
                }
            }
            Err(e) => return Err(format!("Failed to read from interpreter: {}", e)),
        }
        
        // Create the interpreter
        self.interpreter = Some(AnarchyInterpreter {
            process,
            stdin,
            stdout,
            stderr,
            ready: true,
        });
        
        Ok(())
    }
    
    /// Prepare the execution context
    fn prepare_execution_context(&self, session: &Session) -> Result<Value, String> {
        // Create the context
        let context = json!({
            "sessionId": session.id,
            "variables": session.variables,
        });
        
        Ok(context)
    }
    
    /// Execute code
    fn execute_code(
        &mut self,
        code: &str,
        context: Value,
        timeout_ms: u64,
        capture_output: bool
    ) -> Result<Value, String> {
        // Get the interpreter
        let interpreter = match &mut self.interpreter {
            Some(interpreter) => interpreter,
            None => return Err("Interpreter not initialized".to_string()),
        };
        
        // Create the command
        let command = json!({
            "type": "execute",
            "code": code,
            "context": context,
            "captureOutput": capture_output,
        });
        
        // Send the command
        let command_str = match serde_json::to_string(&command) {
            Ok(str) => str,
            Err(e) => return Err(format!("Failed to serialize command: {}", e)),
        };
        
        if let Err(e) = writeln!(interpreter.stdin, "{}", command_str) {
            return Err(format!("Failed to write to interpreter: {}", e));
        }
        
        // Read the result with timeout
        let timeout = Duration::from_millis(timeout_ms);
        let start_time = Instant::now();
        
        let mut result_line = String::new();
        let mut buffer = [0; 1024];
        
        while start_time.elapsed() < timeout {
            // Check if there's data available
            match interpreter.stdout.read(&mut buffer) {
                Ok(0) => break, // End of file
                Ok(n) => {
                    result_line.push_str(&String::from_utf8_lossy(&buffer[0..n]));
                    if result_line.ends_with("\n") {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available yet, sleep for a bit
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(format!("Failed to read from interpreter: {}", e)),
            }
        }
        
        // Check for timeout
        if start_time.elapsed() >= timeout {
            return Err("Execution timed out".to_string());
        }
        
        // Parse the result
        match serde_json::from_str::<Value>(&result_line) {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Failed to parse result: {}", e)),
        }
    }
    
    /// Cancel an execution
    pub fn cancel_execution(&mut self, execution_id: &str) -> Result<(), String> {
        // Check if the execution exists
        if !self.active_executions.contains_key(execution_id) {
            return Err(format!("Execution not found: {}", execution_id));
        }
        
        // For now, we don't have a way to cancel an execution in progress
        // In a real implementation, we would need to send a signal to the interpreter
        // or restart it if necessary
        
        // Remove from active executions
        self.active_executions.remove(execution_id);
        
        Ok(())
    }
    
    /// Get active executions
    pub fn get_active_executions(&self) -> Vec<String> {
        self.active_executions.keys().cloned().collect()
    }
    
    /// Get execution information
    pub fn get_execution_info(&self, execution_id: &str) -> Option<ExecutionInfo> {
        self.active_executions.get(execution_id).cloned()
    }
    
    /// Shutdown the execution engine
    pub fn shutdown(&mut self) -> Result<(), String> {
        // Cancel all active executions
        for execution_id in self.get_active_executions() {
            let _ = self.cancel_execution(&execution_id);
        }
        
        // Shutdown the interpreter
        if let Some(interpreter) = &mut self.interpreter {
            // Send exit command
            let command = json!({
                "type": "exit",
            });
            
            let command_str = match serde_json::to_string(&command) {
                Ok(str) => str,
                Err(e) => return Err(format!("Failed to serialize exit command: {}", e)),
            };
            
            if let Err(e) = writeln!(interpreter.stdin, "{}", command_str) {
                eprintln!("Warning: Failed to send exit command: {}", e);
            }
            
            // Wait for the process to exit
            match interpreter.process.wait() {
                Ok(_) => {}
                Err(e) => eprintln!("Warning: Failed to wait for interpreter process: {}", e),
            }
        }
        
        // Clear the interpreter
        self.interpreter = None;
        
        Ok(())
    }
}

impl Clone for ExecutionInfo {
    fn clone(&self) -> Self {
        ExecutionInfo {
            id: self.id.clone(),
            session_id: self.session_id.clone(),
            code: self.code.clone(),
            start_time: self.start_time,
            timeout: self.timeout,
            capture_output: self.capture_output,
        }
    }
}
