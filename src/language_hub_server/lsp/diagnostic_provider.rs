// Diagnostic provider module for LSP-like Component
//
// This module provides diagnostic functionality for Anarchy Inference code,
// including syntax errors, semantic errors, style issues, and best practices.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range};
use crate::language_hub_server::lsp::document::Document;
use crate::language_hub_server::lsp::parser_integration::{AstNode, DiagnosticSeverity};
use crate::language_hub_server::lsp::semantic_analyzer::{SemanticAnalyzer, SharedSemanticAnalyzer, SemanticError};
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager};
use crate::language_hub_server::lsp::type_checker::{TypeChecker, SharedTypeChecker, TypeError};
use crate::language_hub_server::lsp::diagnostic_generator::{DiagnosticGenerator, SharedDiagnosticGenerator, Diagnostic};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Diagnostic options
#[derive(Debug, Clone)]
pub struct DiagnosticOptions {
    /// Whether to report syntax errors
    pub report_syntax_errors: bool,
    
    /// Whether to report semantic errors
    pub report_semantic_errors: bool,
    
    /// Whether to report style issues
    pub report_style_issues: bool,
    
    /// Whether to report best practice suggestions
    pub report_best_practices: bool,
    
    /// Whether to report performance warnings
    pub report_performance_warnings: bool,
    
    /// Maximum number of diagnostics to report
    pub max_diagnostics: usize,
}

impl Default for DiagnosticOptions {
    fn default() -> Self {
        DiagnosticOptions {
            report_syntax_errors: true,
            report_semantic_errors: true,
            report_style_issues: true,
            report_best_practices: true,
            report_performance_warnings: true,
            max_diagnostics: 100,
        }
    }
}

/// Diagnostic provider for Anarchy Inference code
pub struct DiagnosticProvider {
    /// The diagnostic generator
    diagnostic_generator: SharedDiagnosticGenerator,
    
    /// The semantic analyzer
    semantic_analyzer: SharedSemanticAnalyzer,
    
    /// The type checker
    type_checker: SharedTypeChecker,
    
    /// Diagnostic options
    options: DiagnosticOptions,
    
    /// Cache of diagnostics
    diagnostics_cache: HashMap<String, (i64, Vec<Diagnostic>)>,
}

impl DiagnosticProvider {
    /// Create a new diagnostic provider
    pub fn new(
        diagnostic_generator: SharedDiagnosticGenerator,
        semantic_analyzer: SharedSemanticAnalyzer,
        type_checker: SharedTypeChecker,
        options: Option<DiagnosticOptions>
    ) -> Self {
        DiagnosticProvider {
            diagnostic_generator,
            semantic_analyzer,
            type_checker,
            options: options.unwrap_or_default(),
            diagnostics_cache: HashMap::new(),
        }
    }
    
    /// Provide diagnostics for a document
    pub fn provide_diagnostics(&mut self, document: &Document, ast: &AstNode) -> Result<Vec<Diagnostic>, String> {
        // Check if we have cached diagnostics for this version
        if let Some((version, diagnostics)) = self.diagnostics_cache.get(&document.uri) {
            if *version == document.version {
                return Ok(diagnostics.clone());
            }
        }
        
        // Generate diagnostics
        let mut diagnostics = Vec::new();
        
        // Get diagnostics from the diagnostic generator
        if self.options.report_syntax_errors || 
           self.options.report_semantic_errors || 
           self.options.report_style_issues || 
           self.options.report_best_practices || 
           self.options.report_performance_warnings {
            let mut generator = self.diagnostic_generator.lock().unwrap();
            let generator_diagnostics = generator.generate_diagnostics(document, ast)?;
            
            // Filter diagnostics based on options
            for diagnostic in generator_diagnostics {
                let should_include = match diagnostic.source.as_str() {
                    "anarchy-inference-syntax" => self.options.report_syntax_errors,
                    "anarchy-inference-semantic" => self.options.report_semantic_errors,
                    "anarchy-inference-style" => self.options.report_style_issues,
                    "anarchy-inference-best-practices" => self.options.report_best_practices,
                    "anarchy-inference-performance" => self.options.report_performance_warnings,
                    _ => true,
                };
                
                if should_include {
                    diagnostics.push(diagnostic);
                }
            }
        }
        
        // Get type errors from the type checker
        if self.options.report_semantic_errors {
            let mut checker = self.type_checker.lock().unwrap();
            let type_errors = checker.type_check(document, ast)?;
            
            // Convert type errors to diagnostics
            for error in type_errors {
                diagnostics.push(Diagnostic {
                    range: error.range,
                    severity: error.severity,
                    code: error.code.clone(),
                    message: error.message.clone(),
                    source: "anarchy-inference-type".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }
        
        // Limit the number of diagnostics
        if diagnostics.len() > self.options.max_diagnostics {
            diagnostics.truncate(self.options.max_diagnostics);
        }
        
        // Cache the diagnostics
        self.diagnostics_cache.insert(document.uri.clone(), (document.version, diagnostics.clone()));
        
        Ok(diagnostics)
    }
    
    /// Clear diagnostics for a document
    pub fn clear_diagnostics(&mut self, uri: &str) {
        self.diagnostics_cache.remove(uri);
    }
    
    /// Update diagnostic options
    pub fn update_options(&mut self, options: DiagnosticOptions) {
        self.options = options;
        
        // Clear the cache to force regeneration with new options
        self.diagnostics_cache.clear();
    }
    
    /// Get diagnostic options
    pub fn get_options(&self) -> DiagnosticOptions {
        self.options.clone()
    }
    
    /// Check if a document has diagnostics
    pub fn has_diagnostics(&self, uri: &str) -> bool {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            !diagnostics.is_empty()
        } else {
            false
        }
    }
    
    /// Get diagnostics for a document
    pub fn get_diagnostics(&self, uri: &str) -> Option<Vec<Diagnostic>> {
        self.diagnostics_cache.get(uri).map(|(_, diagnostics)| diagnostics.clone())
    }
    
    /// Get diagnostics at a specific position
    pub fn get_diagnostics_at_position(&self, uri: &str, position: Position) -> Vec<Diagnostic> {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| position_in_range(position, &d.range))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get diagnostics by severity
    pub fn get_diagnostics_by_severity(&self, uri: &str, severity: DiagnosticSeverity) -> Vec<Diagnostic> {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| d.severity == severity)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get diagnostics by source
    pub fn get_diagnostics_by_source(&self, uri: &str, source: &str) -> Vec<Diagnostic> {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| d.source == source)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get error count for a document
    pub fn get_error_count(&self, uri: &str) -> usize {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| d.severity == DiagnosticSeverity::Error)
                .count()
        } else {
            0
        }
    }
    
