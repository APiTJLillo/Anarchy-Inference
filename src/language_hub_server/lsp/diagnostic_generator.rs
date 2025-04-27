// Diagnostic generation module for LSP-like Component
//
// This module provides functionality for generating diagnostics from
// Anarchy Inference code, including syntax errors, semantic errors,
// and other issues.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range};
use crate::language_hub_server::lsp::document::Document;
use crate::language_hub_server::lsp::parser_integration::{AstNode, SyntaxError, DiagnosticSeverity};
use crate::language_hub_server::lsp::semantic_analyzer::{SemanticAnalyzer, SharedSemanticAnalyzer, SemanticError};
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Diagnostic information
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The range where the diagnostic applies
    pub range: Range,
    
    /// The severity of the diagnostic
    pub severity: DiagnosticSeverity,
    
    /// The diagnostic code
    pub code: Option<String>,
    
    /// The diagnostic message
    pub message: String,
    
    /// The source of the diagnostic
    pub source: String,
    
    /// Related diagnostic information
    pub related_information: Vec<DiagnosticRelatedInformation>,
    
    /// Tags for the diagnostic
    pub tags: Vec<DiagnosticTag>,
}

/// Related diagnostic information
#[derive(Debug, Clone)]
pub struct DiagnosticRelatedInformation {
    /// The location of the related information
    pub location: DiagnosticLocation,
    
    /// The message of the related information
    pub message: String,
}

/// Diagnostic location
#[derive(Debug, Clone)]
pub struct DiagnosticLocation {
    /// The URI of the location
    pub uri: String,
    
    /// The range of the location
    pub range: Range,
}

/// Diagnostic tag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticTag {
    /// Unused declaration
    Unnecessary = 1,
    
    /// Deprecated declaration
    Deprecated = 2,
}

/// Diagnostic generator for Anarchy Inference code
pub struct DiagnosticGenerator {
    /// The semantic analyzer
    semantic_analyzer: SharedSemanticAnalyzer,
    
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// Cache of diagnostics
    diagnostics_cache: HashMap<String, (i64, Vec<Diagnostic>)>,
}

impl DiagnosticGenerator {
    /// Create a new diagnostic generator
    pub fn new(semantic_analyzer: SharedSemanticAnalyzer, symbol_manager: SharedSymbolManager) -> Self {
        DiagnosticGenerator {
            semantic_analyzer,
            symbol_manager,
            diagnostics_cache: HashMap::new(),
        }
    }
    
    /// Generate diagnostics for a document
    pub fn generate_diagnostics(&mut self, document: &Document, ast: &AstNode) -> Result<Vec<Diagnostic>, String> {
        // Check if we have cached diagnostics for this version
        if let Some((version, diagnostics)) = self.diagnostics_cache.get(&document.uri) {
            if *version == document.version {
                return Ok(diagnostics.clone());
            }
        }
        
        // Generate diagnostics
        let mut diagnostics = Vec::new();
        
        // Add syntax errors
        let syntax_errors = self.get_syntax_errors(document, ast);
        for error in syntax_errors {
            diagnostics.push(Diagnostic {
                range: error.range,
                severity: error.severity,
                code: error.code.clone(),
                message: error.message.clone(),
                source: "anarchy-inference-syntax".to_string(),
                related_information: Vec::new(),
                tags: Vec::new(),
            });
        }
        
        // Add semantic errors
        let semantic_errors = self.get_semantic_errors(document, ast)?;
        for error in semantic_errors {
            diagnostics.push(Diagnostic {
                range: error.range,
                severity: error.severity,
                code: error.code.clone(),
                message: error.message.clone(),
                source: "anarchy-inference-semantic".to_string(),
                related_information: Vec::new(),
                tags: Vec::new(),
            });
        }
        
        // Add style issues
        let style_issues = self.get_style_issues(document, ast);
        for issue in style_issues {
            diagnostics.push(issue);
        }
        
        // Add best practice suggestions
        let best_practices = self.get_best_practice_suggestions(document, ast);
        for suggestion in best_practices {
            diagnostics.push(suggestion);
        }
        
        // Add performance warnings
        let performance_warnings = self.get_performance_warnings(document, ast);
        for warning in performance_warnings {
            diagnostics.push(warning);
        }
        
        // Cache the diagnostics
        self.diagnostics_cache.insert(document.uri.clone(), (document.version, diagnostics.clone()));
        
        Ok(diagnostics)
    }
    
