// Syntactic & semantic checking API module for LSP-like Component
//
// This module provides APIs for syntactic and semantic checking of Anarchy Inference code,
// offering standardized interfaces for error detection and validation.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, Diagnostic, DiagnosticSeverity};
use crate::language_hub_server::lsp::document::{Document, DocumentManager, SharedDocumentManager};
use crate::language_hub_server::lsp::parser_integration::{AstNode, ParseResult};
use crate::language_hub_server::lsp::diagnostic_provider::{DiagnosticProvider, SharedDiagnosticProvider};
use crate::language_hub_server::lsp::semantic_analyzer::{SemanticAnalyzer, SharedSemanticAnalyzer};
use crate::language_hub_server::lsp::type_checker::{TypeChecker, SharedTypeChecker};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Checking level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckingLevel {
    /// Syntax only
    Syntax,
    
    /// Syntax and semantics
    Semantics,
    
    /// Syntax, semantics, and types
    Types,
    
    /// Syntax, semantics, types, and style
    Style,
}

impl Default for CheckingLevel {
    fn default() -> Self {
        CheckingLevel::Semantics
    }
}

/// Checking options
#[derive(Debug, Clone)]
pub struct CheckingOptions {
    /// The checking level
    pub level: CheckingLevel,
    
    /// Whether to check imports
    pub check_imports: bool,
    
    /// Whether to check unused variables
    pub check_unused: bool,
    
    /// Whether to check for deprecated features
    pub check_deprecated: bool,
    
    /// Whether to check for performance issues
    pub check_performance: bool,
    
    /// Whether to check for security issues
    pub check_security: bool,
    
    /// Maximum number of diagnostics to return
    pub max_diagnostics: usize,
}

impl Default for CheckingOptions {
    fn default() -> Self {
        CheckingOptions {
            level: CheckingLevel::default(),
            check_imports: true,
            check_unused: true,
            check_deprecated: true,
            check_performance: true,
            check_security: true,
            max_diagnostics: 100,
        }
    }
}

/// Checking request
#[derive(Debug, Clone)]
pub struct CheckingRequest {
    /// The document URI
    pub document_uri: String,
    
    /// The document text
    pub text: Option<String>,
    
    /// The checking options
    pub options: Option<CheckingOptions>,
    
    /// The AST of the document
    pub ast: Option<AstNode>,
    
    /// The parse result
    pub parse_result: Option<ParseResult>,
}

impl Default for CheckingRequest {
    fn default() -> Self {
        CheckingRequest {
            document_uri: String::new(),
            text: None,
            options: None,
            ast: None,
            parse_result: None,
        }
    }
}

/// Checking response
#[derive(Debug, Clone)]
pub struct CheckingResponse {
    /// The diagnostics
    pub diagnostics: Vec<Diagnostic>,
    
    /// Whether the document is valid
    pub is_valid: bool,
    
    /// The checking level that was applied
    pub level_applied: CheckingLevel,
    
    /// The number of syntax errors
    pub syntax_error_count: usize,
    
    /// The number of semantic errors
    pub semantic_error_count: usize,
    
    /// The number of type errors
    pub type_error_count: usize,
    
    /// The number of style issues
    pub style_issue_count: usize,
}

/// Syntactic & semantic checking API
pub struct CheckingApi {
    /// The document manager
    document_manager: SharedDocumentManager,
    
    /// The diagnostic provider
    diagnostic_provider: SharedDiagnosticProvider,
    
    /// The semantic analyzer
    semantic_analyzer: SharedSemanticAnalyzer,
    
    /// The type checker
    type_checker: SharedTypeChecker,
}

impl CheckingApi {
    /// Create a new checking API
    pub fn new(
        document_manager: SharedDocumentManager,
        diagnostic_provider: SharedDiagnosticProvider,
        semantic_analyzer: SharedSemanticAnalyzer,
        type_checker: SharedTypeChecker
    ) -> Self {
        CheckingApi {
            document_manager,
            diagnostic_provider,
            semantic_analyzer,
            type_checker,
        }
    }
    
