// src/profiling/integration.rs - Integration with the interpreter

use std::sync::Arc;
use std::time::Instant;

use crate::ast::NodeType;
use crate::error::LangError;
use crate::interpreter::Interpreter;
use crate::profiling::{Profiler, SpanType, MetricValue, OperationType};

/// Extension trait to add profiling capabilities to the interpreter
pub trait ProfilingInterpreter {
    /// Get the profiler
    fn profiler(&self) -> Option<&Profiler>;
    
    /// Get a mutable reference to the profiler
    fn profiler_mut(&mut self) -> Option<&mut Profiler>;
    
    /// Set the profiler
    fn set_profiler(&mut self, profiler: Profiler);
    
    /// Enable profiling
    fn enable_profiling(&mut self);
    
    /// Disable profiling
    fn disable_profiling(&mut self);
    
    /// Start a profiling session
    fn start_profiling_session(&mut self, name: &str) -> Result<(), LangError>;
    
    /// End the current profiling session
    fn end_profiling_session(&mut self) -> Result<(), LangError>;
    
    /// Generate a profiling report
    fn generate_profiling_report(&self, format: crate::profiling::ReportFormat) -> Result<String, LangError>;
}

impl ProfilingInterpreter for Interpreter {
    fn profiler(&self) -> Option<&Profiler> {
        self.profiler.as_ref()
    }
    
    fn profiler_mut(&mut self) -> Option<&mut Profiler> {
        self.profiler.as_mut()
    }
    
    fn set_profiler(&mut self, profiler: Profiler) {
        self.profiler = Some(profiler);
    }
    
    fn enable_profiling(&mut self) {
        if let Some(profiler) = self.profiler_mut() {
            profiler.set_enabled(true);
        } else {
            let mut profiler = Profiler::new();
            profiler.set_enabled(true);
            self.profiler = Some(profiler);
        }
    }
    
    fn disable_profiling(&mut self) {
        if let Some(profiler) = self.profiler_mut() {
            profiler.set_enabled(false);
        }
    }
    
    fn start_profiling_session(&mut self, name: &str) -> Result<(), LangError> {
        if let Some(profiler) = self.profiler_mut() {
            profiler.start_session(name).map_err(|e| e.into())
        } else {
            let mut profiler = Profiler::new();
            profiler.set_enabled(true);
            profiler.start_session(name).map_err(|e| e.into())?;
            self.profiler = Some(profiler);
            Ok(())
        }
    }
    
    fn end_profiling_session(&mut self) -> Result<(), LangError> {
        if let Some(profiler) = self.profiler_mut() {
            profiler.end_session().map(|_| ()).map_err(|e| e.into())
        } else {
            Err(LangError::runtime_error("No active profiling session"))
        }
    }
    
    fn generate_profiling_report(&self, format: crate::profiling::ReportFormat) -> Result<String, LangError> {
        if let Some(profiler) = self.profiler() {
            profiler.generate_report(format).map_err(|e| e.into())
        } else {
            Err(LangError::runtime_error("No profiler available"))
        }
    }
}