    /// Get syntax errors from the document
    fn get_syntax_errors(&self, document: &Document, ast: &AstNode) -> Vec<SyntaxError> {
        // In a real implementation, this would call the Anarchy Inference parser
        // For now, we'll use a simplified implementation that simulates parsing
        
        let mut errors = Vec::new();
        
        // Check for syntax errors in the document
        if document.text.contains("syntax error") {
            errors.push(SyntaxError {
                range: Range {
                    start: Position { line: 1, character: 0 },
                    end: Position { line: 1, character: 10 },
                },
                message: "Syntax error in document".to_string(),
                code: Some("E001".to_string()),
                severity: DiagnosticSeverity::Error,
            });
        }
        
        // Check for undefined variables
        if document.text.contains("undefined variable") {
            errors.push(SyntaxError {
                range: Range {
                    start: Position { line: 2, character: 0 },
                    end: Position { line: 2, character: 20 },
                },
                message: "Use of undefined variable".to_string(),
                code: Some("E101".to_string()),
                severity: DiagnosticSeverity::Error,
            });
        }
        
        // Check for type errors
        if document.text.contains("type error") {
            errors.push(SyntaxError {
                range: Range {
                    start: Position { line: 3, character: 0 },
                    end: Position { line: 3, character: 15 },
                },
                message: "Type error in expression".to_string(),
                code: Some("E201".to_string()),
                severity: DiagnosticSeverity::Error,
            });
        }
        
        errors
    }
    
    /// Get semantic errors from the document
    fn get_semantic_errors(&self, document: &Document, ast: &AstNode) -> Result<Vec<SemanticError>, String> {
        // Get semantic errors from the semantic analyzer
        let mut analyzer = self.semantic_analyzer.lock().unwrap();
        analyzer.analyze_document(document, ast)
    }
    
    /// Get style issues from the document
    fn get_style_issues(&self, document: &Document, ast: &AstNode) -> Vec<Diagnostic> {
        let mut issues = Vec::new();
        
        // Check for inconsistent indentation
        self.check_indentation(document, &mut issues);
        
        // Check for inconsistent naming conventions
        self.check_naming_conventions(document, ast, &mut issues);
        
        // Check for missing semicolons
        self.check_missing_semicolons(document, ast, &mut issues);
        
        // Check for excessive line length
        self.check_line_length(document, &mut issues);
        
        issues
    }
    