    /// Check a document
    pub fn check_document(
        &self,
        request: CheckingRequest
    ) -> Result<CheckingResponse, String> {
        // Get the document
        let document = if let Some(text) = &request.text {
            // Create a temporary document with the provided text
            Document::new(request.document_uri.clone(), text.clone())
        } else {
            // Get the document from the document manager
            self.get_document(&request.document_uri)?
        };
        
        // Get the options
        let options = request.options.unwrap_or_default();
        
        // Get or create the AST
        let parse_result = if let Some(parse_result) = request.parse_result {
            parse_result
        } else if let Some(ast) = request.ast {
            ParseResult {
                ast,
                errors: Vec::new(),
            }
        } else {
            // Parse the document
            self.parse_document(&document)?
        };
        
        // Collect diagnostics based on the checking level
        let mut diagnostics = Vec::new();
        let mut syntax_error_count = 0;
        let mut semantic_error_count = 0;
        let mut type_error_count = 0;
        let mut style_issue_count = 0;
        
        // Always check syntax
        let syntax_diagnostics = self.check_syntax(&document, &parse_result)?;
        syntax_error_count = syntax_diagnostics.len();
        diagnostics.extend(syntax_diagnostics);
        
        // Check semantics if requested
        if options.level >= CheckingLevel::Semantics {
            let semantic_diagnostics = self.check_semantics(&document, &parse_result.ast, &options)?;
            semantic_error_count = semantic_diagnostics.len();
            diagnostics.extend(semantic_diagnostics);
        }
        
        // Check types if requested
        if options.level >= CheckingLevel::Types {
            let type_diagnostics = self.check_types(&document, &parse_result.ast, &options)?;
            type_error_count = type_diagnostics.len();
            diagnostics.extend(type_diagnostics);
        }
        
        // Check style if requested
        if options.level >= CheckingLevel::Style {
            let style_diagnostics = self.check_style(&document, &parse_result.ast, &options)?;
            style_issue_count = style_diagnostics.len();
            diagnostics.extend(style_diagnostics);
        }
        
        // Limit the number of diagnostics
        if diagnostics.len() > options.max_diagnostics {
            diagnostics.truncate(options.max_diagnostics);
        }
        
        // Determine if the document is valid
        let is_valid = syntax_error_count == 0 && 
                      (options.level < CheckingLevel::Semantics || semantic_error_count == 0) &&
                      (options.level < CheckingLevel::Types || type_error_count == 0);
        
        // Create the response
        let response = CheckingResponse {
            diagnostics,
            is_valid,
            level_applied: options.level,
            syntax_error_count,
            semantic_error_count,
            type_error_count,
            style_issue_count,
        };
        
        Ok(response)
    }
    
    /// Check syntax only
    pub fn check_syntax_only(
        &self,
        document_uri: &str,
        text: Option<String>
    ) -> Result<CheckingResponse, String> {
        let request = CheckingRequest {
            document_uri: document_uri.to_string(),
            text,
            options: Some(CheckingOptions {
                level: CheckingLevel::Syntax,
                ..Default::default()
            }),
            ast: None,
            parse_result: None,
        };
        
        self.check_document(request)
    }
    
    /// Check semantics only
    pub fn check_semantics_only(
        &self,
        document_uri: &str,
        text: Option<String>,
        ast: Option<AstNode>
    ) -> Result<CheckingResponse, String> {
        let request = CheckingRequest {
            document_uri: document_uri.to_string(),
            text,
            options: Some(CheckingOptions {
                level: CheckingLevel::Semantics,
                ..Default::default()
            }),
            ast,
            parse_result: None,
        };
        
        self.check_document(request)
    }
    
    /// Check types only
    pub fn check_types_only(
        &self,
        document_uri: &str,
        text: Option<String>,
        ast: Option<AstNode>
    ) -> Result<CheckingResponse, String> {
        let request = CheckingRequest {
            document_uri: document_uri.to_string(),
            text,
            options: Some(CheckingOptions {
                level: CheckingLevel::Types,
                check_imports: false,
                check_unused: false,
                check_deprecated: false,
                check_performance: false,
                check_security: false,
                ..Default::default()
            }),
            ast,
            parse_result: None,
        };
        
        self.check_document(request)
    }
    
