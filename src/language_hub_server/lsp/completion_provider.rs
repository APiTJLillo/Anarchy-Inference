// Code completion provider module for LSP-like Component
//
// This module provides code completion functionality for Anarchy Inference code,
// including context-aware suggestions for identifiers, keywords, and snippets.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, CompletionItem, CompletionItemKind};
use crate::language_hub_server::lsp::document::Document;
use crate::language_hub_server::lsp::parser_integration::AstNode;
use crate::language_hub_server::lsp::semantic_analyzer::{SemanticAnalyzer, SharedSemanticAnalyzer};
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager, SymbolInformation};
use crate::language_hub_server::lsp::type_checker::{TypeChecker, SharedTypeChecker, TypeInfo};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Completion context information
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// The position where completion was requested
    pub position: Position,
    
    /// The trigger character (if any)
    pub trigger_character: Option<String>,
    
    /// The trigger kind
    pub trigger_kind: CompletionTriggerKind,
}

/// Completion trigger kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionTriggerKind {
    /// Completion was triggered by typing an identifier
    Invoked = 1,
    
    /// Completion was triggered by a trigger character
    TriggerCharacter = 2,
    
    /// Completion was re-triggered as the current completion list is incomplete
    TriggerForIncompleteCompletions = 3,
}

/// Completion provider for Anarchy Inference code
pub struct CompletionProvider {
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// The semantic analyzer
    semantic_analyzer: SharedSemanticAnalyzer,
    
    /// The type checker
    type_checker: SharedTypeChecker,
    
    /// Anarchy Inference keywords
    keywords: Vec<String>,
    
    /// Anarchy Inference snippets
    snippets: HashMap<String, String>,
}

impl CompletionProvider {
    /// Create a new completion provider
    pub fn new(
        symbol_manager: SharedSymbolManager,
        semantic_analyzer: SharedSemanticAnalyzer,
        type_checker: SharedTypeChecker
    ) -> Self {
        // Initialize keywords
        let keywords = vec![
            "if".to_string(),
            "else".to_string(),
            "while".to_string(),
            "for".to_string(),
            "return".to_string(),
            "break".to_string(),
            "continue".to_string(),
            "function".to_string(),
            "var".to_string(),
            "let".to_string(),
            "const".to_string(),
            "true".to_string(),
            "false".to_string(),
            "null".to_string(),
            "undefined".to_string(),
            "import".to_string(),
            "export".to_string(),
            "from".to_string(),
            "as".to_string(),
            "module".to_string(),
            "class".to_string(),
            "extends".to_string(),
            "implements".to_string(),
            "interface".to_string(),
            "type".to_string(),
            "enum".to_string(),
            "public".to_string(),
            "private".to_string(),
            "protected".to_string(),
            "static".to_string(),
            "async".to_string(),
            "await".to_string(),
            "try".to_string(),
            "catch".to_string(),
            "finally".to_string(),
            "throw".to_string(),
        ];
        
        // Initialize snippets
        let mut snippets = HashMap::new();
        
        // Function declaration snippet
        snippets.insert(
            "function".to_string(),
            "function ${1:name}(${2:params}) {\n\t${0}\n}".to_string()
        );
        
        // If statement snippet
        snippets.insert(
            "if".to_string(),
            "if (${1:condition}) {\n\t${0}\n}".to_string()
        );
        
        // If-else statement snippet
        snippets.insert(
            "ifelse".to_string(),
            "if (${1:condition}) {\n\t${2}\n} else {\n\t${0}\n}".to_string()
        );
        
        // For loop snippet
        snippets.insert(
            "for".to_string(),
            "for (let ${1:i} = 0; ${1:i} < ${2:count}; ${1:i}++) {\n\t${0}\n}".to_string()
        );
        
        // While loop snippet
        snippets.insert(
            "while".to_string(),
            "while (${1:condition}) {\n\t${0}\n}".to_string()
        );
        
        // Module declaration snippet
        snippets.insert(
            "module".to_string(),
            "module ${1:name} {\n\t${0}\n}".to_string()
        );
        
        // Try-catch snippet
        snippets.insert(
            "try".to_string(),
            "try {\n\t${1}\n} catch (${2:error}) {\n\t${0}\n}".to_string()
        );
        
        // Class declaration snippet
        snippets.insert(
            "class".to_string(),
            "class ${1:Name} {\n\tconstructor(${2:params}) {\n\t\t${0}\n\t}\n}".to_string()
        );
        
        CompletionProvider {
            symbol_manager,
            semantic_analyzer,
            type_checker,
            keywords,
            snippets,
        }
    }
    
