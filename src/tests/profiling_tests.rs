// src/tests/profiling_tests.rs - Tests for the Performance Profiling system

use std::time::Duration;

use crate::ast::{ASTNode, NodeType};
use crate::interpreter::Interpreter;
use crate::profiling::{Profiler, ProfilerConfig, SpanType, ReportFormat};
use crate::profiling::integration::ProfilingInterpreter;
use crate::value::Value;

/// Test basic profiler functionality
#[test]
fn test_basic_profiler() {
    // Create a profiler
    let mut profiler = Profiler::new();
    
    // Enable profiling
    profiler.set_enabled(true);
    
    // Start a session
    assert!(profiler.start_session("test_session").is_ok());
    
    // Start a span
    let span_guard = profiler.start_span("test_span", SpanType::Function).unwrap();
    
    // Sleep for a bit to simulate work
    std::thread::sleep(Duration::from_millis(10));
    
    // End the span (automatically done by dropping the guard)
    drop(span_guard);
    
    // End the session
    let session = profiler.end_session().unwrap();
    
    // Generate a report
    let report = profiler.generate_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains our span
    assert!(report.contains("test_span"));
}

/// Test profiling with the interpreter
#[test]
fn test_interpreter_profiling() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a simple AST for testing
    let ast = ASTNode {
        node_type: NodeType::Number(42.0),
        line: 1,
        column: 1,
    };
    
    // Execute the AST with profiling
    let result = interpreter.profile_execute_node(&ast).unwrap();
    
    // Check the result
    assert_eq!(result, Value::Number(42.0));
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains our operation
    assert!(report.contains("Number"));
}

/// Test profiling with multiple operations
#[test]
fn test_multiple_operations() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a more complex AST for testing
    let ast = ASTNode {
        node_type: NodeType::BinaryOp {
            op: "+".to_string(),
            left: Box::new(ASTNode {
                node_type: NodeType::Number(10.0),
                line: 1,
                column: 1,
            }),
            right: Box::new(ASTNode {
                node_type: NodeType::Number(20.0),
                line: 1,
                column: 5,
            }),
        },
        line: 1,
        column: 3,
    };
    
    // Execute the AST with profiling
    let result = interpreter.profile_execute_node(&ast).unwrap();
    
    // Check the result
    assert_eq!(result, Value::Number(30.0));
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains our operations
    assert!(report.contains("+"));
    assert!(report.contains("Arithmetic"));
}

/// Test profiling with function calls
#[test]
fn test_function_calls() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a function declaration
    let function_decl = ASTNode {
        node_type: NodeType::FunctionDeclaration {
            name: "test_function".to_string(),
            parameters: vec!["x".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::BinaryOp {
                    op: "*".to_string(),
                    left: Box::new(ASTNode {
                        node_type: NodeType::Variable("x".to_string()),
                        line: 2,
                        column: 5,
                    }),
                    right: Box::new(ASTNode {
                        node_type: NodeType::Number(2.0),
                        line: 2,
                        column: 9,
                    }),
                },
                line: 2,
                column: 7,
            }),
        },
        line: 1,
        column: 1,
    };
    
    // Execute the function declaration
    interpreter.profile_execute_node(&function_decl).unwrap();
    
    // Create a function call
    let function_call = ASTNode {
        node_type: NodeType::FunctionCall {
            function: Box::new(ASTNode {
                node_type: NodeType::Variable("test_function".to_string()),
                line: 3,
                column: 1,
            }),
            arguments: vec![ASTNode {
                node_type: NodeType::Number(5.0),
                line: 3,
                column: 15,
            }],
        },
        line: 3,
        column: 1,
    };
    
    // Execute the function call
    let result = interpreter.profile_execute_node(&function_call).unwrap();
    
    // Check the result
    assert_eq!(result, Value::Number(10.0));
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains our function
    assert!(report.contains("test_function"));
    assert!(report.contains("Function"));
}

/// Test profiling with blocks of code
#[test]
fn test_code_blocks() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a block of code
    let block = ASTNode {
        node_type: NodeType::Block(vec![
            ASTNode {
                node_type: NodeType::VariableDeclaration {
                    name: "x".to_string(),
                    value: Box::new(ASTNode {
                        node_type: NodeType::Number(10.0),
                        line: 1,
                        column: 9,
                    }),
                },
                line: 1,
                column: 1,
            },
            ASTNode {
                node_type: NodeType::VariableDeclaration {
                    name: "y".to_string(),
                    value: Box::new(ASTNode {
                        node_type: NodeType::Number(20.0),
                        line: 2,
                        column: 9,
                    }),
                },
                line: 2,
                column: 1,
            },
            ASTNode {
                node_type: NodeType::BinaryOp {
                    op: "+".to_string(),
                    left: Box::new(ASTNode {
                        node_type: NodeType::Variable("x".to_string()),
                        line: 3,
                        column: 1,
                    }),
                    right: Box::new(ASTNode {
                        node_type: NodeType::Variable("y".to_string()),
                        line: 3,
                        column: 5,
                    }),
                },
                line: 3,
                column: 3,
            },
        ]),
        line: 1,
        column: 1,
    };
    
    // Execute the block
    let result = interpreter.profile_execute_node(&block).unwrap();
    
    // Check the result
    assert_eq!(result, Value::Number(30.0));
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains our block
    assert!(report.contains("Block"));
    assert!(report.contains("Variable"));
}

