// Language Hub Server module for Anarchy Inference
//
// This module integrates all LSP-like components into a unified server
// that provides intelligent code editing capabilities.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

use crate::language_hub_server::lsp::protocol::*;
use crate::language_hub_server::lsp::document::{Document, DocumentManager, SharedDocumentManager, create_shared_document_manager};
use crate::language_hub_server::lsp::server::{Server, SharedServer, create_shared_server};
use crate::language_hub_server::lsp::json_rpc::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};
use crate::language_hub_server::lsp::parser_integration::{AstNode, ParseResult};
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager, create_shared_symbol_manager};
use crate::language_hub_server::lsp::semantic_analyzer::{SemanticAnalyzer, SharedSemanticAnalyzer, create_shared_semantic_analyzer};
use crate::language_hub_server::lsp::type_checker::{TypeChecker, SharedTypeChecker, create_shared_type_checker};
use crate::language_hub_server::lsp::completion_provider::{CompletionProvider, SharedCompletionProvider, create_shared_completion_provider};
use crate::language_hub_server::lsp::diagnostic_provider::{DiagnosticProvider, SharedDiagnosticProvider, create_shared_diagnostic_provider};
use crate::language_hub_server::lsp::formatting_provider::{FormattingProvider, SharedFormattingProvider, create_shared_formatting_provider};
use crate::language_hub_server::lsp::refactoring_provider::{RefactoringProvider, SharedRefactoringProvider, create_shared_refactoring_provider};
use crate::language_hub_server::lsp::symbol_provider::{SymbolProvider, SharedSymbolProvider, create_shared_symbol_provider};

use crate::language_hub_server::lsp::structured_completion_endpoints::{StructuredCompletionEndpoints, SharedStructuredCompletionEndpoints, create_shared_structured_completion_endpoints};
use crate::language_hub_server::lsp::checking_api::{CheckingApi, SharedCheckingApi, create_shared_checking_api};
use crate::language_hub_server::lsp::error_reporting::{ErrorReportingInterface, SharedErrorReportingInterface, create_shared_error_reporting_interface};
use crate::language_hub_server::lsp::ast_manipulation::{AstManipulationEndpoints, SharedAstManipulationEndpoints, create_shared_ast_manipulation_endpoints};

/// Language Hub Server configuration
#[derive(Debug, Clone)]
pub struct LanguageHubServerConfig {
    /// The server host
    pub host: String,
    
    /// The server port
    pub port: u16,
    
    /// The maximum number of concurrent connections
    pub max_connections: usize,
    
    /// Whether to enable logging
    pub enable_logging: bool,
    
    /// The log file path
    pub log_file: Option<String>,
    
    /// Whether to enable telemetry
    pub enable_telemetry: bool,
    
    /// Whether to enable auto-completion
    pub enable_completion: bool,
    
    /// Whether to enable diagnostics
    pub enable_diagnostics: bool,
    
    /// Whether to enable formatting
    pub enable_formatting: bool,
    
    /// Whether to enable refactoring
    pub enable_refactoring: bool,
    
    /// Whether to enable symbol search
    pub enable_symbol_search: bool,
}

impl Default for LanguageHubServerConfig {
    fn default() -> Self {
        LanguageHubServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 10,
            enable_logging: true,
            log_file: None,
            enable_telemetry: false,
            enable_completion: true,
            enable_diagnostics: true,
            enable_formatting: true,
            enable_refactoring: true,
            enable_symbol_search: true,
        }
    }
}

/// Language Hub Server
pub struct LanguageHubServer {
    /// The server configuration
    config: LanguageHubServerConfig,
    
    /// The document manager
    document_manager: SharedDocumentManager,
    
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// The semantic analyzer
    semantic_analyzer: SharedSemanticAnalyzer,
    
    /// The type checker
    type_checker: SharedTypeChecker,
    
    /// The completion provider
    completion_provider: SharedCompletionProvider,
    