    /// Provide completion items for a document at a specific position
    pub fn provide_completion(
        &self,
        document: &Document,
        position: Position,
        context: Option<CompletionContext>,
        ast: &AstNode
    ) -> Result<Vec<CompletionItem>, String> {
        // Get the current line up to the cursor position
        let line = document.get_line(position.line).unwrap_or_default();
        let line_prefix = if position.character > 0 {
            &line[..position.character as usize]
        } else {
            ""
        };
        
        // Determine the completion context
        let is_member_completion = line_prefix.ends_with(".");
        let is_import_completion = line_prefix.contains("import") || line_prefix.contains("from");
        
        // Get the current scope
        let scope = AstUtils::find_scope_at_position(ast, position);
        
        // Collect completion items
        let mut items = Vec::new();
        
        if is_member_completion {
            // Member completion (after a dot)
            self.provide_member_completion(document, position, line_prefix, ast, &mut items)?;
        } else if is_import_completion {
            // Import completion
            self.provide_import_completion(document, position, line_prefix, &mut items)?;
        } else {
            // Regular completion
            
            // Add keywords
            self.provide_keyword_completion(line_prefix, &mut items);
            
            // Add snippets
            self.provide_snippet_completion(line_prefix, &mut items);
            
            // Add local symbols
            self.provide_local_symbol_completion(document, position, scope.as_ref(), &mut items)?;
            
            // Add global symbols
            self.provide_global_symbol_completion(document, position, &mut items)?;
            
            // Add module symbols
            self.provide_module_symbol_completion(document, position, &mut items)?;
        }
        
        Ok(items)
    }
    
    /// Provide keyword completion
    fn provide_keyword_completion(&self, line_prefix: &str, items: &mut Vec<CompletionItem>) {
        // Get the current word being typed
        let word = self.get_current_word(line_prefix);
        
        // Filter keywords that match the current word
        for keyword in &self.keywords {
            if keyword.starts_with(&word) {
                items.push(CompletionItem {
                    label: keyword.clone(),
                    kind: CompletionItemKind::Keyword,
                    detail: Some("Anarchy Inference keyword".to_string()),
                    documentation: None,
                    deprecated: false,
                    preselect: false,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(keyword.clone()),
                    insert_text_format: None,
                    text_edit: None,
                    additional_text_edits: Vec::new(),
                    command: None,
                    data: None,
                });
            }
        }
    }
    
    /// Provide snippet completion
    fn provide_snippet_completion(&self, line_prefix: &str, items: &mut Vec<CompletionItem>) {
        // Get the current word being typed
        let word = self.get_current_word(line_prefix);
        
        // Filter snippets that match the current word
        for (label, snippet) in &self.snippets {
            if label.starts_with(&word) {
                items.push(CompletionItem {
                    label: label.clone(),
                    kind: CompletionItemKind::Snippet,
                    detail: Some("Anarchy Inference snippet".to_string()),
                    documentation: None,
                    deprecated: false,
                    preselect: false,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(snippet.clone()),
                    insert_text_format: Some(2), // Snippet format
                    text_edit: None,
                    additional_text_edits: Vec::new(),
                    command: None,
                    data: None,
                });
            }
        }
    }
    