/// Test profiling with memory operations
#[test]
fn test_memory_profiling() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling with memory tracking
    let mut config = ProfilerConfig::default();
    config.enabled = true;
    config.memory_profiling.enabled = true;
    config.memory_profiling.track_allocations = true;
    config.memory_profiling.track_peak_memory = true;
    
    let mut profiler = Profiler::with_config(config);
    interpreter.set_profiler(profiler);
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create objects to allocate memory
    let object_literal = ASTNode {
        node_type: NodeType::ObjectLiteral(vec![
            ("a".to_string(), ASTNode {
                node_type: NodeType::Number(1.0),
                line: 1,
                column: 5,
            }),
            ("b".to_string(), ASTNode {
                node_type: NodeType::Number(2.0),
                line: 1,
                column: 10,
            }),
            ("c".to_string(), ASTNode {
                node_type: NodeType::Number(3.0),
                line: 1,
                column: 15,
            }),
        ]),
        line: 1,
        column: 1,
    };
    
    // Execute the object creation
    interpreter.profile_execute_node(&object_literal).unwrap();
    
    // Create an array to allocate more memory
    let array_literal = ASTNode {
        node_type: NodeType::ArrayLiteral(vec![
            ASTNode {
                node_type: NodeType::Number(1.0),
                line: 2,
                column: 2,
            },
            ASTNode {
                node_type: NodeType::Number(2.0),
                line: 2,
                column: 5,
            },
            ASTNode {
                node_type: NodeType::Number(3.0),
                line: 2,
                column: 8,
            },
            ASTNode {
                node_type: NodeType::Number(4.0),
                line: 2,
                column: 11,
            },
            ASTNode {
                node_type: NodeType::Number(5.0),
                line: 2,
                column: 14,
            },
        ]),
        line: 2,
        column: 1,
    };
    
    // Execute the array creation
    interpreter.profile_execute_node(&array_literal).unwrap();
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains memory metrics
    assert!(report.contains("Memory Usage"));
    // Memory metrics might vary, so we just check for the section headers
}

/// Test JSON report generation
#[test]
fn test_json_report() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a simple AST for testing
    let ast = ASTNode {
        node_type: NodeType::Number(42.0),
        line: 1,
        column: 1,
    };
    
    // Execute the AST with profiling
    interpreter.profile_execute_node(&ast).unwrap();
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a JSON report
    let report = interpreter.generate_profiling_report(ReportFormat::Json).unwrap();
    
    // Check that the report is valid JSON
    assert!(report.starts_with("{"));
    assert!(report.ends_with("}"));
    assert!(report.contains("\"session\""));
    assert!(report.contains("\"name\": \"test_session\""));
}

/// Test profiling with string operations
#[test]
fn test_string_operations() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a string operation
    let string_op = ASTNode {
        node_type: NodeType::BinaryOp {
            op: "+".to_string(),
            left: Box::new(ASTNode {
                node_type: NodeType::String("Hello, ".to_string()),
                line: 1,
                column: 1,
            }),
            right: Box::new(ASTNode {
                node_type: NodeType::String("World!".to_string()),
                line: 1,
                column: 12,
            }),
        },
        line: 1,
        column: 10,
    };
    
    // Execute the string operation
    let result = interpreter.profile_execute_node(&string_op).unwrap();
    
    // Check the result
    assert_eq!(result, Value::String("Hello, World!".to_string()));
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains string operations
    assert!(report.contains("String"));
}

/// Test profiling with conditional operations
#[test]
fn test_conditional_operations() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Enable profiling
    interpreter.enable_profiling();
    
    // Start a profiling session
    assert!(interpreter.start_profiling_session("test_session").is_ok());
    
    // Create a conditional operation
    let conditional = ASTNode {
        node_type: NodeType::If {
            condition: Box::new(ASTNode {
                node_type: NodeType::Boolean(true),
                line: 1,
                column: 4,
            }),
            then_branch: Box::new(ASTNode {
                node_type: NodeType::Number(1.0),
                line: 1,
                column: 10,
            }),
            else_branch: Some(Box::new(ASTNode {
                node_type: NodeType::Number(2.0),
                line: 1,
                column: 18,
            })),
        },
        line: 1,
        column: 1,
    };
    
    // Execute the conditional
    let result = interpreter.profile_execute_node(&conditional).unwrap();
    
    // Check the result
    assert_eq!(result, Value::Number(1.0));
    
    // End the profiling session
    assert!(interpreter.end_profiling_session().is_ok());
    
    // Generate a report
    let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
    
    // Check that the report contains conditional operations
    assert!(report.contains("If"));
}
