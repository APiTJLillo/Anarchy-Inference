// Structured completion endpoints module for LSP-like Component
//
// This module provides structured completion endpoints for Anarchy Inference code,
// offering AST-based suggestions through a standardized API.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, CompletionItem, CompletionList};
use crate::language_hub_server::lsp::document::{Document, DocumentManager, SharedDocumentManager};
use crate::language_hub_server::lsp::parser_integration::{AstNode, ParseResult};
use crate::language_hub_server::lsp::completion_provider::{CompletionProvider, SharedCompletionProvider};
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Completion context type
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContextType {
    /// Normal completion
    Normal,
    
    /// Member completion (after a dot)
    Member,
    
    /// Import completion
    Import,
    
    /// Parameter completion
    Parameter,
    
    /// Type completion
    Type,
    
    /// Snippet completion
    Snippet,
}

/// Completion context
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// The type of completion
    pub context_type: CompletionContextType,
    
    /// The trigger character
    pub trigger_character: Option<String>,
    
    /// The trigger kind (1 = invoked, 2 = trigger character, 3 = re-trigger)
    pub trigger_kind: u8,
    
    /// The position in the document
    pub position: Position,
    
    /// The document URI
    pub document_uri: String,
    
    /// The current line
    pub line: String,
    
    /// The prefix (text before cursor on the current line)
    pub prefix: String,
    
    /// The suffix (text after cursor on the current line)
    pub suffix: String,
    
    /// The word at the cursor
    pub word: String,
    
    /// The parent node in the AST
    pub parent_node: Option<AstNode>,
    
    /// The current node in the AST
    pub current_node: Option<AstNode>,
}

/// Structured completion request
#[derive(Debug, Clone)]
pub struct StructuredCompletionRequest {
    /// The document URI
    pub document_uri: String,
    
    /// The position in the document
    pub position: Position,
    
    /// The completion context
    pub context: Option<CompletionContext>,
    
    /// The AST of the document
    pub ast: Option<AstNode>,
    
    /// The parse result
    pub parse_result: Option<ParseResult>,
    
    /// Whether to include snippets
    pub include_snippets: bool,
    
    /// Whether to include keywords
    pub include_keywords: bool,
    
    /// Whether to include symbols
    pub include_symbols: bool,
    
    /// Whether to include members
    pub include_members: bool,
    
    /// Whether to include types
    pub include_types: bool,
    
    /// Maximum number of items to return
    pub max_items: usize,
}

impl Default for StructuredCompletionRequest {
    fn default() -> Self {
        StructuredCompletionRequest {
            document_uri: String::new(),
            position: Position { line: 0, character: 0 },
            context: None,
            ast: None,
            parse_result: None,
            include_snippets: true,
            include_keywords: true,
            include_symbols: true,
            include_members: true,
            include_types: true,
            max_items: 100,
        }
    }
}

/// Structured completion response
#[derive(Debug, Clone)]
pub struct StructuredCompletionResponse {
    /// The completion items
    pub items: Vec<CompletionItem>,
    
    /// Whether the list is incomplete
    pub is_incomplete: bool,
}

/// Structured completion endpoints
pub struct StructuredCompletionEndpoints {
    /// The document manager
    document_manager: SharedDocumentManager,
    
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// The completion provider
    completion_provider: SharedCompletionProvider,
}

impl StructuredCompletionEndpoints {
    /// Create new structured completion endpoints
    pub fn new(
        document_manager: SharedDocumentManager,
        symbol_manager: SharedSymbolManager,
        completion_provider: SharedCompletionProvider
    ) -> Self {
        StructuredCompletionEndpoints {
            document_manager,
            symbol_manager,
            completion_provider,
        }
    }
    
    /// Get completion items
    pub fn get_completion_items(
        &self,
        request: StructuredCompletionRequest
    ) -> Result<StructuredCompletionResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get or create the completion context
        let context = if let Some(ctx) = request.context {
            ctx
        } else {
            self.create_completion_context(&document, request.position, request.ast.as_ref())?
        };
        