    /// Provide local symbol completion
    fn provide_local_symbol_completion(
        &self,
        document: &Document,
        position: Position,
        scope: Option<&AstNode>,
        items: &mut Vec<CompletionItem>
    ) -> Result<(), String> {
        // Get the current word being typed
        let line = document.get_line(position.line).unwrap_or_default();
        let line_prefix = if position.character > 0 {
            &line[..position.character as usize]
        } else {
            ""
        };
        let word = self.get_current_word(line_prefix);
        
        // Get local symbols from the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        let local_symbols = symbol_manager.get_symbols_in_scope(&document.uri, position);
        
        // Add local symbols to completion items
        for symbol in local_symbols {
            if symbol.name.starts_with(&word) {
                // Get the symbol type
                let type_info = if let Some(type_str) = &symbol.symbol_type {
                    type_str.clone()
                } else {
                    "unknown".to_string()
                };
                
                // Determine the symbol kind
                let kind = match symbol.kind.as_str() {
                    "function" => CompletionItemKind::Function,
                    "variable" => CompletionItemKind::Variable,
                    "parameter" => CompletionItemKind::Variable,
                    "class" => CompletionItemKind::Class,
                    "interface" => CompletionItemKind::Interface,
                    "module" => CompletionItemKind::Module,
                    "property" => CompletionItemKind::Property,
                    "method" => CompletionItemKind::Method,
                    "enum" => CompletionItemKind::Enum,
                    "constant" => CompletionItemKind::Constant,
                    _ => CompletionItemKind::Text,
                };
                
                items.push(CompletionItem {
                    label: symbol.name.clone(),
                    kind,
                    detail: Some(format!("{}: {}", symbol.kind, type_info)),
                    documentation: symbol.documentation.clone(),
                    deprecated: false,
                    preselect: false,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(symbol.name.clone()),
                    insert_text_format: None,
                    text_edit: None,
                    additional_text_edits: Vec::new(),
                    command: None,
                    data: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Provide global symbol completion
    fn provide_global_symbol_completion(
        &self,
        document: &Document,
        position: Position,
        items: &mut Vec<CompletionItem>
    ) -> Result<(), String> {
        // Get the current word being typed
        let line = document.get_line(position.line).unwrap_or_default();
        let line_prefix = if position.character > 0 {
            &line[..position.character as usize]
        } else {
            ""
        };
        let word = self.get_current_word(line_prefix);
        
        // Get global symbols from the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        let global_symbols = symbol_manager.get_global_symbols();
        
        // Add global symbols to completion items
        for symbol in global_symbols {
            if symbol.name.starts_with(&word) {
                // Get the symbol type
                let type_info = if let Some(type_str) = &symbol.symbol_type {
                    type_str.clone()
                } else {
                    "unknown".to_string()
                };
                
                // Determine the symbol kind
                let kind = match symbol.kind.as_str() {
                    "function" => CompletionItemKind::Function,
                    "variable" => CompletionItemKind::Variable,
                    "class" => CompletionItemKind::Class,
                    "interface" => CompletionItemKind::Interface,
                    "module" => CompletionItemKind::Module,
                    "property" => CompletionItemKind::Property,
                    "method" => CompletionItemKind::Method,
                    "enum" => CompletionItemKind::Enum,
                    "constant" => CompletionItemKind::Constant,
                    _ => CompletionItemKind::Text,
                };
                
                items.push(CompletionItem {
                    label: symbol.name.clone(),
                    kind,
                    detail: Some(format!("{}: {}", symbol.kind, type_info)),
                    documentation: symbol.documentation.clone(),
                    deprecated: false,
                    preselect: false,
                    sort_text: Some(format!("2-{}", symbol.name)), // Sort after local symbols
                    filter_text: None,
                    insert_text: Some(symbol.name.clone()),
                    insert_text_format: None,
                    text_edit: None,
                    additional_text_edits: Vec::new(),
                    command: None,
                    data: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Provide module symbol completion
    fn provide_module_symbol_completion(
        &self,
        document: &Document,
        position: Position,
        items: &mut Vec<CompletionItem>
    ) -> Result<(), String> {
        // Get the current word being typed
        let line = document.get_line(position.line).unwrap_or_default();
        let line_prefix = if position.character > 0 {
            &line[..position.character as usize]
        } else {
            ""
        };
        let word = self.get_current_word(line_prefix);
        
        // Get module symbols from the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        let module_symbols = symbol_manager.get_module_symbols();
        
        // Add module symbols to completion items
        for symbol in module_symbols {
            if symbol.name.starts_with(&word) {
                items.push(CompletionItem {
                    label: symbol.name.clone(),
                    kind: CompletionItemKind::Module,
                    detail: Some("module".to_string()),
                    documentation: symbol.documentation.clone(),
                    deprecated: false,
                    preselect: false,
                    sort_text: Some(format!("3-{}", symbol.name)), // Sort after global symbols
                    filter_text: None,
                    insert_text: Some(symbol.name.clone()),
                    insert_text_format: None,
                    text_edit: None,
                    additional_text_edits: Vec::new(),
                    command: None,
                    data: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Provide member completion (after a dot)
    fn provide_member_completion(
        &self,
        document: &Document,
        position: Position,
        line_prefix: &str,
        ast: &AstNode,
        items: &mut Vec<CompletionItem>
    ) -> Result<(), String> {
        // Find the object before the dot
        let object_name = self.get_object_before_dot(line_prefix);
        if object_name.is_empty() {
            return Ok(());
        }
        
        // Get the object type from the type checker
        let type_checker = self.type_checker.lock().unwrap();
        let object_type = type_checker.get_symbol_type(&document.uri, &object_name, position);
        
        match object_type {
            TypeInfo::Object(props) => {
                // Add object properties to completion items
                for (name, prop_type) in props {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionItemKind::Property,
                        detail: Some(format!("property: {}", prop_type.to_string())),
                        documentation: None,
                        deprecated: false,
                        preselect: false,
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(name.clone()),
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: Vec::new(),
                        command: None,
                        data: None,
                    });
                }
            }
            
            TypeInfo::Module(exports) => {
                // Add module exports to completion items
                for (name, export_type) in exports {
                    // Determine the export kind
                    let kind = match export_type {
                        TypeInfo::Function { .. } => CompletionItemKind::Function,
                        TypeInfo::Object(_) => CompletionItemKind::Class,
                        TypeInfo::Module(_) => CompletionItemKind::Module,
                        _ => CompletionItemKind::Value,
                    };
                    
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind,
                        detail: Some(format!("export: {}", export_type.to_string())),
                        documentation: None,
                        deprecated: false,
                        preselect: false,
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(name.clone()),
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: Vec::new(),
                        command: None,
                        data: None,
                    });
                }
            }
            
            TypeInfo::Array(_) => {
                // Add array properties and methods
                let array_members = vec![
                    ("length", "property: number", CompletionItemKind::Property),
                    ("push", "method(item: any): number", CompletionItemKind::Method),
                    ("pop", "method(): any", CompletionItemKind::Method),
                    ("shift", "method(): any", CompletionItemKind::Method),
                    ("unshift", "method(item: any): number", CompletionItemKind::Method),
                    ("slice", "method(start?: number, end?: number): array", CompletionItemKind::Method),
                    ("splice", "method(start: number, deleteCount: number, ...items: any[]): array", CompletionItemKind::Method),
                    ("concat", "method(...arrays: array[]): array", CompletionItemKind::Method),
                    ("join", "method(separator?: string): string", CompletionItemKind::Method),
                    ("indexOf", "method(searchElement: any, fromIndex?: number): number", CompletionItemKind::Method),
                    ("lastIndexOf", "method(searchElement: any, fromIndex?: number): number", CompletionItemKind::Method),
                    ("forEach", "method(callback: function): void", CompletionItemKind::Method),
                    ("map", "method(callback: function): array", CompletionItemKind::Method),
                    ("filter", "method(callback: function): array", CompletionItemKind::Method),
                    ("reduce", "method(callback: function, initialValue?: any): any", CompletionItemKind::Method),
                    ("reduceRight", "method(callback: function, initialValue?: any): any", CompletionItemKind::Method),
                    ("some", "method(callback: function): boolean", CompletionItemKind::Method),
                    ("every", "method(callback: function): boolean", CompletionItemKind::Method),
                    ("find", "method(callback: function): any", CompletionItemKind::Method),
                    ("findIndex", "method(callback: function): number", CompletionItemKind::Method),
                ];
                
                for (name, detail, kind) in array_members {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind,
                        detail: Some(detail.to_string()),
                        documentation: None,
                        deprecated: false,
                        preselect: false,
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(name.to_string()),
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: Vec::new(),
                        command: None,
                        data: None,
                    });
                }
            }
            
            TypeInfo::String => {
                // Add string properties and methods
                let string_members = vec![
                    ("length", "property: number", CompletionItemKind::Property),
                    ("charAt", "method(index: number): string", CompletionItemKind::Method),
                    ("charCodeAt", "method(index: number): number", CompletionItemKind::Method),
                    ("concat", "method(...strings: string[]): string", CompletionItemKind::Method),
                    ("indexOf", "method(searchValue: string, fromIndex?: number): number", CompletionItemKind::Method),
                    ("lastIndexOf", "method(searchValue: string, fromIndex?: number): number", CompletionItemKind::Method),
                    ("localeCompare", "method(compareString: string): number", CompletionItemKind::Method),
                    ("match", "method(regexp: RegExp): array", CompletionItemKind::Method),
                    ("replace", "method(searchValue: string|RegExp, replaceValue: string): string", CompletionItemKind::Method),
                    ("search", "method(regexp: RegExp): number", CompletionItemKind::Method),
                    ("slice", "method(start?: number, end?: number): string", CompletionItemKind::Method),
                    ("split", "method(separator: string|RegExp, limit?: number): array", CompletionItemKind::Method),
                    ("substring", "method(start: number, end?: number): string", CompletionItemKind::Method),
                    ("toLowerCase", "method(): string", CompletionItemKind::Method),
                    ("toUpperCase", "method(): string", CompletionItemKind::Method),
                    ("trim", "method(): string", CompletionItemKind::Method),
                ];
                
                for (name, detail, kind) in string_members {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind,
                        detail: Some(detail.to_string()),
                        documentation: None,
                        deprecated: false,
                        preselect: false,
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(name.to_string()),
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: Vec::new(),
                        command: None,
                        data: None,
                    });
                }
            }
            
            TypeInfo::Number => {
                // Add number properties and methods
                let number_members = vec![
                    ("toFixed", "method(digits?: number): string", CompletionItemKind::Method),
                    ("toPrecision", "method(precision?: number): string", CompletionItemKind::Method),
                    ("toString", "method(radix?: number): string", CompletionItemKind::Method),
                    ("valueOf", "method(): number", CompletionItemKind::Method),
                ];
                
                for (name, detail, kind) in number_members {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind,
                        detail: Some(detail.to_string()),
                        documentation: None,
                        deprecated: false,
                        preselect: false,
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(name.to_string()),
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: Vec::new(),
                        command: None,
                        data: None,
                    });
                }
            }
            
            TypeInfo::Any => {
                // For Any type, provide common properties and methods
                let common_members = vec![
                    ("toString", "method(): string", CompletionItemKind::Method),
                    ("valueOf", "method(): any", CompletionItemKind::Method),
                ];
                
                for (name, detail, kind) in common_members {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind,
                        detail: Some(detail.to_string()),
                        documentation: None,
                        deprecated: false,
                        preselect: false,
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(name.to_string()),
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: Vec::new(),
                        command: None,
                        data: None,
                    });
                }
            }
            
            _ => {
                // No completions for other types
            }
        }
        
        Ok(())
    }
    
    /// Provide import completion
    fn provide_import_completion(
        &self,
        document: &Document,
        position: Position,
        line_prefix: &str,
        items: &mut Vec<CompletionItem>
    ) -> Result<(), String> {
        // Get module symbols from the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        let module_symbols = symbol_manager.get_module_symbols();
        
        // Add module symbols to completion items
        for symbol in module_symbols {
            items.push(CompletionItem {
                label: symbol.name.clone(),
                kind: CompletionItemKind::Module,
                detail: Some("module".to_string()),
                documentation: symbol.documentation.clone(),
                deprecated: false,
                preselect: false,
                sort_text: None,
                filter_text: None,
                insert_text: Some(symbol.name.clone()),
                insert_text_format: None,
                text_edit: None,
                additional_text_edits: Vec::new(),
                command: None,
                data: None,
            });
        }
        
        Ok(())
    }
    
    /// Get the current word being typed
    fn get_current_word(&self, line_prefix: &str) -> String {
        // Find the last word in the line prefix
        let mut word = String::new();
        
        for c in line_prefix.chars().rev() {
            if c.is_alphanumeric() || c == '_' {
                word.insert(0, c);
            } else {
                break;
            }
        }
        
        word
    }
    
    /// Get the object name before a dot
    fn get_object_before_dot(&self, line_prefix: &str) -> String {
        // Find the object name before the dot
        let mut object_name = String::new();
        let mut chars = line_prefix.chars().rev();
        
        // Skip the dot
        if let Some('.') = chars.next() {
            // Get the object name
            for c in chars {
                if c.is_alphanumeric() || c == '_' {
                    object_name.insert(0, c);
                } else {
                    break;
                }
            }
        }
        
        object_name
    }
}

/// Shared completion provider that can be used across threads
pub type SharedCompletionProvider = Arc<CompletionProvider>;

/// Create a new shared completion provider
pub fn create_shared_completion_provider(
    symbol_manager: SharedSymbolManager,
    semantic_analyzer: SharedSemanticAnalyzer,
    type_checker: SharedTypeChecker
) -> SharedCompletionProvider {
    Arc::new(CompletionProvider::new(symbol_manager, semantic_analyzer, type_checker))
}