    /// Check style only
    pub fn check_style_only(
        &self,
        document_uri: &str,
        text: Option<String>,
        ast: Option<AstNode>
    ) -> Result<CheckingResponse, String> {
        let request = CheckingRequest {
            document_uri: document_uri.to_string(),
            text,
            options: Some(CheckingOptions {
                level: CheckingLevel::Style,
                check_imports: false,
                check_unused: false,
                check_deprecated: false,
                check_performance: false,
                check_security: false,
                ..Default::default()
            }),
            ast,
            parse_result: None,
        };
        
        self.check_document(request)
    }
    
    /// Check a specific node
    pub fn check_node(
        &self,
        document_uri: &str,
        node: &AstNode,
        options: Option<CheckingOptions>
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the document
        let document = self.get_document(document_uri)?;
        
        // Get the options
        let options = options.unwrap_or_default();
        
        // Collect diagnostics based on the checking level
        let mut diagnostics = Vec::new();
        
        // Always check syntax
        let syntax_diagnostics = self.check_node_syntax(&document, node)?;
        diagnostics.extend(syntax_diagnostics);
        
        // Check semantics if requested
        if options.level >= CheckingLevel::Semantics {
            let semantic_diagnostics = self.check_node_semantics(&document, node, &options)?;
            diagnostics.extend(semantic_diagnostics);
        }
        
        // Check types if requested
        if options.level >= CheckingLevel::Types {
            let type_diagnostics = self.check_node_types(&document, node, &options)?;
            diagnostics.extend(type_diagnostics);
        }
        
        // Check style if requested
        if options.level >= CheckingLevel::Style {
            let style_diagnostics = self.check_node_style(&document, node, &options)?;
            diagnostics.extend(style_diagnostics);
        }
        
        // Limit the number of diagnostics
        if diagnostics.len() > options.max_diagnostics {
            diagnostics.truncate(options.max_diagnostics);
        }
        
        Ok(diagnostics)
    }
    
    /// Validate a document
    pub fn validate_document(
        &self,
        document_uri: &str,
        text: Option<String>,
        level: Option<CheckingLevel>
    ) -> Result<bool, String> {
        let request = CheckingRequest {
            document_uri: document_uri.to_string(),
            text,
            options: Some(CheckingOptions {
                level: level.unwrap_or_default(),
                ..Default::default()
            }),
            ast: None,
            parse_result: None,
        };
        
        let response = self.check_document(request)?;
        
        Ok(response.is_valid)
    }
    