        // Get completion items from the completion provider
        let mut completion_provider = self.completion_provider.lock().unwrap();
        let completion_list = completion_provider.provide_completion(&document, request.position, request.ast.as_ref())?;
        
        // Filter completion items based on request parameters
        let mut filtered_items = Vec::new();
        
        for item in completion_list.items {
            let should_include = match item.kind {
                // Snippets
                15 => request.include_snippets,
                
                // Keywords
                14 => request.include_keywords,
                
                // Types
                7 | 8 | 22 | 23 => request.include_types,
                
                // Members
                2 | 3 | 4 | 5 | 6 | 10 => request.include_members,
                
                // Symbols
                _ => request.include_symbols,
            };
            
            if should_include {
                filtered_items.push(item);
            }
        }
        
        // Limit the number of items
        if filtered_items.len() > request.max_items {
            filtered_items.truncate(request.max_items);
        }
        
        // Create the response
        let response = StructuredCompletionResponse {
            items: filtered_items,
            is_incomplete: completion_list.is_incomplete,
        };
        
        Ok(response)
    }
    
    /// Get AST-based completion suggestions
    pub fn get_ast_completion_suggestions(
        &self,
        request: StructuredCompletionRequest
    ) -> Result<StructuredCompletionResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get or create the AST
        let ast = if let Some(ast_node) = request.ast {
            ast_node
        } else {
            // Parse the document
            let parse_result = self.parse_document(&document)?;
            parse_result.ast
        };
        
        // Get the node at the position
        let node_at_position = self.find_node_at_position(&ast, request.position)?;
        
        // Get completion suggestions based on the AST
        let suggestions = self.get_suggestions_from_ast(&document, request.position, &node_at_position)?;
        
        // Create the response
        let response = StructuredCompletionResponse {
            items: suggestions,
            is_incomplete: false,
        };
        
        Ok(response)
    }
    
    /// Get context-aware completion
    pub fn get_context_aware_completion(
        &self,
        request: StructuredCompletionRequest
    ) -> Result<StructuredCompletionResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get or create the completion context
        let context = if let Some(ctx) = request.context {
            ctx
        } else {
            self.create_completion_context(&document, request.position, request.ast.as_ref())?
        };
        
        // Get completion items based on the context
        let suggestions = match context.context_type {
            CompletionContextType::Normal => self.get_normal_completion(&document, &context)?,
            CompletionContextType::Member => self.get_member_completion(&document, &context)?,
            CompletionContextType::Import => self.get_import_completion(&document, &context)?,
            CompletionContextType::Parameter => self.get_parameter_completion(&document, &context)?,
            CompletionContextType::Type => self.get_type_completion(&document, &context)?,
            CompletionContextType::Snippet => self.get_snippet_completion(&document, &context)?,
        };
        
        // Create the response
        let response = StructuredCompletionResponse {
            items: suggestions,
            is_incomplete: false,
        };
        
        Ok(response)
    }
    
    /// Get template-based completion
    pub fn get_template_based_completion(
        &self,
        request: StructuredCompletionRequest,
        template_name: &str
    ) -> Result<StructuredCompletionResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get completion items based on the template
        let suggestions = self.get_completion_from_template(&document, request.position, template_name)?;
        
        // Create the response
        let response = StructuredCompletionResponse {
            items: suggestions,
            is_incomplete: false,
        };
        
        Ok(response)
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
        // This is a simplified implementation
        // In a real implementation, we would use the parser to parse the document
        
        // For now, we'll just return a dummy parse result
        Ok(ParseResult {
            ast: AstNode {
                node_type: "Program".to_string(),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: document.line_count() as u32, character: 0 },
                },
                children: Vec::new(),
                properties: HashMap::new(),
            },
            errors: Vec::new(),
        })
    }
    
    /// Create completion context
    fn create_completion_context(
        &self,
        document: &Document,
        position: Position,
        ast: Option<&AstNode>
    ) -> Result<CompletionContext, String> {
        // Get the current line
        let line = document.get_line(position.line).unwrap_or_default();
        
        // Get the prefix and suffix
        let prefix = if position.character > 0 && position.character <= line.len() as u32 {
            line[0..position.character as usize].to_string()
        } else {
            String::new()
        };
        
        let suffix = if position.character < line.len() as u32 {
            line[position.character as usize..].to_string()
        } else {
            String::new()
        };
        
        // Get the word at the cursor
        let word = self.get_word_at_position(line.as_str(), position.character as usize);
        
        // Determine the context type
        let context_type = if prefix.trim_end().ends_with('.') {
            CompletionContextType::Member
        } else if prefix.trim_end().ends_with("import ") || prefix.trim_end().ends_with("from ") {
            CompletionContextType::Import
        } else if let Some(ast_node) = ast {
            if self.is_in_parameter_list(ast_node, position) {
                CompletionContextType::Parameter
            } else if self.is_in_type_annotation(ast_node, position) {
                CompletionContextType::Type
            } else {
                CompletionContextType::Normal
            }
        } else {
            CompletionContextType::Normal
        };
        
        // Get the trigger character
        let trigger_character = if prefix.ends_with('.') {
            Some(".".to_string())
        } else if prefix.ends_with('(') {
            Some("(".to_string())
        } else if prefix.ends_with('{') {
            Some("{".to_string())
        } else if prefix.ends_with('[') {
            Some("[".to_string())
        } else {
            None
        };
        
        // Get the parent and current nodes
        let (parent_node, current_node) = if let Some(ast_node) = ast {
            self.find_nodes_at_position(ast_node, position)?
        } else {
            (None, None)
        };
        
        // Create the completion context
        let context = CompletionContext {
            context_type,
            trigger_character,
            trigger_kind: if trigger_character.is_some() { 2 } else { 1 },
            position,
            document_uri: document.uri.clone(),
            line,
            prefix,
            suffix,
            word,
            parent_node,
            current_node,
        };
        
        Ok(context)
    }
    
    /// Get word at position
    fn get_word_at_position(&self, line: &str, position: usize) -> String {
        if position > line.len() {
            return String::new();
        }
        
        // Find the start of the word
        let mut start = position;
        while start > 0 {
            let prev_char = line.chars().nth(start - 1).unwrap_or(' ');
            if !prev_char.is_alphanumeric() && prev_char != '_' {
                break;
            }
            start -= 1;
        }
        
        // Find the end of the word
        let mut end = position;
        while end < line.len() {
            let next_char = line.chars().nth(end).unwrap_or(' ');
            if !next_char.is_alphanumeric() && next_char != '_' {
                break;
            }
            end += 1;
        }
        
        // Extract the word
        if start < end {
            line[start..end].to_string()
        } else {
            String::new()
        }
    }
    
    /// Check if position is in parameter list
    fn is_in_parameter_list(&self, ast: &AstNode, position: Position) -> bool {
        // Find all parameter lists in the AST
        let parameter_lists = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "ParameterList" ||
            node.node_type == "FormalParameterList"
        });
        
        // Check if the position is in any parameter list
        for param_list in parameter_lists {
            if position.line >= param_list.range.start.line &&
               position.line <= param_list.range.end.line &&
               (position.line > param_list.range.start.line ||
                position.character >= param_list.range.start.character) &&
               (position.line < param_list.range.end.line ||
                position.character <= param_list.range.end.character) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if position is in type annotation
    fn is_in_type_annotation(&self, ast: &AstNode, position: Position) -> bool {
        // Find all type annotations in the AST
        let type_annotations = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "TypeAnnotation" ||
            node.node_type == "TypeReference"
        });
        
        // Check if the position is in any type annotation
        for type_annotation in type_annotations {
            if position.line >= type_annotation.range.start.line &&
               position.line <= type_annotation.range.end.line &&
               (position.line > type_annotation.range.start.line ||
                position.character >= type_annotation.range.start.character) &&
               (position.line < type_annotation.range.end.line ||
                position.character <= type_annotation.range.end.character) {
                return true;
            }
        }
        
        false
    }
    
    /// Find node at position
    fn find_node_at_position(&self, ast: &AstNode, position: Position) -> Result<AstNode, String> {
        // Find the innermost node that contains the position
        let nodes = AstUtils::collect_nodes(ast, |node| {
            position.line >= node.range.start.line &&
            position.line <= node.range.end.line &&
            (position.line > node.range.start.line ||
             position.character >= node.range.start.character) &&
            (position.line < node.range.end.line ||
             position.character <= node.range.end.character)
        });
        
        if nodes.is_empty() {
            return Err(format!("No node found at position {:?}", position));
        }
        
        // Find the innermost node
        let mut innermost = &nodes[0];
        
        for node in &nodes {
            if node.range.start.line >= innermost.range.start.line &&
               node.range.end.line <= innermost.range.end.line &&
               (node.range.start.line > innermost.range.start.line ||
                node.range.start.character >= innermost.range.start.character) &&
               (node.range.end.line < innermost.range.end.line ||
                node.range.end.character <= innermost.range.end.character) {
                innermost = node;
            }
        }
        
        Ok(innermost.clone())
    }
    
    /// Find parent and current nodes at position
    fn find_nodes_at_position(
        &self,
        ast: &AstNode,
        position: Position
    ) -> Result<(Option<AstNode>, Option<AstNode>), String> {
        // Find all nodes that contain the position
        let nodes = AstUtils::collect_nodes(ast, |node| {
            position.line >= node.range.start.line &&
            position.line <= node.range.end.line &&
            (position.line > node.range.start.line ||
             position.character >= node.range.start.character) &&
            (position.line < node.range.end.line ||
             position.character <= node.range.end.character)
        });
        
        if nodes.is_empty() {
            return Ok((None, None));
        }
        
        // Sort nodes by size (smallest to largest)
        let mut sorted_nodes = nodes;
        sorted_nodes.sort_by(|a, b| {
            let a_size = (a.range.end.line - a.range.start.line) * 1000 +
                         (a.range.end.character - a.range.start.character);
            let b_size = (b.range.end.line - b.range.start.line) * 1000 +
                         (b.range.end.character - b.range.start.character);
            a_size.cmp(&b_size)
        });
        
        // The smallest node is the current node
        let current_node = sorted_nodes.first().cloned();
        
        // The second smallest node is the parent node
        let parent_node = if sorted_nodes.len() > 1 {
            Some(sorted_nodes[1].clone())
        } else {
            None
        };
        
        Ok((parent_node, current_node))
    }
    
    /// Get suggestions from AST
    fn get_suggestions_from_ast(
        &self,
        document: &Document,
        position: Position,
        node: &AstNode
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would analyze the AST to provide context-aware suggestions
        
        // For now, we'll just return some basic suggestions
        let mut items = Vec::new();
        
        // Add some basic suggestions based on node type
        match node.node_type.as_str() {
            "Program" => {
                // Suggest top-level declarations
                items.push(CompletionItem {
                    label: "function".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Define a function".to_string()),
                    documentation: Some("function name() {\n  // code\n}".to_string()),
                    insert_text: Some("function ${1:name}() {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "class".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Define a class".to_string()),
                    documentation: Some("class Name {\n  constructor() {\n    // code\n  }\n}".to_string()),
                    insert_text: Some("class ${1:Name} {\n  constructor() {\n    ${0}\n  }\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "import".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Import a module".to_string()),
                    documentation: Some("import { name } from 'module';".to_string()),
                    insert_text: Some("import { ${1:name} } from '${2:module}';".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            "BlockStatement" => {
                // Suggest statements
                items.push(CompletionItem {
                    label: "if".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("If statement".to_string()),
                    documentation: Some("if (condition) {\n  // code\n}".to_string()),
                    insert_text: Some("if (${1:condition}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "for".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("For loop".to_string()),
                    documentation: Some("for (let i = 0; i < n; i++) {\n  // code\n}".to_string()),
                    insert_text: Some("for (let ${1:i} = 0; ${1:i} < ${2:n}; ${1:i}++) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "while".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("While loop".to_string()),
                    documentation: Some("while (condition) {\n  // code\n}".to_string()),
                    insert_text: Some("while (${1:condition}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "let".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Variable declaration".to_string()),
                    documentation: Some("let name = value;".to_string()),
                    insert_text: Some("let ${1:name} = ${2:value};".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "return".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Return statement".to_string()),
                    documentation: Some("return value;".to_string()),
                    insert_text: Some("return ${1:value};".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            "FunctionDeclaration" | "MethodDefinition" => {
                // Suggest function-related items
                items.push(CompletionItem {
                    label: "return".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Return statement".to_string()),
                    documentation: Some("return value;".to_string()),
                    insert_text: Some("return ${1:value};".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "throw".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Throw an error".to_string()),
                    documentation: Some("throw new Error('message');".to_string()),
                    insert_text: Some("throw new Error('${1:message}');".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            "IfStatement" => {
                // Suggest if-related items
                items.push(CompletionItem {
                    label: "else".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Else clause".to_string()),
                    documentation: Some("else {\n  // code\n}".to_string()),
                    insert_text: Some("else {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "else if".to_string(),
                    kind: Some(14), // Keyword
                    detail: Some("Else if clause".to_string()),
                    documentation: Some("else if (condition) {\n  // code\n}".to_string()),
                    insert_text: Some("else if (${1:condition}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            _ => {
                // Add some generic suggestions
                items.push(CompletionItem {
                    label: "console.log".to_string(),
                    kind: Some(1), // Text
                    detail: Some("Log to console".to_string()),
                    documentation: Some("console.log(message);".to_string()),
                    insert_text: Some("console.log(${1:message});".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            }
        }
        
        Ok(items)
    }
    
    /// Get normal completion
    fn get_normal_completion(
        &self,
        document: &Document,
        context: &CompletionContext
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would provide context-aware suggestions
        
        // For now, we'll just return some basic suggestions
        let mut items = Vec::new();
        
        // Add keywords
        let keywords = [
            "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete",
            "do", "else", "export", "extends", "finally", "for", "function", "if", "import", "in",
            "instanceof", "new", "return", "super", "switch", "this", "throw", "try", "typeof",
            "var", "void", "while", "with", "yield", "let", "static", "enum", "await", "implements",
            "interface", "package", "private", "protected", "public"
        ];
        
        for keyword in keywords {
            if keyword.starts_with(&context.word) {
                items.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(14), // Keyword
                    ..Default::default()
                });
            }
        }
        
        // Add snippets
        items.push(CompletionItem {
            label: "if".to_string(),
            kind: Some(15), // Snippet
            detail: Some("If statement".to_string()),
            documentation: Some("if (condition) {\n  // code\n}".to_string()),
            insert_text: Some("if (${1:condition}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "for".to_string(),
            kind: Some(15), // Snippet
            detail: Some("For loop".to_string()),
            documentation: Some("for (let i = 0; i < n; i++) {\n  // code\n}".to_string()),
            insert_text: Some("for (let ${1:i} = 0; ${1:i} < ${2:n}; ${1:i}++) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "function".to_string(),
            kind: Some(15), // Snippet
            detail: Some("Function declaration".to_string()),
            documentation: Some("function name(params) {\n  // code\n}".to_string()),
            insert_text: Some("function ${1:name}(${2:params}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        Ok(items)
    }
    
    /// Get member completion
    fn get_member_completion(
        &self,
        document: &Document,
        context: &CompletionContext
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would analyze the object type and provide its members
        
        // For now, we'll just return some common methods
        let mut items = Vec::new();
        
        // Add some common methods
        items.push(CompletionItem {
            label: "toString".to_string(),
            kind: Some(2), // Method
            detail: Some("Convert to string".to_string()),
            documentation: Some("Returns a string representation of the object.".to_string()),
            insert_text: Some("toString()".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "valueOf".to_string(),
            kind: Some(2), // Method
            detail: Some("Get primitive value".to_string()),
            documentation: Some("Returns the primitive value of the object.".to_string()),
            insert_text: Some("valueOf()".to_string()),
            ..Default::default()
        });
        
        // Add array methods if the object might be an array
        items.push(CompletionItem {
            label: "length".to_string(),
            kind: Some(10), // Property
            detail: Some("Array length".to_string()),
            documentation: Some("The number of elements in the array.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "push".to_string(),
            kind: Some(2), // Method
            detail: Some("Add elements".to_string()),
            documentation: Some("Adds one or more elements to the end of an array.".to_string()),
            insert_text: Some("push(${1:element})".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "pop".to_string(),
            kind: Some(2), // Method
            detail: Some("Remove last element".to_string()),
            documentation: Some("Removes the last element from an array.".to_string()),
            insert_text: Some("pop()".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "map".to_string(),
            kind: Some(2), // Method
            detail: Some("Map elements".to_string()),
            documentation: Some("Creates a new array with the results of calling a function on every element.".to_string()),
            insert_text: Some("map(${1:callback})".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "filter".to_string(),
            kind: Some(2), // Method
            detail: Some("Filter elements".to_string()),
            documentation: Some("Creates a new array with all elements that pass the test.".to_string()),
            insert_text: Some("filter(${1:callback})".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        Ok(items)
    }
    
    /// Get import completion
    fn get_import_completion(
        &self,
        document: &Document,
        context: &CompletionContext
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would provide available modules and exports
        
        // For now, we'll just return some common modules
        let mut items = Vec::new();
        
        // Add some common modules
        items.push(CompletionItem {
            label: "fs".to_string(),
            kind: Some(9), // Module
            detail: Some("File system module".to_string()),
            documentation: Some("Provides file system-related functionality.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "path".to_string(),
            kind: Some(9), // Module
            detail: Some("Path module".to_string()),
            documentation: Some("Provides utilities for working with file and directory paths.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "http".to_string(),
            kind: Some(9), // Module
            detail: Some("HTTP module".to_string()),
            documentation: Some("Provides HTTP server and client functionality.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "util".to_string(),
            kind: Some(9), // Module
            detail: Some("Utility module".to_string()),
            documentation: Some("Provides utility functions.".to_string()),
            ..Default::default()
        });
        
        Ok(items)
    }
    
    /// Get parameter completion
    fn get_parameter_completion(
        &self,
        document: &Document,
        context: &CompletionContext
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would analyze the function signature and provide parameter suggestions
        
        // For now, we'll just return some generic parameter suggestions
        let mut items = Vec::new();
        
        // Add some generic parameter suggestions
        items.push(CompletionItem {
            label: "options".to_string(),
            kind: Some(6), // Variable
            detail: Some("Options object".to_string()),
            documentation: Some("An object containing various options.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "callback".to_string(),
            kind: Some(6), // Variable
            detail: Some("Callback function".to_string()),
            documentation: Some("A function to be called when the operation completes.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "data".to_string(),
            kind: Some(6), // Variable
            detail: Some("Data parameter".to_string()),
            documentation: Some("The data to be processed.".to_string()),
            ..Default::default()
        });
        
        Ok(items)
    }
    
    /// Get type completion
    fn get_type_completion(
        &self,
        document: &Document,
        context: &CompletionContext
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would provide available types
        
        // For now, we'll just return some common types
        let mut items = Vec::new();
        
        // Add some common types
        items.push(CompletionItem {
            label: "string".to_string(),
            kind: Some(7), // Class
            detail: Some("String type".to_string()),
            documentation: Some("A sequence of characters.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "number".to_string(),
            kind: Some(7), // Class
            detail: Some("Number type".to_string()),
            documentation: Some("A numeric value.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "boolean".to_string(),
            kind: Some(7), // Class
            detail: Some("Boolean type".to_string()),
            documentation: Some("A true or false value.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "object".to_string(),
            kind: Some(7), // Class
            detail: Some("Object type".to_string()),
            documentation: Some("A collection of properties.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "array".to_string(),
            kind: Some(7), // Class
            detail: Some("Array type".to_string()),
            documentation: Some("An ordered collection of values.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "function".to_string(),
            kind: Some(7), // Class
            detail: Some("Function type".to_string()),
            documentation: Some("A callable object.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "any".to_string(),
            kind: Some(7), // Class
            detail: Some("Any type".to_string()),
            documentation: Some("Any type of value.".to_string()),
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "void".to_string(),
            kind: Some(7), // Class
            detail: Some("Void type".to_string()),
            documentation: Some("No type (used for functions that don't return a value).".to_string()),
            ..Default::default()
        });
        
        Ok(items)
    }
    
    /// Get snippet completion
    fn get_snippet_completion(
        &self,
        document: &Document,
        context: &CompletionContext
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would provide various code snippets
        
        // For now, we'll just return some common snippets
        let mut items = Vec::new();
        
        // Add some common snippets
        items.push(CompletionItem {
            label: "if".to_string(),
            kind: Some(15), // Snippet
            detail: Some("If statement".to_string()),
            documentation: Some("if (condition) {\n  // code\n}".to_string()),
            insert_text: Some("if (${1:condition}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "ifelse".to_string(),
            kind: Some(15), // Snippet
            detail: Some("If-else statement".to_string()),
            documentation: Some("if (condition) {\n  // code\n} else {\n  // code\n}".to_string()),
            insert_text: Some("if (${1:condition}) {\n  ${2}\n} else {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "for".to_string(),
            kind: Some(15), // Snippet
            detail: Some("For loop".to_string()),
            documentation: Some("for (let i = 0; i < n; i++) {\n  // code\n}".to_string()),
            insert_text: Some("for (let ${1:i} = 0; ${1:i} < ${2:n}; ${1:i}++) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "forin".to_string(),
            kind: Some(15), // Snippet
            detail: Some("For-in loop".to_string()),
            documentation: Some("for (const key in object) {\n  // code\n}".to_string()),
            insert_text: Some("for (const ${1:key} in ${2:object}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "forof".to_string(),
            kind: Some(15), // Snippet
            detail: Some("For-of loop".to_string()),
            documentation: Some("for (const item of items) {\n  // code\n}".to_string()),
            insert_text: Some("for (const ${1:item} of ${2:items}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "while".to_string(),
            kind: Some(15), // Snippet
            detail: Some("While loop".to_string()),
            documentation: Some("while (condition) {\n  // code\n}".to_string()),
            insert_text: Some("while (${1:condition}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "function".to_string(),
            kind: Some(15), // Snippet
            detail: Some("Function declaration".to_string()),
            documentation: Some("function name(params) {\n  // code\n}".to_string()),
            insert_text: Some("function ${1:name}(${2:params}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "arrow".to_string(),
            kind: Some(15), // Snippet
            detail: Some("Arrow function".to_string()),
            documentation: Some("(params) => {\n  // code\n}".to_string()),
            insert_text: Some("(${1:params}) => {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "class".to_string(),
            kind: Some(15), // Snippet
            detail: Some("Class declaration".to_string()),
            documentation: Some("class Name {\n  constructor(params) {\n    // code\n  }\n}".to_string()),
            insert_text: Some("class ${1:Name} {\n  constructor(${2:params}) {\n    ${0}\n  }\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        items.push(CompletionItem {
            label: "try".to_string(),
            kind: Some(15), // Snippet
            detail: Some("Try-catch block".to_string()),
            documentation: Some("try {\n  // code\n} catch (error) {\n  // code\n}".to_string()),
            insert_text: Some("try {\n  ${1}\n} catch (${2:error}) {\n  ${0}\n}".to_string()),
            insert_text_format: Some(2), // Snippet
            ..Default::default()
        });
        
        Ok(items)
    }
    
    /// Get completion from template
    fn get_completion_from_template(
        &self,
        document: &Document,
        position: Position,
        template_name: &str
    ) -> Result<Vec<CompletionItem>, String> {
        // This is a simplified implementation
        // In a real implementation, we would provide template-specific suggestions
        
        // For now, we'll just return some template-specific snippets
        let mut items = Vec::new();
        
        match template_name {
            "function" => {
                items.push(CompletionItem {
                    label: "function".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Function declaration".to_string()),
                    documentation: Some("function name(params) {\n  // code\n}".to_string()),
                    insert_text: Some("function ${1:name}(${2:params}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "arrow".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Arrow function".to_string()),
                    documentation: Some("(params) => {\n  // code\n}".to_string()),
                    insert_text: Some("(${1:params}) => {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "async".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Async function".to_string()),
                    documentation: Some("async function name(params) {\n  // code\n}".to_string()),
                    insert_text: Some("async function ${1:name}(${2:params}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "asyncarrow".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Async arrow function".to_string()),
                    documentation: Some("async (params) => {\n  // code\n}".to_string()),
                    insert_text: Some("async (${1:params}) => {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            "class" => {
                items.push(CompletionItem {
                    label: "class".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Class declaration".to_string()),
                    documentation: Some("class Name {\n  constructor(params) {\n    // code\n  }\n}".to_string()),
                    insert_text: Some("class ${1:Name} {\n  constructor(${2:params}) {\n    ${0}\n  }\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "method".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Class method".to_string()),
                    documentation: Some("methodName(params) {\n  // code\n}".to_string()),
                    insert_text: Some("${1:methodName}(${2:params}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "getter".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Class getter".to_string()),
                    documentation: Some("get propertyName() {\n  // code\n}".to_string()),
                    insert_text: Some("get ${1:propertyName}() {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "setter".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Class setter".to_string()),
                    documentation: Some("set propertyName(value) {\n  // code\n}".to_string()),
                    insert_text: Some("set ${1:propertyName}(${2:value}) {\n  ${0}\n}".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            "import" => {
                items.push(CompletionItem {
                    label: "import".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Import statement".to_string()),
                    documentation: Some("import { name } from 'module';".to_string()),
                    insert_text: Some("import { ${1:name} } from '${2:module}';".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "importdefault".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Import default".to_string()),
                    documentation: Some("import name from 'module';".to_string()),
                    insert_text: Some("import ${1:name} from '${2:module}';".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
                
                items.push(CompletionItem {
                    label: "importall".to_string(),
                    kind: Some(15), // Snippet
                    detail: Some("Import all".to_string()),
                    documentation: Some("import * as name from 'module';".to_string()),
                    insert_text: Some("import * as ${1:name} from '${2:module}';".to_string()),
                    insert_text_format: Some(2), // Snippet
                    ..Default::default()
                });
            },
            _ => {
                // Return empty list for unknown templates
            }
        }
        
        Ok(items)
    }
}

/// Shared structured completion endpoints that can be used across threads
pub type SharedStructuredCompletionEndpoints = Arc<Mutex<StructuredCompletionEndpoints>>;

/// Create a new shared structured completion endpoints
pub fn create_shared_structured_completion_endpoints(
    document_manager: SharedDocumentManager,
    symbol_manager: SharedSymbolManager,
    completion_provider: SharedCompletionProvider
) -> SharedStructuredCompletionEndpoints {
    Arc::new(Mutex::new(StructuredCompletionEndpoints::new(
        document_manager,
        symbol_manager,
        completion_provider
    )))
}
