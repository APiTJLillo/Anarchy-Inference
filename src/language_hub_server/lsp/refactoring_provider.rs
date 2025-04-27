// Refactoring capabilities module for LSP-like Component
//
// This module provides refactoring functionality for Anarchy Inference code,
// including rename, extract function, extract variable, and inline refactorings.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, TextEdit, WorkspaceEdit};
use crate::language_hub_server::lsp::document::{Document, DocumentManager, SharedDocumentManager};
use crate::language_hub_server::lsp::parser_integration::AstNode;
use crate::language_hub_server::lsp::symbol_manager::{SymbolManager, SharedSymbolManager, Symbol, SymbolKind};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Refactoring options
#[derive(Debug, Clone)]
pub struct RefactoringOptions {
    /// Whether to update comments when refactoring
    pub update_comments: bool,
    
    /// Whether to update strings when refactoring
    pub update_strings: bool,
    
    /// Whether to preview refactorings before applying
    pub preview_refactorings: bool,
    
    /// Maximum number of files to modify in a single refactoring
    pub max_files: usize,
}

impl Default for RefactoringOptions {
    fn default() -> Self {
        RefactoringOptions {
            update_comments: true,
            update_strings: false,
            preview_refactorings: true,
            max_files: 50,
        }
    }
}

/// Refactoring provider for Anarchy Inference code
pub struct RefactoringProvider {
    /// The document manager
    document_manager: SharedDocumentManager,
    
    /// The symbol manager
    symbol_manager: SharedSymbolManager,
    
    /// Refactoring options
    options: RefactoringOptions,
}

impl RefactoringProvider {
    /// Create a new refactoring provider
    pub fn new(
        document_manager: SharedDocumentManager,
        symbol_manager: SharedSymbolManager,
        options: Option<RefactoringOptions>
    ) -> Self {
        RefactoringProvider {
            document_manager,
            symbol_manager,
            options: options.unwrap_or_default(),
        }
    }
    
    /// Rename a symbol
    pub fn rename(
        &self,
        document_uri: &str,
        position: Position,
        new_name: &str,
        ast: &AstNode
    ) -> Result<WorkspaceEdit, String> {
        // Get the symbol at the position
        let symbol = self.get_symbol_at_position(document_uri, position, ast)?;
        
        // Validate the new name
        self.validate_identifier(new_name)?;
        
        // Find all references to the symbol
        let references = self.find_references(&symbol, ast)?;
        
        // Create edits for each reference
        let mut edits_by_uri = HashMap::new();
        
        for reference in references {
            let uri = reference.uri.clone();
            let edit = TextEdit {
                range: reference.range,
                new_text: new_name.to_string(),
            };
            
            edits_by_uri.entry(uri).or_insert_with(Vec::new).push(edit);
        }
        
        // Limit the number of files to modify
        if edits_by_uri.len() > self.options.max_files {
            return Err(format!("Rename would modify {} files, which exceeds the maximum of {}",
                              edits_by_uri.len(), self.options.max_files));
        }
        
        // Create the workspace edit
        let workspace_edit = WorkspaceEdit {
            changes: edits_by_uri,
        };
        
        Ok(workspace_edit)
    }
    
    /// Extract a function
    pub fn extract_function(
        &self,
        document_uri: &str,
        range: Range,
        function_name: &str,
        ast: &AstNode
    ) -> Result<WorkspaceEdit, String> {
        // Validate the function name
        self.validate_identifier(function_name)?;
        
        // Get the document
        let document = self.get_document(document_uri)?;
        
        // Get the selected code
        let selected_code = self.get_text_in_range(&document, range)?;
        
        // Analyze the selected code
        let (input_variables, output_variables) = self.analyze_code_block(document_uri, range, ast)?;
        
        // Generate the function declaration
        let function_declaration = self.generate_function_declaration(
            function_name,
            &input_variables,
            &output_variables,
            &selected_code
        )?;
        
        // Generate the function call
        let function_call = self.generate_function_call(
            function_name,
            &input_variables,
            &output_variables
        )?;
        
        // Find the insertion point for the function declaration
        let insertion_point = self.find_function_insertion_point(document_uri, range, ast)?;
        
        // Create edits
        let mut edits = Vec::new();
        
        // Replace the selected code with the function call
        edits.push(TextEdit {
            range,
            new_text: function_call,
        });
        
        // Insert the function declaration
        edits.push(TextEdit {
            range: Range {
                start: insertion_point,
                end: insertion_point,
            },
            new_text: format!("\n\n{}", function_declaration),
        });
        
        // Create the workspace edit
        let mut changes = HashMap::new();
        changes.insert(document_uri.to_string(), edits);
        
        let workspace_edit = WorkspaceEdit {
            changes,
        };
        
        Ok(workspace_edit)
    }
    