/// Extension methods for the interpreter to profile operations
impl Interpreter {
    /// Profile the execution of a node
    pub fn profile_execute_node(&mut self, node: &crate::ast::ASTNode) -> Result<crate::value::Value, LangError> {
        // If profiling is not enabled, just execute the node normally
        if self.profiler.is_none() || !self.profiler.as_ref().unwrap().is_enabled() {
            return self.execute_node(node);
        }
        
        // Get the span name based on the node type
        let span_name = match &node.node_type {
            NodeType::Number(_) => "Number",
            NodeType::Boolean(_) => "Boolean",
            NodeType::String(_) => "String",
            NodeType::Null => "Null",
            NodeType::Variable(name) => name,
            NodeType::VariableDeclaration { name, .. } => name,
            NodeType::FunctionDeclaration { name, .. } => name,
            NodeType::FunctionCall { .. } => "FunctionCall",
            NodeType::Return(_) => "Return",
            NodeType::Print(_) => "Print",
            NodeType::Block(_) => "Block",
            NodeType::If { .. } => "If",
            NodeType::BinaryOp { op, .. } => op,
            NodeType::UnaryOp { op, .. } => op,
            NodeType::ObjectLiteral(_) => "ObjectLiteral",
            NodeType::ArrayLiteral(_) => "ArrayLiteral",
            NodeType::PropertyAccess { .. } => "PropertyAccess",
            NodeType::PropertyAssignment { .. } => "PropertyAssignment",
            NodeType::IndexAccess { .. } => "IndexAccess",
            NodeType::IndexAssignment { .. } => "IndexAssignment",
            NodeType::StringDictLookup(key) => key,
            NodeType::StringDictFormat { key, .. } => key,
            NodeType::UserInput => "UserInput",
            _ => "Unknown",
        };
        
        // Determine the span type based on the node type
        let span_type = match &node.node_type {
            NodeType::FunctionDeclaration { .. } => SpanType::Function,
            NodeType::Block(_) => SpanType::Block,
            _ => SpanType::Expression,
        };
        
        // Start a span for this node
        let profiler = self.profiler.as_mut().unwrap();
        let span_guard = profiler.start_span(span_name, span_type).map_err(|e| e.into())?;
        
        // Record the operation type
        let operation_type = match &node.node_type {
            NodeType::BinaryOp { op, .. } => {
                match op.as_str() {
                    "+" | "-" | "*" | "/" => OperationType::Arithmetic,
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => OperationType::Arithmetic,
                    "&&" | "||" => OperationType::Arithmetic,
                    _ => OperationType::Other,
                }
            },
            NodeType::UnaryOp { op, .. } => {
                match op.as_str() {
                    "-" => OperationType::Arithmetic,
                    "!" => OperationType::Arithmetic,
                    _ => OperationType::Other,
                }
            },
            NodeType::String(_) => OperationType::String,
            NodeType::ArrayLiteral(_) | NodeType::IndexAccess { .. } | NodeType::IndexAssignment { .. } => OperationType::Array,
            NodeType::ObjectLiteral(_) | NodeType::PropertyAccess { .. } | NodeType::PropertyAssignment { .. } => OperationType::Object,
            NodeType::FunctionDeclaration { .. } | NodeType::FunctionCall { .. } => OperationType::Function,
            NodeType::Variable(_) | NodeType::VariableDeclaration { .. } => OperationType::Variable,
            NodeType::PropertyAccess { .. } | NodeType::PropertyAssignment { .. } => OperationType::Property,
            NodeType::StringDictLookup(_) | NodeType::StringDictFormat { .. } => OperationType::StringDictionary,
            _ => OperationType::Other,
        };
        
        // Record the operation
        if let Some(profiler) = self.profiler_mut() {
            if let Some(op_collector) = profiler.operation_metrics() {
                op_collector.record_operation(operation_type);
            }
        }
        
        // Execute the node and measure the time
        let start_time = Instant::now();
        let result = self.execute_node(node);
        let duration = start_time.elapsed();
        
        // Record the execution time
        if let Some(profiler) = self.profiler_mut() {
            profiler.record_metric("execution_time", MetricValue::from_duration(duration)).ok();
        }
        
        // Return the result
        result
    }
    
    /// Profile the execution of multiple nodes
    pub fn profile_execute_nodes(&mut self, nodes: &[crate::ast::ASTNode]) -> Result<crate::value::Value, LangError> {
        // If profiling is not enabled, just execute the nodes normally
        if self.profiler.is_none() || !self.profiler.as_ref().unwrap().is_enabled() {
            return self.execute_nodes(nodes);
        }
        
        // Start a span for this block of nodes
        let profiler = self.profiler.as_mut().unwrap();
        let span_guard = profiler.start_span("ExecuteNodes", SpanType::Block).map_err(|e| e.into())?;
        
        // Execute the nodes and measure the time
        let start_time = Instant::now();
        let result = self.execute_nodes(nodes);
        let duration = start_time.elapsed();
        
        // Record the execution time
        if let Some(profiler) = self.profiler_mut() {
            profiler.record_metric("execution_time", MetricValue::from_duration(duration)).ok();
        }
        
        // Return the result
        result
    }
}