    /// Get warning count for a document
    pub fn get_warning_count(&self, uri: &str) -> usize {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| d.severity == DiagnosticSeverity::Warning)
                .count()
        } else {
            0
        }
    }
    
    /// Get information count for a document
    pub fn get_information_count(&self, uri: &str) -> usize {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| d.severity == DiagnosticSeverity::Information)
                .count()
        } else {
            0
        }
    }
    
    /// Get hint count for a document
    pub fn get_hint_count(&self, uri: &str) -> usize {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.iter()
                .filter(|d| d.severity == DiagnosticSeverity::Hint)
                .count()
        } else {
            0
        }
    }
    
    /// Get total diagnostic count for a document
    pub fn get_total_count(&self, uri: &str) -> usize {
        if let Some((_, diagnostics)) = self.diagnostics_cache.get(uri) {
            diagnostics.len()
        } else {
            0
        }
    }
    
    /// Check if a document has errors
    pub fn has_errors(&self, uri: &str) -> bool {
        self.get_error_count(uri) > 0
    }
    
    /// Check if a document has warnings
    pub fn has_warnings(&self, uri: &str) -> bool {
        self.get_warning_count(uri) > 0
    }
    
    /// Check if a document is error-free
    pub fn is_error_free(&self, uri: &str) -> bool {
        !self.has_errors(uri)
    }
    
    /// Get diagnostic statistics for a document
    pub fn get_diagnostic_statistics(&self, uri: &str) -> DiagnosticStatistics {
        DiagnosticStatistics {
            error_count: self.get_error_count(uri),
            warning_count: self.get_warning_count(uri),
            information_count: self.get_information_count(uri),
            hint_count: self.get_hint_count(uri),
            total_count: self.get_total_count(uri),
        }
    }
}

/// Diagnostic statistics
#[derive(Debug, Clone)]
pub struct DiagnosticStatistics {
    /// Number of errors
    pub error_count: usize,
    
    /// Number of warnings
    pub warning_count: usize,
    
    /// Number of information messages
    pub information_count: usize,
    
    /// Number of hints
    pub hint_count: usize,
    
    /// Total number of diagnostics
    pub total_count: usize,
}

/// Shared diagnostic provider that can be used across threads
pub type SharedDiagnosticProvider = Arc<Mutex<DiagnosticProvider>>;

/// Create a new shared diagnostic provider
pub fn create_shared_diagnostic_provider(
    diagnostic_generator: SharedDiagnosticGenerator,
    semantic_analyzer: SharedSemanticAnalyzer,
    type_checker: SharedTypeChecker,
    options: Option<DiagnosticOptions>
) -> SharedDiagnosticProvider {
    Arc::new(Mutex::new(DiagnosticProvider::new(
        diagnostic_generator,
        semantic_analyzer,
        type_checker,
        options
    )))
}

/// Check if a position is within a range
fn position_in_range(position: Position, range: &Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }
    
    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }
    
    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }
    
    true
}