    /// Extract a variable
    pub fn extract_variable(
        &self,
        document_uri: &str,
        range: Range,
        variable_name: &str,
        ast: &AstNode
    ) -> Result<WorkspaceEdit, String> {
        // Validate the variable name
        self.validate_identifier(variable_name)?;
        
        // Get the document
        let document = self.get_document(document_uri)?;
        
        // Get the selected expression
        let selected_expression = self.get_text_in_range(&document, range)?;
        
        // Find the statement containing the expression
        let containing_statement = self.find_containing_statement(document_uri, range, ast)?;
        
        // Find the insertion point for the variable declaration
        let insertion_point = containing_statement.range.start;
        
        // Generate the variable declaration
        let variable_declaration = format!("let {} = {};\n", variable_name, selected_expression);
        
        // Create edits
        let mut edits = Vec::new();
        
        // Replace the selected expression with the variable name
        edits.push(TextEdit {
            range,
            new_text: variable_name.to_string(),
        });
        
        // Insert the variable declaration
        edits.push(TextEdit {
            range: Range {
                start: insertion_point,
                end: insertion_point,
            },
            new_text: variable_declaration,
        });
        
        // Create the workspace edit
        let mut changes = HashMap::new();
        changes.insert(document_uri.to_string(), edits);
        
        let workspace_edit = WorkspaceEdit {
            changes,
        };
        
        Ok(workspace_edit)
    }
    
    /// Inline a variable or function
    pub fn inline(
        &self,
        document_uri: &str,
        position: Position,
        ast: &AstNode
    ) -> Result<WorkspaceEdit, String> {
        // Get the symbol at the position
        let symbol = self.get_symbol_at_position(document_uri, position, ast)?;
        
        match symbol.kind {
            SymbolKind::Variable => self.inline_variable(document_uri, &symbol, ast),
            SymbolKind::Function => self.inline_function(document_uri, &symbol, ast),
            _ => Err(format!("Cannot inline symbol of kind {:?}", symbol.kind)),
        }
    }
    
    /// Inline a variable
    fn inline_variable(
        &self,
        document_uri: &str,
        symbol: &Symbol,
        ast: &AstNode
    ) -> Result<WorkspaceEdit, String> {
        // Get the variable declaration
        let declaration = self.find_variable_declaration(document_uri, symbol, ast)?;
        
        // Get the variable value
        let value = self.get_variable_value(&declaration)?;
        
        // Find all references to the variable
        let references = self.find_references(symbol, ast)?;
        
        // Create edits for each reference
        let mut edits_by_uri = HashMap::new();
        
        for reference in references {
            let uri = reference.uri.clone();
            let edit = TextEdit {
                range: reference.range,
                new_text: value.clone(),
            };
            
            edits_by_uri.entry(uri).or_insert_with(Vec::new).push(edit);
        }
        
        // Add an edit to remove the variable declaration
        edits_by_uri.entry(document_uri.to_string()).or_insert_with(Vec::new).push(TextEdit {
            range: declaration.range,
            new_text: "".to_string(),
        });
        
        // Create the workspace edit
        let workspace_edit = WorkspaceEdit {
            changes: edits_by_uri,
        };
        
        Ok(workspace_edit)
    }
    
    /// Inline a function
    fn inline_function(
        &self,
        document_uri: &str,
        symbol: &Symbol,
        ast: &AstNode
    ) -> Result<WorkspaceEdit, String> {
        // Get the function declaration
        let declaration = self.find_function_declaration(document_uri, symbol, ast)?;
        
        // Get the function body
        let body = self.get_function_body(&declaration)?;
        
        // Find all references to the function
        let references = self.find_references(symbol, ast)?;
        
        // Create edits for each reference
        let mut edits_by_uri = HashMap::new();
        
        for reference in references {
            // Get the function call
            let call = self.get_function_call(&reference)?;
            
            // Get the arguments
            let arguments = self.get_function_call_arguments(&call)?;
            
            // Get the parameters
            let parameters = self.get_function_parameters(&declaration)?;
            
            // Create a mapping from parameters to arguments
            let mut parameter_map = HashMap::new();
            
            for (i, param) in parameters.iter().enumerate() {
                if i < arguments.len() {
                    parameter_map.insert(param.clone(), arguments[i].clone());
                }
            }
            
            // Replace parameters with arguments in the function body
            let inlined_body = self.replace_parameters_with_arguments(&body, &parameter_map)?;
            
            // Create an edit to replace the function call with the inlined body
            let uri = reference.uri.clone();
            let edit = TextEdit {
                range: call.range,
                new_text: inlined_body,
            };
            
            edits_by_uri.entry(uri).or_insert_with(Vec::new).push(edit);
        }
        
        // Add an edit to remove the function declaration
        edits_by_uri.entry(document_uri.to_string()).or_insert_with(Vec::new).push(TextEdit {
            range: declaration.range,
            new_text: "".to_string(),
        });
        
        // Create the workspace edit
        let workspace_edit = WorkspaceEdit {
            changes: edits_by_uri,
        };
        
        Ok(workspace_edit)
    }
    