    /// The diagnostic provider
    diagnostic_provider: SharedDiagnosticProvider,
    
    /// The formatting provider
    formatting_provider: SharedFormattingProvider,
    
    /// The refactoring provider
    refactoring_provider: SharedRefactoringProvider,
    
    /// The symbol provider
    symbol_provider: SharedSymbolProvider,
    
    /// The structured completion endpoints
    structured_completion_endpoints: SharedStructuredCompletionEndpoints,
    
    /// The checking API
    checking_api: SharedCheckingApi,
    
    /// The error reporting interface
    error_reporting_interface: SharedErrorReportingInterface,
    
    /// The AST manipulation endpoints
    ast_manipulation_endpoints: SharedAstManipulationEndpoints,
    
    /// The LSP server
    server: SharedServer,
}

impl LanguageHubServer {
    /// Create a new Language Hub Server
    pub fn new(config: Option<LanguageHubServerConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        // Create the document manager
        let document_manager = create_shared_document_manager();
        
        // Create the symbol manager
        let symbol_manager = create_shared_symbol_manager(document_manager.clone());
        
        // Create the semantic analyzer
        let semantic_analyzer = create_shared_semantic_analyzer(document_manager.clone(), symbol_manager.clone());
        
        // Create the type checker
        let type_checker = create_shared_type_checker(document_manager.clone(), symbol_manager.clone());
        
        // Create the diagnostic provider
        let diagnostic_provider = create_shared_diagnostic_provider(document_manager.clone(), semantic_analyzer.clone(), type_checker.clone());
        
        // Create the completion provider
        let completion_provider = create_shared_completion_provider(document_manager.clone(), symbol_manager.clone());
        
        // Create the formatting provider
        let formatting_provider = create_shared_formatting_provider(document_manager.clone());
        
        // Create the refactoring provider
        let refactoring_provider = create_shared_refactoring_provider(document_manager.clone(), symbol_manager.clone());
        
        // Create the symbol provider
        let symbol_provider = create_shared_symbol_provider(document_manager.clone(), symbol_manager.clone(), None);
        
        // Create the structured completion endpoints
        let structured_completion_endpoints = create_shared_structured_completion_endpoints(
            document_manager.clone(),
            symbol_manager.clone(),
            completion_provider.clone()
        );
        
        // Create the checking API
        let checking_api = create_shared_checking_api(
            document_manager.clone(),
            diagnostic_provider.clone(),
            semantic_analyzer.clone(),
            type_checker.clone()
        );
        
        // Create the error reporting interface
        let error_reporting_interface = create_shared_error_reporting_interface(
            document_manager.clone(),
            checking_api.clone()
        );
        
        // Create the AST manipulation endpoints
        let ast_manipulation_endpoints = create_shared_ast_manipulation_endpoints(
            document_manager.clone(),
            refactoring_provider.clone()
        );
        
        // Create the LSP server
        let server = create_shared_server(
            document_manager.clone(),
            symbol_manager.clone(),
            semantic_analyzer.clone(),
            type_checker.clone(),
            completion_provider.clone(),
            diagnostic_provider.clone(),
            formatting_provider.clone(),
            refactoring_provider.clone(),
            symbol_provider.clone()
        );
        
        LanguageHubServer {
            config,
            document_manager,
            symbol_manager,
            semantic_analyzer,
            type_checker,
            completion_provider,
            diagnostic_provider,
            formatting_provider,
            refactoring_provider,
            symbol_provider,
            structured_completion_endpoints,
            checking_api,
            error_reporting_interface,
            ast_manipulation_endpoints,
            server,
        }
    }
    
    /// Start the server
    pub fn start(&self) -> Result<(), String> {
        // Create the TCP listener
        let address = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&address)
            .map_err(|e| format!("Failed to bind to {}: {}", address, e))?;
        
        println!("Language Hub Server started on {}", address);
        
        // Accept connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Clone the server for the new connection
                    let server = self.server.clone();
                    