    /// Check for inconsistent indentation
    fn check_indentation(&self, document: &Document, issues: &mut Vec<Diagnostic>) {
        // Get all lines in the document
        let mut prev_indent = 0;
        let mut in_block = false;
        
        for i in 0..document.line_count() {
            if let Some(line) = document.get_line(i as u32) {
                // Count leading spaces
                let indent = line.chars().take_while(|c| *c == ' ').count();
                
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }
                
                // Check for inconsistent indentation
                if in_block && indent != prev_indent + 2 && indent != prev_indent {
                    issues.push(Diagnostic {
                        range: Range {
                            start: Position { line: i as u32, character: 0 },
                            end: Position { line: i as u32, character: line.len() as u32 },
                        },
                        severity: DiagnosticSeverity::Warning,
                        code: Some("S001".to_string()),
                        message: "Inconsistent indentation".to_string(),
                        source: "anarchy-inference-style".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
                
                // Update state
                if line.contains("{") {
                    in_block = true;
                    prev_indent = indent;
                } else if line.contains("}") {
                    in_block = false;
                }
            }
        }
    }
    
    /// Check for inconsistent naming conventions
    fn check_naming_conventions(&self, document: &Document, ast: &AstNode, issues: &mut Vec<Diagnostic>) {
        // Get all identifiers
        let identifiers = AstUtils::get_all_identifiers(ast);
        
        for identifier in identifiers {
            if let Some(name) = identifier.properties.get("name").and_then(|v| v.as_str()) {
                // Check for camelCase for variables and functions
                if identifier.node_type == "Identifier" {
                    let parent = AstUtils::find_parent(ast, &identifier);
                    if let Some(parent) = parent {
                        if parent.node_type == "VariableDeclaration" || parent.node_type == "FunctionDeclaration" {
                            if !is_camel_case(name) && !is_snake_case(name) {
                                issues.push(Diagnostic {
                                    range: identifier.range.clone(),
                                    severity: DiagnosticSeverity::Information,
                                    code: Some("S002".to_string()),
                                    message: format!("Inconsistent naming convention: '{}'. Consider using camelCase or snake_case.", name),
                                    source: "anarchy-inference-style".to_string(),
                                    related_information: Vec::new(),
                                    tags: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Check for missing semicolons
    fn check_missing_semicolons(&self, document: &Document, ast: &AstNode, issues: &mut Vec<Diagnostic>) {
        // Get all statements
        let statements = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "ExpressionStatement" || 
            node.node_type == "VariableDeclaration" || 
            node.node_type == "ReturnStatement"
        });
        
        for statement in statements {
            // Get the line of the statement end
            let line_index = statement.range.end.line;
            if let Some(line) = document.get_line(line_index) {
                // Check if the line ends with a semicolon
                if !line.trim().ends_with(';') {
                    issues.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_index, character: line.len() as u32 },
                            end: Position { line: line_index, character: line.len() as u32 },
                        },
                        severity: DiagnosticSeverity::Information,
                        code: Some("S003".to_string()),
                        message: "Missing semicolon".to_string(),
                        source: "anarchy-inference-style".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    
    /// Check for excessive line length
    fn check_line_length(&self, document: &Document, issues: &mut Vec<Diagnostic>) {
        const MAX_LINE_LENGTH: usize = 80;
        
        for i in 0..document.line_count() {
            if let Some(line) = document.get_line(i as u32) {
                if line.len() > MAX_LINE_LENGTH {
                    issues.push(Diagnostic {
                        range: Range {
                            start: Position { line: i as u32, character: MAX_LINE_LENGTH as u32 },
                            end: Position { line: i as u32, character: line.len() as u32 },
                        },
                        severity: DiagnosticSeverity::Information,
                        code: Some("S004".to_string()),
                        message: format!("Line exceeds {} characters", MAX_LINE_LENGTH),
                        source: "anarchy-inference-style".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    
    /// Get best practice suggestions from the document
    fn get_best_practice_suggestions(&self, document: &Document, ast: &AstNode) -> Vec<Diagnostic> {
        let mut suggestions = Vec::new();
        
        // Check for unused variables
        self.check_unused_variables(document, ast, &mut suggestions);
        
        // Check for magic numbers
        self.check_magic_numbers(document, ast, &mut suggestions);
        
        // Check for complex functions
        self.check_complex_functions(document, ast, &mut suggestions);
        
        // Check for deep nesting
        self.check_deep_nesting(document, ast, &mut suggestions);
        
        suggestions
    }
    
    /// Check for unused variables
    fn check_unused_variables(&self, document: &Document, ast: &AstNode, suggestions: &mut Vec<Diagnostic>) {
        // Get all variable declarations
        let variable_declarations = AstUtils::get_all_variable_declarations(ast);
        
        for declaration in variable_declarations {
            if let Some(name) = declaration.properties.get("name").and_then(|v| v.as_str()) {
                // Get all references to this variable
                let references = AstUtils::get_symbol_references(ast, name);
                
                // If there's only one reference (the declaration itself), the variable is unused
                if references.len() <= 1 {
                    suggestions.push(Diagnostic {
                        range: declaration.range.clone(),
                        severity: DiagnosticSeverity::Information,
                        code: Some("BP001".to_string()),
                        message: format!("Unused variable: '{}'", name),
                        source: "anarchy-inference-best-practices".to_string(),
                        related_information: Vec::new(),
                        tags: vec![DiagnosticTag::Unnecessary],
                    });
                }
            }
        }
    }
    
    /// Check for magic numbers
    fn check_magic_numbers(&self, document: &Document, ast: &AstNode, suggestions: &mut Vec<Diagnostic>) {
        // Get all literals
        let literals = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "Literal" && 
            node.properties.get("literalType").and_then(|v| v.as_str()) == Some("number")
        });
        
        for literal in literals {
            if let Some(value) = literal.properties.get("value").and_then(|v| v.as_f64()) {
                // Skip common values like 0, 1, -1
                if value != 0.0 && value != 1.0 && value != -1.0 {
                    suggestions.push(Diagnostic {
                        range: literal.range.clone(),
                        severity: DiagnosticSeverity::Information,
                        code: Some("BP002".to_string()),
                        message: format!("Consider using a named constant instead of the magic number {}", value),
                        source: "anarchy-inference-best-practices".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    
    /// Check for complex functions
    fn check_complex_functions(&self, document: &Document, ast: &AstNode, suggestions: &mut Vec<Diagnostic>) {
        // Get all function declarations
        let function_declarations = AstUtils::get_all_function_declarations(ast);
        
        for function in function_declarations {
            // Count the number of statements in the function
            let statements = AstUtils::collect_nodes(&function, |node| {
                node.node_type == "ExpressionStatement" || 
                node.node_type == "VariableDeclaration" || 
                node.node_type == "ReturnStatement" ||
                node.node_type == "IfStatement" ||
                node.node_type == "WhileStatement" ||
                node.node_type == "ForStatement"
            });
            
            const MAX_STATEMENTS: usize = 15;
            if statements.len() > MAX_STATEMENTS {
                if let Some(name) = function.properties.get("name").and_then(|v| v.as_str()) {
                    suggestions.push(Diagnostic {
                        range: function.range.clone(),
                        severity: DiagnosticSeverity::Information,
                        code: Some("BP003".to_string()),
                        message: format!("Function '{}' is too complex ({} statements). Consider refactoring.", name, statements.len()),
                        source: "anarchy-inference-best-practices".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    
    /// Check for deep nesting
    fn check_deep_nesting(&self, document: &Document, ast: &AstNode, suggestions: &mut Vec<Diagnostic>) {
        // Get all control flow statements
        let control_flow = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "IfStatement" ||
            node.node_type == "WhileStatement" ||
            node.node_type == "ForStatement"
        });
        
        for statement in control_flow {
            // Calculate nesting depth
            let mut depth = 0;
            let mut current = Some(statement.clone());
            
            while let Some(node) = current {
                let parent = AstUtils::find_parent(ast, &node);
                if let Some(parent) = parent {
                    if parent.node_type == "IfStatement" ||
                       parent.node_type == "WhileStatement" ||
                       parent.node_type == "ForStatement" {
                        depth += 1;
                    }
                    current = Some(parent);
                } else {
                    current = None;
                }
            }
            
            const MAX_NESTING: usize = 3;
            if depth >= MAX_NESTING {
                suggestions.push(Diagnostic {
                    range: statement.range.clone(),
                    severity: DiagnosticSeverity::Information,
                    code: Some("BP004".to_string()),
                    message: format!("Deep nesting (depth {}). Consider refactoring to reduce nesting.", depth),
                    source: "anarchy-inference-best-practices".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }
    }
    
    /// Get performance warnings from the document
    fn get_performance_warnings(&self, document: &Document, ast: &AstNode) -> Vec<Diagnostic> {
        let mut warnings = Vec::new();
        
        // Check for inefficient loops
        self.check_inefficient_loops(document, ast, &mut warnings);
        
        // Check for excessive string concatenation
        self.check_string_concatenation(document, ast, &mut warnings);
        
        // Check for redundant operations
        self.check_redundant_operations(document, ast, &mut warnings);
        
        warnings
    }
    
    /// Check for inefficient loops
    fn check_inefficient_loops(&self, document: &Document, ast: &AstNode, warnings: &mut Vec<Diagnostic>) {
        // Get all loop statements
        let loops = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "WhileStatement" ||
            node.node_type == "ForStatement"
        });
        
        for loop_stmt in loops {
            // Check for array access in loop condition
            if let Some(condition) = loop_stmt.children.first() {
                let array_accesses = AstUtils::collect_nodes(condition, |node| {
                    node.node_type == "MemberExpression" &&
                    node.properties.get("computed").and_then(|v| v.as_bool()) == Some(true)
                });
                
                if !array_accesses.is_empty() {
                    warnings.push(Diagnostic {
                        range: condition.range.clone(),
                        severity: DiagnosticSeverity::Information,
                        code: Some("P001".to_string()),
                        message: "Array access in loop condition may be inefficient".to_string(),
                        source: "anarchy-inference-performance".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    
    /// Check for excessive string concatenation
    fn check_string_concatenation(&self, document: &Document, ast: &AstNode, warnings: &mut Vec<Diagnostic>) {
        // Get all binary expressions with + operator
        let concat_ops = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "BinaryExpression" &&
            node.properties.get("operator").and_then(|v| v.as_str()) == Some("+")
        });
        
        // Build a map of concatenation chains
        let mut concat_chains: HashMap<usize, Vec<AstNode>> = HashMap::new();
        
        for op in concat_ops {
            // Check if either operand is a string
            let left_is_string = op.children.get(0).map_or(false, |child| {
                child.node_type == "Literal" && 
                child.properties.get("literalType").and_then(|v| v.as_str()) == Some("string")
            });
            
            let right_is_string = op.children.get(1).map_or(false, |child| {
                child.node_type == "Literal" && 
                child.properties.get("literalType").and_then(|v| v.as_str()) == Some("string")
            });
            
            if left_is_string || right_is_string {
                // Find the statement this is part of
                let mut current = Some(op.clone());
                while let Some(node) = current {
                    let parent = AstUtils::find_parent(ast, &node);
                    if let Some(parent) = parent {
                        if parent.node_type == "ExpressionStatement" {
                            let chain = concat_chains.entry(parent.range.start.line as usize).or_insert_with(Vec::new);
                            chain.push(op.clone());
                            break;
                        }
                        current = Some(parent);
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Check for chains with more than 3 concatenations
        for (line, chain) in concat_chains {
            if chain.len() >= 3 {
                if let Some(first_op) = chain.first() {
                    warnings.push(Diagnostic {
                        range: first_op.range.clone(),
                        severity: DiagnosticSeverity::Information,
                        code: Some("P002".to_string()),
                        message: format!("Excessive string concatenation ({} operations). Consider using string interpolation or a string builder.", chain.len()),
                        source: "anarchy-inference-performance".to_string(),
                        related_information: Vec::new(),
                        tags: Vec::new(),
                    });
                }
            }
        }
    }
    
    /// Check for redundant operations
    fn check_redundant_operations(&self, document: &Document, ast: &AstNode, warnings: &mut Vec<Diagnostic>) {
        // Check for redundant boolean operations
        let bool_ops = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "BinaryExpression" &&
            (node.properties.get("operator").and_then(|v| v.as_str()) == Some("&&") ||
             node.properties.get("operator").and_then(|v| v.as_str()) == Some("||"))
        });
        
        for op in bool_ops {
            if op.children.len() >= 2 {
                let left = &op.children[0];
                let right = &op.children[1];
                
                // Check for identical operands
                if left.node_type == right.node_type {
                    if left.node_type == "Identifier" {
                        let left_name = left.properties.get("name").and_then(|v| v.as_str());
                        let right_name = right.properties.get("name").and_then(|v| v.as_str());
                        
                        if left_name == right_name {
                            warnings.push(Diagnostic {
                                range: op.range.clone(),
                                severity: DiagnosticSeverity::Warning,
                                code: Some("P003".to_string()),
                                message: "Redundant boolean operation with identical operands".to_string(),
                                source: "anarchy-inference-performance".to_string(),
                                related_information: Vec::new(),
                                tags: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Shared diagnostic generator that can be used across threads
pub type SharedDiagnosticGenerator = Arc<Mutex<DiagnosticGenerator>>;

/// Create a new shared diagnostic generator
pub fn create_shared_diagnostic_generator(
    semantic_analyzer: SharedSemanticAnalyzer,
    symbol_manager: SharedSymbolManager
) -> SharedDiagnosticGenerator {
    Arc::new(Mutex::new(DiagnosticGenerator::new(semantic_analyzer, symbol_manager)))
}

/// Check if a string is in camelCase
fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let first_char = s.chars().next().unwrap();
    if !first_char.is_lowercase() {
        return false;
    }
    
    let has_uppercase = s.chars().any(|c| c.is_uppercase());
    let has_underscore = s.contains('_');
    
    has_uppercase && !has_underscore
}

/// Check if a string is in snake_case
fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    s.chars().all(|c| c.is_lowercase() || c == '_')
}