    /// Get the symbol at a position
    fn get_symbol_at_position(
        &self,
        document_uri: &str,
        position: Position,
        ast: &AstNode
    ) -> Result<Symbol, String> {
        // Get the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        
        // Find the symbol at the position
        let symbols = symbol_manager.get_symbols_at_position(document_uri, position);
        
        if symbols.is_empty() {
            return Err(format!("No symbol found at position {:?}", position));
        }
        
        // Return the first symbol
        Ok(symbols[0].clone())
    }
    
    /// Validate an identifier
    fn validate_identifier(&self, identifier: &str) -> Result<(), String> {
        // Check if the identifier is empty
        if identifier.is_empty() {
            return Err("Identifier cannot be empty".to_string());
        }
        
        // Check if the identifier starts with a letter or underscore
        let first_char = identifier.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err("Identifier must start with a letter or underscore".to_string());
        }
        
        // Check if the identifier contains only letters, digits, and underscores
        for c in identifier.chars() {
            if !c.is_alphanumeric() && c != '_' {
                return Err("Identifier must contain only letters, digits, and underscores".to_string());
            }
        }
        
        // Check if the identifier is a reserved keyword
        let keywords = [
            "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete",
            "do", "else", "export", "extends", "finally", "for", "function", "if", "import", "in",
            "instanceof", "new", "return", "super", "switch", "this", "throw", "try", "typeof",
            "var", "void", "while", "with", "yield", "let", "static", "enum", "await", "implements",
            "interface", "package", "private", "protected", "public", "null", "true", "false"
        ];
        
        if keywords.contains(&identifier) {
            return Err(format!("Identifier cannot be a reserved keyword: {}", identifier));
        }
        