                    // Handle the connection in a new thread
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_connection(stream, server) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a connection
    fn handle_connection(stream: TcpStream, server: SharedServer) -> Result<(), String> {
        // Get the server
        let mut server = server.lock().unwrap();
        
        // Handle the connection
        server.handle_connection(stream)
    }
    
    /// Handle a request
    pub fn handle_request(&self, request: &str) -> Result<String, String> {
        // Parse the request
        let message: JsonRpcMessage = serde_json::from_str(request)
            .map_err(|e| format!("Failed to parse request: {}", e))?;
        
        // Handle the message
        match message {
            JsonRpcMessage::Request(request) => {
                self.handle_json_rpc_request(&request)
            }
            JsonRpcMessage::Notification(notification) => {
                self.handle_json_rpc_notification(&notification)?;
                Ok("".to_string())
            }
            JsonRpcMessage::Response(_) => {
                Err("Unexpected response message".to_string())
            }
        }
    }
    
    /// Handle a JSON-RPC request
    fn handle_json_rpc_request(&self, request: &JsonRpcRequest) -> Result<String, String> {
        // Handle the request based on the method
        let result = match request.method.as_str() {
            // LSP methods
            "initialize" => {
                let server = self.server.lock().unwrap();
                let result = server.initialize(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "shutdown" => {
                let server = self.server.lock().unwrap();
                let result = server.shutdown()?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/completion" => {
                let server = self.server.lock().unwrap();
                let result = server.completion(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/definition" => {
                let server = self.server.lock().unwrap();
                let result = server.definition(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/references" => {
                let server = self.server.lock().unwrap();
                let result = server.references(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/hover" => {
                let server = self.server.lock().unwrap();
                let result = server.hover(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/documentSymbol" => {
                let server = self.server.lock().unwrap();
                let result = server.document_symbol(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/formatting" => {
                let server = self.server.lock().unwrap();
                let result = server.formatting(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/rangeFormatting" => {
                let server = self.server.lock().unwrap();
                let result = server.range_formatting(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/onTypeFormatting" => {
                let server = self.server.lock().unwrap();
                let result = server.on_type_formatting(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/rename" => {
                let server = self.server.lock().unwrap();
                let result = server.rename(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "textDocument/codeAction" => {
                let server = self.server.lock().unwrap();
                let result = server.code_action(request.params.clone())?;
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            
            // Structured completion endpoints
            "anarchy/completion/getCompletionItems" => {
                let structured_completion_endpoints = self.structured_completion_endpoints.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let position = Position {
                    line: request_params["position"]["line"].as_u64()
                        .ok_or_else(|| "Missing position.line parameter".to_string())? as u32,
                    character: request_params["position"]["character"].as_u64()
                        .ok_or_else(|| "Missing position.character parameter".to_string())? as u32,
                };
                
                // Create the request
                let completion_request = crate::language_hub_server::lsp::structured_completion_endpoints::StructuredCompletionRequest {
                    document_uri,
                    position,
                    context: None,
                    ast: None,
                    parse_result: None,
                    include_snippets: request_params["includeSnippets"].as_bool().unwrap_or(true),
                    include_keywords: request_params["includeKeywords"].as_bool().unwrap_or(true),
                    include_symbols: request_params["includeSymbols"].as_bool().unwrap_or(true),
                    include_members: request_params["includeMembers"].as_bool().unwrap_or(true),
                    include_types: request_params["includeTypes"].as_bool().unwrap_or(true),
                    max_items: request_params["maxItems"].as_u64().unwrap_or(100) as usize,
                };
                
                // Get completion items
                let response = structured_completion_endpoints.get_completion_items(completion_request)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "items": response.items,
                    "isIncomplete": response.is_incomplete
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "anarchy/completion/getAstCompletionSuggestions" => {
                let structured_completion_endpoints = self.structured_completion_endpoints.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let position = Position {
                    line: request_params["position"]["line"].as_u64()
                        .ok_or_else(|| "Missing position.line parameter".to_string())? as u32,
                    character: request_params["position"]["character"].as_u64()
                        .ok_or_else(|| "Missing position.character parameter".to_string())? as u32,
                };
                
                // Create the request
                let completion_request = crate::language_hub_server::lsp::structured_completion_endpoints::StructuredCompletionRequest {
                    document_uri,
                    position,
                    context: None,
                    ast: None,
                    parse_result: None,
                    include_snippets: request_params["includeSnippets"].as_bool().unwrap_or(true),
                    include_keywords: request_params["includeKeywords"].as_bool().unwrap_or(true),
                    include_symbols: request_params["includeSymbols"].as_bool().unwrap_or(true),
                    include_members: request_params["includeMembers"].as_bool().unwrap_or(true),
                    include_types: request_params["includeTypes"].as_bool().unwrap_or(true),
                    max_items: request_params["maxItems"].as_u64().unwrap_or(100) as usize,
                };
                
                // Get AST completion suggestions
                let response = structured_completion_endpoints.get_ast_completion_suggestions(completion_request)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "items": response.items,
                    "isIncomplete": response.is_incomplete
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            
            // Checking API
            "anarchy/checking/checkDocument" => {
                let checking_api = self.checking_api.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let text = if request_params["text"].is_string() {
                    Some(request_params["text"].as_str().unwrap().to_string())
                } else {
                    None
                };
                
                // Create the request
                let checking_request = crate::language_hub_server::lsp::checking_api::CheckingRequest {
                    document_uri,
                    text,
                    options: None,
                    ast: None,
                    parse_result: None,
                };
                
                // Check the document
                let response = checking_api.check_document(checking_request)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "diagnostics": response.diagnostics,
                    "isValid": response.is_valid,
                    "levelApplied": response.level_applied as u8,
                    "syntaxErrorCount": response.syntax_error_count,
                    "semanticErrorCount": response.semantic_error_count,
                    "typeErrorCount": response.type_error_count,
                    "styleIssueCount": response.style_issue_count
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "anarchy/checking/validateDocument" => {
                let checking_api = self.checking_api.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let text = if request_params["text"].is_string() {
                    Some(request_params["text"].as_str().unwrap().to_string())
                } else {
                    None
                };
                
                let level = if request_params["level"].is_u64() {
                    Some(match request_params["level"].as_u64().unwrap() {
                        0 => crate::language_hub_server::lsp::checking_api::CheckingLevel::Syntax,
                        1 => crate::language_hub_server::lsp::checking_api::CheckingLevel::Semantics,
                        2 => crate::language_hub_server::lsp::checking_api::CheckingLevel::Types,
                        3 => crate::language_hub_server::lsp::checking_api::CheckingLevel::Style,
                        _ => crate::language_hub_server::lsp::checking_api::CheckingLevel::Semantics,
                    })
                } else {
                    None
                };
                
                // Validate the document
                let is_valid = checking_api.validate_document(&document_uri, text, level)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "isValid": is_valid
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            
            // Error reporting interface
            "anarchy/errorReporting/reportErrors" => {
                let error_reporting_interface = self.error_reporting_interface.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let text = if request_params["text"].is_string() {
                    Some(request_params["text"].as_str().unwrap().to_string())
                } else {
                    None
                };
                
                // Create the request
                let error_reporting_request = crate::language_hub_server::lsp::error_reporting::ErrorReportingRequest {
                    document_uri,
                    text,
                    options: None,
                    checking_request: None,
                };
                
                // Report errors
                let json = error_reporting_interface.report_errors_as_json(error_reporting_request)?;
                
                // Return the JSON directly
                json
            }
            
            // AST manipulation endpoints
            "anarchy/astManipulation/applyTransformation" => {
                let ast_manipulation_endpoints = self.ast_manipulation_endpoints.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let position = Position {
                    line: request_params["position"]["line"].as_u64()
                        .ok_or_else(|| "Missing position.line parameter".to_string())? as u32,
                    character: request_params["position"]["character"].as_u64()
                        .ok_or_else(|| "Missing position.character parameter".to_string())? as u32,
                };
                
                let transformation_type = match request_params["transformationType"].as_str() {
                    Some("rename") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::Rename,
                    Some("extractFunction") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::ExtractFunction,
                    Some("extractVariable") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::ExtractVariable,
                    Some("inlineFunction") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::InlineFunction,
                    Some("inlineVariable") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::InlineVariable,
                    Some("moveDeclaration") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::MoveDeclaration,
                    Some("changeSignature") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::ChangeSignature,
                    Some("convertToArrowFunction") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::ConvertToArrowFunction,
                    Some("convertToRegularFunction") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::ConvertToRegularFunction,
                    Some("addParameter") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::AddParameter,
                    Some("removeParameter") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::RemoveParameter,
                    Some("reorderParameters") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::ReorderParameters,
                    Some("addImport") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::AddImport,
                    Some("removeImport") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::RemoveImport,
                    Some("organizeImports") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::OrganizeImports,
                    Some("custom") => crate::language_hub_server::lsp::ast_manipulation::TransformationType::Custom,
                    _ => return Err("Invalid transformationType parameter".to_string()),
                };
                
                // Extract parameters
                let mut parameters = HashMap::new();
                if let Some(params) = request_params["parameters"].as_object() {
                    for (key, value) in params {
                        if let Some(value_str) = value.as_str() {
                            parameters.insert(key.clone(), value_str.to_string());
                        }
                    }
                }
                
                // Create the request
                let transformation_request = crate::language_hub_server::lsp::ast_manipulation::TransformationRequest {
                    document_uri,
                    position,
                    transformation_type,
                    options: None,
                    ast: None,
                    parse_result: None,
                    parameters,
                };
                
                // Apply the transformation
                let response = ast_manipulation_endpoints.apply_transformation(transformation_request)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "edit": response.edit,
                    "success": response.success,
                    "errorMessage": response.error_message,
                    "filesAffected": response.files_affected,
                    "editCount": response.edit_count
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "anarchy/astManipulation/executeQuery" => {
                let ast_manipulation_endpoints = self.ast_manipulation_endpoints.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let position = Position {
                    line: request_params["position"]["line"].as_u64()
                        .ok_or_else(|| "Missing position.line parameter".to_string())? as u32,
                    character: request_params["position"]["character"].as_u64()
                        .ok_or_else(|| "Missing position.character parameter".to_string())? as u32,
                };
                
                let query_type = match request_params["queryType"].as_str() {
                    Some("findReferences") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindReferences,
                    Some("findDefinition") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindDefinition,
                    Some("findImplementations") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindImplementations,
                    Some("findTypeDefinition") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindTypeDefinition,
                    Some("findSymbols") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindSymbols,
                    Some("findFunctions") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindFunctions,
                    Some("findVariables") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindVariables,
                    Some("findClasses") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindClasses,
                    Some("findImports") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindImports,
                    Some("findExports") => crate::language_hub_server::lsp::ast_manipulation::QueryType::FindExports,
                    Some("custom") => crate::language_hub_server::lsp::ast_manipulation::QueryType::Custom,
                    _ => return Err("Invalid queryType parameter".to_string()),
                };
                
                // Extract parameters
                let mut parameters = HashMap::new();
                if let Some(params) = request_params["parameters"].as_object() {
                    for (key, value) in params {
                        if let Some(value_str) = value.as_str() {
                            parameters.insert(key.clone(), value_str.to_string());
                        }
                    }
                }
                
                // Create the request
                let query_request = crate::language_hub_server::lsp::ast_manipulation::QueryRequest {
                    document_uri,
                    position,
                    query_type,
                    options: None,
                    ast: None,
                    parse_result: None,
                    parameters,
                };
                
                // Execute the query
                let response = ast_manipulation_endpoints.execute_query(query_request)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "results": response.results,
                    "success": response.success,
                    "errorMessage": response.error_message,
                    "resultCount": response.result_count
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            "anarchy/astManipulation/generateCode" => {
                let ast_manipulation_endpoints = self.ast_manipulation_endpoints.lock().unwrap();
                let request_params: serde_json::Value = request.params.clone();
                
                // Parse the request parameters
                let document_uri = request_params["documentUri"].as_str()
                    .ok_or_else(|| "Missing documentUri parameter".to_string())?
                    .to_string();
                
                let position = Position {
                    line: request_params["position"]["line"].as_u64()
                        .ok_or_else(|| "Missing position.line parameter".to_string())? as u32,
                    character: request_params["position"]["character"].as_u64()
                        .ok_or_else(|| "Missing position.character parameter".to_string())? as u32,
                };
                
                let generation_type = match request_params["generationType"].as_str() {
                    Some("function") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Function,
                    Some("class") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Class,
                    Some("interface") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Interface,
                    Some("enum") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Enum,
                    Some("module") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Module,
                    Some("test") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Test,
                    Some("documentation") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Documentation,
                    Some("custom") => crate::language_hub_server::lsp::ast_manipulation::GenerationType::Custom,
                    _ => return Err("Invalid generationType parameter".to_string()),
                };
                
                // Extract parameters
                let mut parameters = HashMap::new();
                if let Some(params) = request_params["parameters"].as_object() {
                    for (key, value) in params {
                        if let Some(value_str) = value.as_str() {
                            parameters.insert(key.clone(), value_str.to_string());
                        }
                    }
                }
                
                // Create the request
                let generation_request = crate::language_hub_server::lsp::ast_manipulation::GenerationRequest {
                    document_uri,
                    position,
                    generation_type,
                    options: None,
                    ast: None,
                    parse_result: None,
                    parameters,
                };
                
                // Generate the code
                let response = ast_manipulation_endpoints.generate_code(generation_request)?;
                
                // Convert to JSON
                let result = serde_json::json!({
                    "edit": response.edit,
                    "success": response.success,
                    "errorMessage": response.error_message,
                    "generatedCode": response.generated_code
                });
                
                serde_json::to_string(&result).map_err(|e| format!("Failed to serialize response: {}", e))?
            }
            
            // Unknown method
            _ => {
                return Err(format!("Unknown method: {}", request.method));
            }
        };
        
        // Create the response
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(serde_json::from_str(&result).unwrap_or(serde_json::Value::Null)),
            error: None,
        };
        
        // Serialize the response
        serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))
    }
    
    /// Handle a JSON-RPC notification
    fn handle_json_rpc_notification(&self, notification: &JsonRpcNotification) -> Result<(), String> {
        // Handle the notification based on the method
        match notification.method.as_str() {
            // LSP notifications
            "exit" => {
                let mut server = self.server.lock().unwrap();
                server.exit()?;
            }
            "textDocument/didOpen" => {
                let mut server = self.server.lock().unwrap();
                server.did_open(notification.params.clone())?;
            }
            "textDocument/didChange" => {
                let mut server = self.server.lock().unwrap();
                server.did_change(notification.params.clone())?;
            }
            "textDocument/didClose" => {
                let mut server = self.server.lock().unwrap();
                server.did_close(notification.params.clone())?;
            }
            "textDocument/didSave" => {
                let mut server = self.server.lock().unwrap();
                server.did_save(notification.params.clone())?;
            }
            
            // Unknown method
            _ => {
                return Err(format!("Unknown notification method: {}", notification.method));
            }
        }
        
        Ok(())
    }
    
    /// Get the server configuration
    pub fn get_config(&self) -> LanguageHubServerConfig {
        self.config.clone()
    }
    
    /// Set the server configuration
    pub fn set_config(&mut self, config: LanguageHubServerConfig) {
        self.config = config;
    }
}

/// Create a new Language Hub Server
pub fn create_language_hub_server(config: Option<LanguageHubServerConfig>) -> LanguageHubServer {
    LanguageHubServer::new(config)
}