    /// Get document
    fn get_document(&self, uri: &str) -> Result<Document, String> {
        let document_manager = self.document_manager.lock().unwrap();
        document_manager.get_document(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))
            .map(|doc| doc.clone())
    }
    
    /// Parse document
    fn parse_document(&self, document: &Document) -> Result<ParseResult, String> {
        // Get the diagnostic provider
        let diagnostic_provider = self.diagnostic_provider.lock().unwrap();
        
        // Parse the document
        diagnostic_provider.parse_document(document)
    }
    
    /// Check syntax
    fn check_syntax(
        &self,
        document: &Document,
        parse_result: &ParseResult
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the diagnostic provider
        let diagnostic_provider = self.diagnostic_provider.lock().unwrap();
        
        // Get syntax diagnostics
        let diagnostics = diagnostic_provider.get_syntax_diagnostics(document, parse_result);
        
        Ok(diagnostics)
    }
    
    /// Check semantics
    fn check_semantics(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &CheckingOptions
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the semantic analyzer
        let semantic_analyzer = self.semantic_analyzer.lock().unwrap();
        
        // Get semantic diagnostics
        let mut diagnostics = semantic_analyzer.analyze(document, ast);
        
        // Filter diagnostics based on options
        diagnostics = diagnostics.into_iter()
            .filter(|diagnostic| {
                // Keep all errors
                if diagnostic.severity.unwrap_or(1) <= 1 {
                    return true;
                }
                
                // Filter based on options
                match diagnostic.code.as_ref().map(|s| s.as_str()) {
                    Some("unused-var") | Some("unused-import") | Some("unused-function") => options.check_unused,
                    Some("import-error") | Some("module-not-found") => options.check_imports,
                    Some("deprecated") => options.check_deprecated,
                    Some("performance") => options.check_performance,
                    Some("security") => options.check_security,
                    _ => true,
                }
            })
            .collect();
        
        Ok(diagnostics)
    }
    
    /// Check types
    fn check_types(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &CheckingOptions
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the type checker
        let type_checker = self.type_checker.lock().unwrap();
        
        // Get type diagnostics
        let diagnostics = type_checker.check(document, ast);
        
        Ok(diagnostics)
    }
    
    /// Check style
    fn check_style(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &CheckingOptions
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the diagnostic provider
        let diagnostic_provider = self.diagnostic_provider.lock().unwrap();
        
        // Get style diagnostics
        let diagnostics = diagnostic_provider.get_style_diagnostics(document, ast);
        
        Ok(diagnostics)
    }
    
    /// Check node syntax
    fn check_node_syntax(
        &self,
        document: &Document,
        node: &AstNode
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the diagnostic provider
        let diagnostic_provider = self.diagnostic_provider.lock().unwrap();
        
        // Get syntax diagnostics for the node
        let diagnostics = diagnostic_provider.get_node_syntax_diagnostics(document, node);
        
        Ok(diagnostics)
    }
    
    /// Check node semantics
    fn check_node_semantics(
        &self,
        document: &Document,
        node: &AstNode,
        options: &CheckingOptions
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the semantic analyzer
        let semantic_analyzer = self.semantic_analyzer.lock().unwrap();
        
        // Get semantic diagnostics for the node
        let mut diagnostics = semantic_analyzer.analyze_node(document, node);
        
        // Filter diagnostics based on options
        diagnostics = diagnostics.into_iter()
            .filter(|diagnostic| {
                // Keep all errors
                if diagnostic.severity.unwrap_or(1) <= 1 {
                    return true;
                }
                
                // Filter based on options
                match diagnostic.code.as_ref().map(|s| s.as_str()) {
                    Some("unused-var") | Some("unused-import") | Some("unused-function") => options.check_unused,
                    Some("import-error") | Some("module-not-found") => options.check_imports,
                    Some("deprecated") => options.check_deprecated,
                    Some("performance") => options.check_performance,
                    Some("security") => options.check_security,
                    _ => true,
                }
            })
            .collect();
        
        Ok(diagnostics)
    }
    
    /// Check node types
    fn check_node_types(
        &self,
        document: &Document,
        node: &AstNode,
        options: &CheckingOptions
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the type checker
        let type_checker = self.type_checker.lock().unwrap();
        
        // Get type diagnostics for the node
        let diagnostics = type_checker.check_node(document, node);
        
        Ok(diagnostics)
    }
    
    /// Check node style
    fn check_node_style(
        &self,
        document: &Document,
        node: &AstNode,
        options: &CheckingOptions
    ) -> Result<Vec<Diagnostic>, String> {
        // Get the diagnostic provider
        let diagnostic_provider = self.diagnostic_provider.lock().unwrap();
        
        // Get style diagnostics for the node
        let diagnostics = diagnostic_provider.get_node_style_diagnostics(document, node);
        
        Ok(diagnostics)
    }
}

/// Shared checking API that can be used across threads
pub type SharedCheckingApi = Arc<Mutex<CheckingApi>>;

/// Create a new shared checking API
pub fn create_shared_checking_api(
    document_manager: SharedDocumentManager,
    diagnostic_provider: SharedDiagnosticProvider,
    semantic_analyzer: SharedSemanticAnalyzer,
    type_checker: SharedTypeChecker
) -> SharedCheckingApi {
    Arc::new(Mutex::new(CheckingApi::new(
        document_manager,
        diagnostic_provider,
        semantic_analyzer,
        type_checker
    )))
}