        Ok(())
    }
    
    /// Find references to a symbol
    fn find_references(
        &self,
        symbol: &Symbol,
        ast: &AstNode
    ) -> Result<Vec<Symbol>, String> {
        // Get the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        
        // Find all references to the symbol
        let references = symbol_manager.find_references(symbol);
        
        Ok(references)
    }
    
    /// Get a document
    fn get_document(&self, uri: &str) -> Result<Document, String> {
        // Get the document manager
        let document_manager = self.document_manager.lock().unwrap();
        
        // Get the document
        document_manager.get_document(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))
            .map(|doc| doc.clone())
    }
    
    /// Get text in a range
    fn get_text_in_range(&self, document: &Document, range: Range) -> Result<String, String> {
        // Get the text in the range
        let mut text = String::new();
        
        for line_number in range.start.line..=range.end.line {
            if let Some(line) = document.get_line(line_number) {
                let start_char = if line_number == range.start.line { range.start.character as usize } else { 0 };
                let end_char = if line_number == range.end.line { range.end.character as usize } else { line.len() };
                
                if start_char <= end_char && start_char < line.len() {
                    let end_char = std::cmp::min(end_char, line.len());
                    text.push_str(&line[start_char..end_char]);
                }
                
                if line_number < range.end.line {
                    text.push('\n');
                }
            }
        }
        
        Ok(text)
    }
    
    /// Analyze a code block to find input and output variables
    fn analyze_code_block(
        &self,
        document_uri: &str,
        range: Range,
        ast: &AstNode
    ) -> Result<(Vec<String>, Vec<String>), String> {
        // Get the symbol manager
        let symbol_manager = self.symbol_manager.lock().unwrap();
        
        // Find all symbols in the range
        let symbols_in_range = symbol_manager.get_symbols_in_range(document_uri, range);
        
        // Find all symbols referenced in the range but declared outside
        let mut input_variables = HashSet::new();
        
        for symbol in &symbols_in_range {
            if symbol.kind == SymbolKind::Variable && !symbol.is_declaration {
                let declaration = symbol_manager.find_declaration(symbol);
                
                if let Some(decl) = declaration {
                    if decl.range.start.line < range.start.line ||
                       (decl.range.start.line == range.start.line && decl.range.start.character < range.start.character) {
                        input_variables.insert(symbol.name.clone());
                    }
                }
            }
        }
        
        // Find all symbols declared in the range and used outside
        let mut output_variables = HashSet::new();
        
        for symbol in &symbols_in_range {
            if symbol.kind == SymbolKind::Variable && symbol.is_declaration {
                let references = symbol_manager.find_references(symbol);
                
                for reference in references {
                    if reference.range.start.line > range.end.line ||
                       (reference.range.start.line == range.end.line && reference.range.start.character > range.end.character) {
                        output_variables.insert(symbol.name.clone());
                        break;
                    }
                }
            }
        }
        
        Ok((input_variables.into_iter().collect(), output_variables.into_iter().collect()))
    }
    
    /// Generate a function declaration
    fn generate_function_declaration(
        &self,
        function_name: &str,
        input_variables: &[String],
        output_variables: &[String],
        body: &str
    ) -> Result<String, String> {
        let mut declaration = String::new();
        
        // Generate the function signature
        declaration.push_str(&format!("function {}(", function_name));
        
        // Add parameters
        for (i, var) in input_variables.iter().enumerate() {
            if i > 0 {
                declaration.push_str(", ");
            }
            declaration.push_str(var);
        }
        
        declaration.push_str(") {\n");
        
        // Add the function body
        for line in body.lines() {
            declaration.push_str(&format!("  {}\n", line));
        }
        
        // Add return statement if there are output variables
        if !output_variables.is_empty() {
            if output_variables.len() == 1 {
                declaration.push_str(&format!("  return {};\n", output_variables[0]));
            } else {
                declaration.push_str("  return {");
                
                for (i, var) in output_variables.iter().enumerate() {
                    if i > 0 {
                        declaration.push_str(", ");
                    }
                    declaration.push_str(&format!("{}: {}", var, var));
                }
                
                declaration.push_str("};\n");
            }
        }
        
        declaration.push_str("}");
        
        Ok(declaration)
    }
    
    /// Generate a function call
    fn generate_function_call(
        &self,
        function_name: &str,
        input_variables: &[String],
        output_variables: &[String]
    ) -> Result<String, String> {
        let mut call = String::new();
        
        // Generate variable declarations for output variables if needed
        if !output_variables.is_empty() {
            if output_variables.len() == 1 {
                call.push_str(&format!("let {} = ", output_variables[0]));
            } else {
                call.push_str("let {");
                
                for (i, var) in output_variables.iter().enumerate() {
                    if i > 0 {
                        call.push_str(", ");
                    }
                    call.push_str(var);
                }
                
                call.push_str("} = ");
            }
        }
        
        // Generate the function call
        call.push_str(&format!("{}(", function_name));
        
        // Add arguments
        for (i, var) in input_variables.iter().enumerate() {
            if i > 0 {
                call.push_str(", ");
            }
            call.push_str(var);
        }
        
        call.push_str(");");
        
        Ok(call)
    }
    
    /// Find the insertion point for a function declaration
    fn find_function_insertion_point(
        &self,
        document_uri: &str,
        range: Range,
        ast: &AstNode
    ) -> Result<Position, String> {
        // Find the containing function
        let containing_function = self.find_containing_function(document_uri, range, ast)?;
        
        // Insert after the containing function
        Ok(containing_function.range.end)
    }
    
    /// Find the containing function
    fn find_containing_function(
        &self,
        document_uri: &str,
        range: Range,
        ast: &AstNode
    ) -> Result<AstNode, String> {
        // Find the function that contains the range
        let functions = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "FunctionDeclaration" &&
            node.range.start.line <= range.start.line &&
            node.range.end.line >= range.end.line
        });
        
        if functions.is_empty() {
            return Err("No containing function found".to_string());
        }
        
        // Return the innermost function
        let mut innermost = &functions[0];
        
        for function in &functions {
            if function.range.start.line >= innermost.range.start.line &&
               function.range.end.line <= innermost.range.end.line {
                innermost = function;
            }
        }
        
        Ok(innermost.clone())
    }
    
    /// Find the containing statement
    fn find_containing_statement(
        &self,
        document_uri: &str,
        range: Range,
        ast: &AstNode
    ) -> Result<AstNode, String> {
        // Find the statement that contains the range
        let statements = AstUtils::collect_nodes(ast, |node| {
            (node.node_type == "ExpressionStatement" ||
             node.node_type == "VariableDeclaration" ||
             node.node_type == "ReturnStatement" ||
             node.node_type == "IfStatement" ||
             node.node_type == "WhileStatement" ||
             node.node_type == "ForStatement" ||
             node.node_type == "BlockStatement") &&
            node.range.start.line <= range.start.line &&
            node.range.end.line >= range.end.line
        });
        
        if statements.is_empty() {
            return Err("No containing statement found".to_string());
        }
        
        // Return the innermost statement
        let mut innermost = &statements[0];
        
        for statement in &statements {
            if statement.range.start.line >= innermost.range.start.line &&
               statement.range.end.line <= innermost.range.end.line {
                innermost = statement;
            }
        }
        
        Ok(innermost.clone())
    }
    
    /// Find a variable declaration
    fn find_variable_declaration(
        &self,
        document_uri: &str,
        symbol: &Symbol,
        ast: &AstNode
    ) -> Result<AstNode, String> {
        // Find the variable declaration
        let declarations = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "VariableDeclaration" &&
            node.properties.get("name").and_then(|v| v.as_str()) == Some(&symbol.name)
        });
        
        if declarations.is_empty() {
            return Err(format!("Variable declaration not found: {}", symbol.name));
        }
        
        Ok(declarations[0].clone())
    }
    
    /// Get the value of a variable declaration
    fn get_variable_value(&self, declaration: &AstNode) -> Result<String, String> {
        // Get the initializer
        if let Some(initializer) = declaration.properties.get("init") {
            Ok(initializer.to_string())
        } else {
            Err("Variable declaration has no initializer".to_string())
        }
    }
    
    /// Find a function declaration
    fn find_function_declaration(
        &self,
        document_uri: &str,
        symbol: &Symbol,
        ast: &AstNode
    ) -> Result<AstNode, String> {
        // Find the function declaration
        let declarations = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "FunctionDeclaration" &&
            node.properties.get("name").and_then(|v| v.as_str()) == Some(&symbol.name)
        });
        
        if declarations.is_empty() {
            return Err(format!("Function declaration not found: {}", symbol.name));
        }
        
        Ok(declarations[0].clone())
    }
    
    /// Get the body of a function declaration
    fn get_function_body(&self, declaration: &AstNode) -> Result<String, String> {
        // Get the body
        if let Some(body) = declaration.properties.get("body") {
            Ok(body.to_string())
        } else {
            Err("Function declaration has no body".to_string())
        }
    }
    
    /// Get a function call
    fn get_function_call(&self, reference: &Symbol) -> Result<AstNode, String> {
        // This is a simplified implementation
        // In a real implementation, we would use the AST to find the function call
        
        // For now, we'll just return a dummy node
        Ok(AstNode {
            node_type: "CallExpression".to_string(),
            range: reference.range,
            children: Vec::new(),
            properties: HashMap::new(),
        })
    }
    
    /// Get the arguments of a function call
    fn get_function_call_arguments(&self, call: &AstNode) -> Result<Vec<String>, String> {
        // This is a simplified implementation
        // In a real implementation, we would use the AST to find the arguments
        
        // For now, we'll just return an empty vector
        Ok(Vec::new())
    }
    
    /// Get the parameters of a function declaration
    fn get_function_parameters(&self, declaration: &AstNode) -> Result<Vec<String>, String> {
        // This is a simplified implementation
        // In a real implementation, we would use the AST to find the parameters
        
        // For now, we'll just return an empty vector
        Ok(Vec::new())
    }
    
    /// Replace parameters with arguments in a function body
    fn replace_parameters_with_arguments(
        &self,
        body: &str,
        parameter_map: &HashMap<String, String>
    ) -> Result<String, String> {
        // This is a simplified implementation
        // In a real implementation, we would use the AST to replace parameters with arguments
        
        // For now, we'll just return the body unchanged
        Ok(body.to_string())
    }
    
    /// Set refactoring options
    pub fn set_options(&mut self, options: RefactoringOptions) {
        self.options = options;
    }
    
    /// Get refactoring options
    pub fn get_options(&self) -> RefactoringOptions {
        self.options.clone()
    }
}

/// Shared refactoring provider that can be used across threads
pub type SharedRefactoringProvider = Arc<Mutex<RefactoringProvider>>;

/// Create a new shared refactoring provider
pub fn create_shared_refactoring_provider(
    document_manager: SharedDocumentManager,
    symbol_manager: SharedSymbolManager,
    options: Option<RefactoringOptions>
) -> SharedRefactoringProvider {
    Arc::new(Mutex::new(RefactoringProvider::new(
        document_manager,
        symbol_manager,
        options
    )))
}
