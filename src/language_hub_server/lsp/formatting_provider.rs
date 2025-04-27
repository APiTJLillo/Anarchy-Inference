// Formatting provider module for LSP-like Component
//
// This module provides code formatting functionality for Anarchy Inference code,
// including indentation, spacing, and style enforcement.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, TextEdit};
use crate::language_hub_server::lsp::document::Document;
use crate::language_hub_server::lsp::parser_integration::AstNode;
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// Formatting options
#[derive(Debug, Clone)]
pub struct FormattingOptions {
    /// Number of spaces per indentation level
    pub tab_size: u32,
    
    /// Whether to use spaces for indentation
    pub insert_spaces: bool,
    
    /// Whether to trim trailing whitespace
    pub trim_trailing_whitespace: bool,
    
    /// Whether to insert a final newline
    pub insert_final_newline: bool,
    
    /// Whether to trim final newlines
    pub trim_final_newlines: bool,
    
    /// Maximum line length
    pub max_line_length: u32,
    
    /// Whether to enforce semicolons
    pub enforce_semicolons: bool,
    
    /// Whether to enforce braces on the same line
    pub braces_same_line: bool,
    
    /// Whether to enforce single quotes
    pub single_quotes: bool,
    
    /// Whether to enforce trailing commas
    pub trailing_commas: bool,
    
    /// Whether to enforce spaces around operators
    pub spaces_around_operators: bool,
    
    /// Whether to enforce spaces after commas
    pub spaces_after_commas: bool,
    
    /// Whether to enforce spaces inside braces
    pub spaces_inside_braces: bool,
    
    /// Whether to enforce spaces inside parentheses
    pub spaces_inside_parentheses: bool,
    
    /// Whether to enforce spaces inside brackets
    pub spaces_inside_brackets: bool,
}

impl Default for FormattingOptions {
    fn default() -> Self {
        FormattingOptions {
            tab_size: 2,
            insert_spaces: true,
            trim_trailing_whitespace: true,
            insert_final_newline: true,
            trim_final_newlines: true,
            max_line_length: 80,
            enforce_semicolons: true,
            braces_same_line: true,
            single_quotes: true,
            trailing_commas: false,
            spaces_around_operators: true,
            spaces_after_commas: true,
            spaces_inside_braces: true,
            spaces_inside_parentheses: false,
            spaces_inside_brackets: false,
        }
    }
}

/// Formatting provider for Anarchy Inference code
pub struct FormattingProvider {
    /// Default formatting options
    default_options: FormattingOptions,
    
    /// Custom formatting options by document URI
    custom_options: HashMap<String, FormattingOptions>,
}

impl FormattingProvider {
    /// Create a new formatting provider
    pub fn new(default_options: Option<FormattingOptions>) -> Self {
        FormattingProvider {
            default_options: default_options.unwrap_or_default(),
            custom_options: HashMap::new(),
        }
    }
    
    /// Format a document
    pub fn format_document(
        &self,
        document: &Document,
        ast: &AstNode,
        options: Option<FormattingOptions>
    ) -> Result<Vec<TextEdit>, String> {
        // Get formatting options
        let options = options.unwrap_or_else(|| self.get_options(&document.uri));
        
        // Format the document
        let mut edits = Vec::new();
        
        // Fix indentation
        self.fix_indentation(document, ast, &options, &mut edits)?;
        
        // Fix spacing
        self.fix_spacing(document, ast, &options, &mut edits)?;
        
        // Fix semicolons
        self.fix_semicolons(document, ast, &options, &mut edits)?;
        
        // Fix quotes
        self.fix_quotes(document, ast, &options, &mut edits)?;
        
        // Fix trailing commas
        self.fix_trailing_commas(document, ast, &options, &mut edits)?;
        
        // Fix line length
        self.fix_line_length(document, ast, &options, &mut edits)?;
        
        // Fix trailing whitespace
        self.fix_trailing_whitespace(document, &options, &mut edits)?;
        
        // Fix final newline
        self.fix_final_newline(document, &options, &mut edits)?;
        
        // Merge overlapping edits
        let merged_edits = self.merge_edits(edits);
        
        Ok(merged_edits)
    }
    
    /// Format a range in a document
    pub fn format_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: Option<FormattingOptions>
    ) -> Result<Vec<TextEdit>, String> {
        // Get formatting options
        let options = options.unwrap_or_else(|| self.get_options(&document.uri));
        
        // Format the range
        let mut edits = Vec::new();
        
        // Fix indentation in range
        self.fix_indentation_in_range(document, ast, range, &options, &mut edits)?;
        
        // Fix spacing in range
        self.fix_spacing_in_range(document, ast, range, &options, &mut edits)?;
        
        // Fix semicolons in range
        self.fix_semicolons_in_range(document, ast, range, &options, &mut edits)?;
        
        // Fix quotes in range
        self.fix_quotes_in_range(document, ast, range, &options, &mut edits)?;
        
        // Fix trailing commas in range
        self.fix_trailing_commas_in_range(document, ast, range, &options, &mut edits)?;
        
        // Fix line length in range
        self.fix_line_length_in_range(document, ast, range, &options, &mut edits)?;
        
        // Fix trailing whitespace in range
        self.fix_trailing_whitespace_in_range(document, range, &options, &mut edits)?;
        
        // Merge overlapping edits
        let merged_edits = self.merge_edits(edits);
        
        Ok(merged_edits)
    }
    
    /// Format on type
    pub fn format_on_type(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        ch: char,
        options: Option<FormattingOptions>
    ) -> Result<Vec<TextEdit>, String> {
        // Get formatting options
        let options = options.unwrap_or_else(|| self.get_options(&document.uri));
        
        // Format on type
        let mut edits = Vec::new();
        
        match ch {
            '}' => {
                // Fix indentation for closing brace
                self.fix_brace_indentation(document, ast, position, &options, &mut edits)?;
            }
            ';' => {
                // Fix spacing around semicolon
                self.fix_semicolon_spacing(document, ast, position, &options, &mut edits)?;
            }
            ',' => {
                // Fix spacing after comma
                self.fix_comma_spacing(document, ast, position, &options, &mut edits)?;
            }
            '{' => {
                // Fix spacing around opening brace
                self.fix_brace_spacing(document, ast, position, &options, &mut edits)?;
            }
            '(' => {
                // Fix spacing inside parentheses
                self.fix_parenthesis_spacing(document, ast, position, &options, &mut edits)?;
            }
            ')' => {
                // Fix spacing inside parentheses
                self.fix_parenthesis_spacing(document, ast, position, &options, &mut edits)?;
            }
            '[' => {
                // Fix spacing inside brackets
                self.fix_bracket_spacing(document, ast, position, &options, &mut edits)?;
            }
            ']' => {
                // Fix spacing inside brackets
                self.fix_bracket_spacing(document, ast, position, &options, &mut edits)?;
            }
            '\n' => {
                // Fix indentation for new line
                self.fix_newline_indentation(document, ast, position, &options, &mut edits)?;
            }
            _ => {
                // No formatting for other characters
            }
        }
        
        Ok(edits)
    }
    
    /// Set formatting options for a document
    pub fn set_options(&mut self, uri: &str, options: FormattingOptions) {
        self.custom_options.insert(uri.to_string(), options);
    }
    
    /// Get formatting options for a document
    pub fn get_options(&self, uri: &str) -> FormattingOptions {
        self.custom_options.get(uri).cloned().unwrap_or_else(|| self.default_options.clone())
    }
    
    /// Reset formatting options for a document
    pub fn reset_options(&mut self, uri: &str) {
        self.custom_options.remove(uri);
    }
    
    /// Set default formatting options
    pub fn set_default_options(&mut self, options: FormattingOptions) {
        self.default_options = options;
    }
    
    /// Get default formatting options
    pub fn get_default_options(&self) -> FormattingOptions {
        self.default_options.clone()
    }
    
    /// Fix indentation
    fn fix_indentation(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Calculate indentation for each line
        let mut indentation_levels = HashMap::new();
        self.calculate_indentation_levels(ast, &mut indentation_levels);
        
        // Fix indentation for each line
        for (line_number, indent_level) in indentation_levels {
            if line_number >= document.line_count() as u32 {
                continue;
            }
            
            let line = document.get_line(line_number).unwrap_or_default();
            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count() as u32;
            let expected_indent = indent_level * options.tab_size;
            
            if current_indent != expected_indent {
                // Create an edit to fix indentation
                let indent_str = if options.insert_spaces {
                    " ".repeat(expected_indent as usize)
                } else {
                    "\t".repeat((expected_indent / options.tab_size) as usize)
                };
                
                edits.push(TextEdit {
                    range: Range {
                        start: Position { line: line_number, character: 0 },
                        end: Position { line: line_number, character: current_indent },
                    },
                    new_text: indent_str,
                });
            }
        }
        
        Ok(())
    }
    
    /// Fix indentation in range
    fn fix_indentation_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Calculate indentation for each line
        let mut indentation_levels = HashMap::new();
        self.calculate_indentation_levels(ast, &mut indentation_levels);
        
        // Fix indentation for lines in range
        for line_number in range.start.line..=range.end.line {
            if line_number >= document.line_count() as u32 {
                continue;
            }
            
            if let Some(indent_level) = indentation_levels.get(&line_number) {
                let line = document.get_line(line_number).unwrap_or_default();
                let current_indent = line.chars().take_while(|c| c.is_whitespace()).count() as u32;
                let expected_indent = indent_level * options.tab_size;
                
                if current_indent != expected_indent {
                    // Create an edit to fix indentation
                    let indent_str = if options.insert_spaces {
                        " ".repeat(expected_indent as usize)
                    } else {
                        "\t".repeat((expected_indent / options.tab_size) as usize)
                    };
                    
                    edits.push(TextEdit {
                        range: Range {
                            start: Position { line: line_number, character: 0 },
                            end: Position { line: line_number, character: current_indent },
                        },
                        new_text: indent_str,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate indentation levels for each line
    fn calculate_indentation_levels(&self, ast: &AstNode, levels: &mut HashMap<u32, u32>) {
        // Set indentation level for the current node
        let indent_level = self.get_node_indent_level(ast);
        let line = ast.range.start.line;
        
        // Only set the indentation level if it's not already set or if the current level is lower
        if !levels.contains_key(&line) || levels[&line] > indent_level {
            levels.insert(line, indent_level);
        }
        
        // Calculate indentation for children
        for child in &ast.children {
            self.calculate_indentation_levels(child, levels);
        }
    }
    
    /// Get indentation level for a node
    fn get_node_indent_level(&self, node: &AstNode) -> u32 {
        // Calculate indentation based on node type and parent
        match node.node_type.as_str() {
            "Program" => 0,
            "ModuleDeclaration" => 0,
            "FunctionDeclaration" => {
                // Function declarations at the top level have indent 0
                // Nested function declarations have parent indent + 1
                if self.is_top_level_node(node) {
                    0
                } else {
                    1
                }
            }
            "BlockStatement" => {
                // Block statements have parent indent + 1
                if let Some(parent) = self.get_parent_indent_level(node) {
                    parent + 1
                } else {
                    1
                }
            }
            "IfStatement" | "WhileStatement" | "ForStatement" | "SwitchStatement" => {
                // Control flow statements have parent indent
                if let Some(parent) = self.get_parent_indent_level(node) {
                    parent
                } else {
                    0
                }
            }
            "SwitchCase" => {
                // Switch cases have parent indent + 1
                if let Some(parent) = self.get_parent_indent_level(node) {
                    parent + 1
                } else {
                    1
                }
            }
            _ => {
                // Other nodes inherit parent indent
                if let Some(parent) = self.get_parent_indent_level(node) {
                    parent
                } else {
                    0
                }
            }
        }
    }
    
    /// Get parent indentation level
    fn get_parent_indent_level(&self, node: &AstNode) -> Option<u32> {
        // This is a simplified implementation
        // In a real implementation, we would traverse the AST to find the parent
        
        // For now, we'll use a heuristic based on the node's range
        if node.range.start.line > 0 {
            Some(1)
        } else {
            None
        }
    }
    
    /// Check if a node is at the top level
    fn is_top_level_node(&self, node: &AstNode) -> bool {
        // This is a simplified implementation
        // In a real implementation, we would check if the parent is the Program node
        
        // For now, we'll use a heuristic based on the node's range
        node.range.start.line == 0 || node.range.start.character == 0
    }
    
    /// Fix spacing
    fn fix_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Fix spacing around operators
        if options.spaces_around_operators {
            self.fix_operator_spacing(document, ast, options, edits)?;
        }
        
        // Fix spacing after commas
        if options.spaces_after_commas {
            self.fix_comma_spacing_all(document, ast, options, edits)?;
        }
        
        // Fix spacing inside braces
        if options.spaces_inside_braces {
            self.fix_brace_spacing_all(document, ast, options, edits)?;
        }
        
        // Fix spacing inside parentheses
        if options.spaces_inside_parentheses {
            self.fix_parenthesis_spacing_all(document, ast, options, edits)?;
        }
        
        // Fix spacing inside brackets
        if options.spaces_inside_brackets {
            self.fix_bracket_spacing_all(document, ast, options, edits)?;
        }
        
        Ok(())
    }
    
    /// Fix spacing in range
    fn fix_spacing_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Fix spacing around operators in range
        if options.spaces_around_operators {
            self.fix_operator_spacing_in_range(document, ast, range, options, edits)?;
        }
        
        // Fix spacing after commas in range
        if options.spaces_after_commas {
            self.fix_comma_spacing_in_range(document, ast, range, options, edits)?;
        }
        
        // Fix spacing inside braces in range
        if options.spaces_inside_braces {
            self.fix_brace_spacing_in_range(document, ast, range, options, edits)?;
        }
        
        // Fix spacing inside parentheses in range
        if options.spaces_inside_parentheses {
            self.fix_parenthesis_spacing_in_range(document, ast, range, options, edits)?;
        }
        
        // Fix spacing inside brackets in range
        if options.spaces_inside_brackets {
            self.fix_bracket_spacing_in_range(document, ast, range, options, edits)?;
        }
        
        Ok(())
    }
    
    /// Fix operator spacing
    fn fix_operator_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Find all binary expressions
        let binary_expressions = AstUtils::collect_nodes(ast, |node| node.node_type == "BinaryExpression");
        
        for expr in binary_expressions {
            if let Some(operator) = expr.properties.get("operator").and_then(|v| v.as_str()) {
                // Get the operator position
                let operator_pos = self.find_operator_position(document, &expr, operator);
                
                if let Some(pos) = operator_pos {
                    // Check spacing before operator
                    if pos.character > 0 {
                        let line = document.get_line(pos.line).unwrap_or_default();
                        let char_before = line.chars().nth((pos.character - 1) as usize);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() {
                                // Add space before operator
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: pos.line, character: pos.character - 1 },
                                        end: Position { line: pos.line, character: pos.character },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                    
                    // Check spacing after operator
                    let line = document.get_line(pos.line).unwrap_or_default();
                    let operator_end = pos.character + operator.len() as u32;
                    
                    if operator_end < line.len() as u32 {
                        let char_after = line.chars().nth(operator_end as usize);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() {
                                // Add space after operator
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: pos.line, character: operator_end },
                                        end: Position { line: pos.line, character: operator_end + 1 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix operator spacing in range
    fn fix_operator_spacing_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Find all binary expressions in range
        let binary_expressions = AstUtils::collect_nodes(ast, |node| {
            node.node_type == "BinaryExpression" &&
            node.range.start.line >= range.start.line &&
            node.range.end.line <= range.end.line
        });
        
        for expr in binary_expressions {
            if let Some(operator) = expr.properties.get("operator").and_then(|v| v.as_str()) {
                // Get the operator position
                let operator_pos = self.find_operator_position(document, &expr, operator);
                
                if let Some(pos) = operator_pos {
                    // Check if operator is in range
                    if pos.line >= range.start.line && pos.line <= range.end.line {
                        // Check spacing before operator
                        if pos.character > 0 {
                            let line = document.get_line(pos.line).unwrap_or_default();
                            let char_before = line.chars().nth((pos.character - 1) as usize);
                            
                            if let Some(c) = char_before {
                                if !c.is_whitespace() {
                                    // Add space before operator
                                    edits.push(TextEdit {
                                        range: Range {
                                            start: Position { line: pos.line, character: pos.character - 1 },
                                            end: Position { line: pos.line, character: pos.character },
                                        },
                                        new_text: format!("{} ", c),
                                    });
                                }
                            }
                        }
                        
                        // Check spacing after operator
                        let line = document.get_line(pos.line).unwrap_or_default();
                        let operator_end = pos.character + operator.len() as u32;
                        
                        if operator_end < line.len() as u32 {
                            let char_after = line.chars().nth(operator_end as usize);
                            
                            if let Some(c) = char_after {
                                if !c.is_whitespace() {
                                    // Add space after operator
                                    edits.push(TextEdit {
                                        range: Range {
                                            start: Position { line: pos.line, character: operator_end },
                                            end: Position { line: pos.line, character: operator_end + 1 },
                                        },
                                        new_text: format!(" {}", c),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Find operator position in a binary expression
    fn find_operator_position(&self, document: &Document, expr: &AstNode, operator: &str) -> Option<Position> {
        // This is a simplified implementation
        // In a real implementation, we would use the AST to find the exact position
        
        // For now, we'll search for the operator in the document text
        let start_line = expr.range.start.line;
        let end_line = expr.range.end.line;
        
        for line_number in start_line..=end_line {
            if let Some(line) = document.get_line(line_number) {
                if let Some(pos) = line.find(operator) {
                    return Some(Position {
                        line: line_number,
                        character: pos as u32,
                    });
                }
            }
        }
        
        None
    }
    
    /// Fix comma spacing
    fn fix_comma_spacing_all(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Find all commas in the document
        for line_number in 0..document.line_count() as u32 {
            if let Some(line) = document.get_line(line_number) {
                let mut pos = 0;
                
                while let Some(comma_pos) = line[pos..].find(',') {
                    let comma_pos = pos + comma_pos;
                    pos = comma_pos + 1;
                    
                    // Check spacing after comma
                    if comma_pos + 1 < line.len() {
                        let char_after = line.chars().nth(comma_pos + 1);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() {
                                // Add space after comma
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: comma_pos as u32 + 1 },
                                        end: Position { line: line_number, character: comma_pos as u32 + 2 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix comma spacing in range
    fn fix_comma_spacing_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Find all commas in the range
        for line_number in range.start.line..=range.end.line {
            if let Some(line) = document.get_line(line_number) {
                let start_char = if line_number == range.start.line { range.start.character as usize } else { 0 };
                let end_char = if line_number == range.end.line { range.end.character as usize } else { line.len() };
                
                if start_char >= end_char || start_char >= line.len() {
                    continue;
                }
                
                let mut pos = start_char;
                
                while let Some(comma_pos) = line[pos..end_char].find(',') {
                    let comma_pos = pos + comma_pos;
                    pos = comma_pos + 1;
                    
                    // Check spacing after comma
                    if comma_pos + 1 < line.len() {
                        let char_after = line.chars().nth(comma_pos + 1);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() {
                                // Add space after comma
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: comma_pos as u32 + 1 },
                                        end: Position { line: line_number, character: comma_pos as u32 + 2 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix comma spacing at position
    fn fix_comma_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if let Some(line) = document.get_line(position.line) {
            // Check if the character at position is a comma
            if position.character < line.len() as u32 {
                let char_at_pos = line.chars().nth(position.character as usize);
                
                if let Some(',') = char_at_pos {
                    // Check spacing after comma
                    if position.character + 1 < line.len() as u32 {
                        let char_after = line.chars().nth((position.character + 1) as usize);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() {
                                // Add space after comma
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: position.line, character: position.character + 1 },
                                        end: Position { line: position.line, character: position.character + 2 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix brace spacing
    fn fix_brace_spacing_all(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Find all braces in the document
        for line_number in 0..document.line_count() as u32 {
            if let Some(line) = document.get_line(line_number) {
                // Fix opening braces
                let mut pos = 0;
                
                while let Some(brace_pos) = line[pos..].find('{') {
                    let brace_pos = pos + brace_pos;
                    pos = brace_pos + 1;
                    
                    // Check spacing after opening brace
                    if brace_pos + 1 < line.len() {
                        let char_after = line.chars().nth(brace_pos + 1);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() && c != '}' {
                                // Add space after opening brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: brace_pos as u32 + 1 },
                                        end: Position { line: line_number, character: brace_pos as u32 + 2 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                    
                    // Check spacing before opening brace
                    if brace_pos > 0 {
                        let char_before = line.chars().nth(brace_pos - 1);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() {
                                // Add space before opening brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: brace_pos as u32 - 1 },
                                        end: Position { line: line_number, character: brace_pos as u32 },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                }
                
                // Fix closing braces
                pos = 0;
                
                while let Some(brace_pos) = line[pos..].find('}') {
                    let brace_pos = pos + brace_pos;
                    pos = brace_pos + 1;
                    
                    // Check spacing before closing brace
                    if brace_pos > 0 {
                        let char_before = line.chars().nth(brace_pos - 1);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() && c != '{' {
                                // Add space before closing brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: brace_pos as u32 - 1 },
                                        end: Position { line: line_number, character: brace_pos as u32 },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix brace spacing in range
    fn fix_brace_spacing_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Find all braces in the range
        for line_number in range.start.line..=range.end.line {
            if let Some(line) = document.get_line(line_number) {
                let start_char = if line_number == range.start.line { range.start.character as usize } else { 0 };
                let end_char = if line_number == range.end.line { range.end.character as usize } else { line.len() };
                
                if start_char >= end_char || start_char >= line.len() {
                    continue;
                }
                
                // Fix opening braces
                let mut pos = start_char;
                
                while let Some(brace_pos) = line[pos..end_char].find('{') {
                    let brace_pos = pos + brace_pos;
                    pos = brace_pos + 1;
                    
                    // Check spacing after opening brace
                    if brace_pos + 1 < line.len() {
                        let char_after = line.chars().nth(brace_pos + 1);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() && c != '}' {
                                // Add space after opening brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: brace_pos as u32 + 1 },
                                        end: Position { line: line_number, character: brace_pos as u32 + 2 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                    
                    // Check spacing before opening brace
                    if brace_pos > 0 {
                        let char_before = line.chars().nth(brace_pos - 1);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() {
                                // Add space before opening brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: brace_pos as u32 - 1 },
                                        end: Position { line: line_number, character: brace_pos as u32 },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                }
                
                // Fix closing braces
                pos = start_char;
                
                while let Some(brace_pos) = line[pos..end_char].find('}') {
                    let brace_pos = pos + brace_pos;
                    pos = brace_pos + 1;
                    
                    // Check spacing before closing brace
                    if brace_pos > 0 {
                        let char_before = line.chars().nth(brace_pos - 1);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() && c != '{' {
                                // Add space before closing brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: line_number, character: brace_pos as u32 - 1 },
                                        end: Position { line: line_number, character: brace_pos as u32 },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix brace spacing at position
    fn fix_brace_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if let Some(line) = document.get_line(position.line) {
            // Check if the character at position is a brace
            if position.character < line.len() as u32 {
                let char_at_pos = line.chars().nth(position.character as usize);
                
                if let Some('{') = char_at_pos {
                    // Check spacing after opening brace
                    if position.character + 1 < line.len() as u32 {
                        let char_after = line.chars().nth((position.character + 1) as usize);
                        
                        if let Some(c) = char_after {
                            if !c.is_whitespace() && c != '}' {
                                // Add space after opening brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: position.line, character: position.character + 1 },
                                        end: Position { line: position.line, character: position.character + 2 },
                                    },
                                    new_text: format!(" {}", c),
                                });
                            }
                        }
                    }
                    
                    // Check spacing before opening brace
                    if position.character > 0 {
                        let char_before = line.chars().nth((position.character - 1) as usize);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() {
                                // Add space before opening brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: position.line, character: position.character - 1 },
                                        end: Position { line: position.line, character: position.character },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                } else if let Some('}') = char_at_pos {
                    // Check spacing before closing brace
                    if position.character > 0 {
                        let char_before = line.chars().nth((position.character - 1) as usize);
                        
                        if let Some(c) = char_before {
                            if !c.is_whitespace() && c != '{' {
                                // Add space before closing brace
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: position.line, character: position.character - 1 },
                                        end: Position { line: position.line, character: position.character },
                                    },
                                    new_text: format!("{} ", c),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix brace indentation at position
    fn fix_brace_indentation(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if let Some(line) = document.get_line(position.line) {
            // Check if the character at position is a closing brace
            if position.character < line.len() as u32 {
                let char_at_pos = line.chars().nth(position.character as usize);
                
                if let Some('}') = char_at_pos {
                    // Find the matching opening brace
                    if let Some(opening_brace_pos) = self.find_matching_opening_brace(document, position) {
                        // Get the indentation of the line with the opening brace
                        if let Some(opening_line) = document.get_line(opening_brace_pos.line) {
                            let opening_indent = opening_line.chars().take_while(|c| c.is_whitespace()).count();
                            
                            // Get the current indentation of the line with the closing brace
                            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
                            
                            // If the indentation is different, fix it
                            if current_indent != opening_indent {
                                let indent_str = if options.insert_spaces {
                                    " ".repeat(opening_indent)
                                } else {
                                    "\t".repeat(opening_indent / options.tab_size as usize)
                                };
                                
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: position.line, character: 0 },
                                        end: Position { line: position.line, character: current_indent as u32 },
                                    },
                                    new_text: indent_str,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Find matching opening brace for a closing brace
    fn find_matching_opening_brace(&self, document: &Document, closing_brace_pos: Position) -> Option<Position> {
        // This is a simplified implementation
        // In a real implementation, we would use the AST to find the matching brace
        
        // For now, we'll use a simple stack-based approach
        let mut brace_stack = Vec::new();
        
        // Start from the closing brace and search backwards
        for line_number in (0..=closing_brace_pos.line).rev() {
            if let Some(line) = document.get_line(line_number) {
                let end_char = if line_number == closing_brace_pos.line { closing_brace_pos.character as usize } else { line.len() };
                
                for (i, c) in line.chars().take(end_char).enumerate().rev() {
                    if c == '}' {
                        brace_stack.push('}');
                    } else if c == '{' {
                        if brace_stack.is_empty() {
                            // Found matching opening brace
                            return Some(Position {
                                line: line_number,
                                character: i as u32,
                            });
                        } else {
                            brace_stack.pop();
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Fix parenthesis spacing
    fn fix_parenthesis_spacing_all(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_brace_spacing_all, but for parentheses
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix parenthesis spacing in range
    fn fix_parenthesis_spacing_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_brace_spacing_in_range, but for parentheses
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix parenthesis spacing at position
    fn fix_parenthesis_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_brace_spacing, but for parentheses
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix bracket spacing
    fn fix_bracket_spacing_all(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_brace_spacing_all, but for brackets
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix bracket spacing in range
    fn fix_bracket_spacing_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_brace_spacing_in_range, but for brackets
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix bracket spacing at position
    fn fix_bracket_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_brace_spacing, but for brackets
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix semicolons
    fn fix_semicolons(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if options.enforce_semicolons {
            // Find all statements that should end with semicolons
            let statements = AstUtils::collect_nodes(ast, |node| {
                node.node_type == "ExpressionStatement" ||
                node.node_type == "VariableDeclaration" ||
                node.node_type == "ReturnStatement"
            });
            
            for stmt in statements {
                // Check if the statement ends with a semicolon
                let line_number = stmt.range.end.line;
                
                if let Some(line) = document.get_line(line_number) {
                    let line_trimmed = line.trim_end();
                    
                    if !line_trimmed.ends_with(';') {
                        // Add semicolon at the end of the statement
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: line_number, character: line_trimmed.len() as u32 },
                                end: Position { line: line_number, character: line_trimmed.len() as u32 },
                            },
                            new_text: ";".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix semicolons in range
    fn fix_semicolons_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if options.enforce_semicolons {
            // Find all statements in range that should end with semicolons
            let statements = AstUtils::collect_nodes(ast, |node| {
                (node.node_type == "ExpressionStatement" ||
                 node.node_type == "VariableDeclaration" ||
                 node.node_type == "ReturnStatement") &&
                node.range.start.line >= range.start.line &&
                node.range.end.line <= range.end.line
            });
            
            for stmt in statements {
                // Check if the statement ends with a semicolon
                let line_number = stmt.range.end.line;
                
                if let Some(line) = document.get_line(line_number) {
                    let line_trimmed = line.trim_end();
                    
                    if !line_trimmed.ends_with(';') {
                        // Add semicolon at the end of the statement
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: line_number, character: line_trimmed.len() as u32 },
                                end: Position { line: line_number, character: line_trimmed.len() as u32 },
                            },
                            new_text: ";".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix semicolon spacing at position
    fn fix_semicolon_spacing(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if let Some(line) = document.get_line(position.line) {
            // Check if the character at position is a semicolon
            if position.character < line.len() as u32 {
                let char_at_pos = line.chars().nth(position.character as usize);
                
                if let Some(';') = char_at_pos {
                    // Check spacing before semicolon
                    if position.character > 0 {
                        let char_before = line.chars().nth((position.character - 1) as usize);
                        
                        if let Some(c) = char_before {
                            if c.is_whitespace() {
                                // Remove space before semicolon
                                let mut start_pos = position.character - 1;
                                
                                while start_pos > 0 {
                                    let prev_char = line.chars().nth((start_pos - 1) as usize);
                                    if let Some(pc) = prev_char {
                                        if pc.is_whitespace() {
                                            start_pos -= 1;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                
                                edits.push(TextEdit {
                                    range: Range {
                                        start: Position { line: position.line, character: start_pos },
                                        end: Position { line: position.line, character: position.character },
                                    },
                                    new_text: ";".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix quotes
    fn fix_quotes(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if options.single_quotes {
            // Find all string literals
            let string_literals = AstUtils::collect_nodes(ast, |node| {
                node.node_type == "Literal" &&
                node.properties.get("literalType").and_then(|v| v.as_str()) == Some("string")
            });
            
            for literal in string_literals {
                // Check if the string literal uses double quotes
                let line_number = literal.range.start.line;
                let start_char = literal.range.start.character;
                let end_char = literal.range.end.character;
                
                if let Some(line) = document.get_line(line_number) {
                    if start_char < line.len() as u32 && end_char <= line.len() as u32 {
                        let literal_text = &line[start_char as usize..end_char as usize];
                        
                        if literal_text.starts_with('"') && literal_text.ends_with('"') {
                            // Convert double quotes to single quotes
                            let new_text = format!("'{}'", &literal_text[1..literal_text.len() - 1]
                                .replace("'", "\\'")
                                .replace("\\'", "'"));
                            
                            edits.push(TextEdit {
                                range: Range {
                                    start: Position { line: line_number, character: start_char },
                                    end: Position { line: line_number, character: end_char },
                                },
                                new_text,
                            });
                        }
                    }
                }
            }
        } else {
            // Find all string literals
            let string_literals = AstUtils::collect_nodes(ast, |node| {
                node.node_type == "Literal" &&
                node.properties.get("literalType").and_then(|v| v.as_str()) == Some("string")
            });
            
            for literal in string_literals {
                // Check if the string literal uses single quotes
                let line_number = literal.range.start.line;
                let start_char = literal.range.start.character;
                let end_char = literal.range.end.character;
                
                if let Some(line) = document.get_line(line_number) {
                    if start_char < line.len() as u32 && end_char <= line.len() as u32 {
                        let literal_text = &line[start_char as usize..end_char as usize];
                        
                        if literal_text.starts_with('\'') && literal_text.ends_with('\'') {
                            // Convert single quotes to double quotes
                            let new_text = format!("\"{}\"", &literal_text[1..literal_text.len() - 1]
                                .replace("\"", "\\\"")
                                .replace("\\\"", "\""));
                            
                            edits.push(TextEdit {
                                range: Range {
                                    start: Position { line: line_number, character: start_char },
                                    end: Position { line: line_number, character: end_char },
                                },
                                new_text,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix quotes in range
    fn fix_quotes_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Similar to fix_quotes, but only for string literals in the specified range
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix trailing commas
    fn fix_trailing_commas(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix trailing commas in range
    fn fix_trailing_commas_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix line length
    fn fix_line_length(
        &self,
        document: &Document,
        ast: &AstNode,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix line length in range
    fn fix_line_length_in_range(
        &self,
        document: &Document,
        ast: &AstNode,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Implementation omitted for brevity
        Ok(())
    }
    
    /// Fix trailing whitespace
    fn fix_trailing_whitespace(
        &self,
        document: &Document,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if options.trim_trailing_whitespace {
            for line_number in 0..document.line_count() as u32 {
                if let Some(line) = document.get_line(line_number) {
                    let trimmed_len = line.trim_end().len();
                    
                    if trimmed_len < line.len() {
                        // Remove trailing whitespace
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: line_number, character: trimmed_len as u32 },
                                end: Position { line: line_number, character: line.len() as u32 },
                            },
                            new_text: "".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix trailing whitespace in range
    fn fix_trailing_whitespace_in_range(
        &self,
        document: &Document,
        range: Range,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        if options.trim_trailing_whitespace {
            for line_number in range.start.line..=range.end.line {
                if let Some(line) = document.get_line(line_number) {
                    let trimmed_len = line.trim_end().len();
                    
                    if trimmed_len < line.len() {
                        // Remove trailing whitespace
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: line_number, character: trimmed_len as u32 },
                                end: Position { line: line_number, character: line.len() as u32 },
                            },
                            new_text: "".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix final newline
    fn fix_final_newline(
        &self,
        document: &Document,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        let line_count = document.line_count() as u32;
        
        if line_count > 0 {
            let last_line_number = line_count - 1;
            
            if let Some(last_line) = document.get_line(last_line_number) {
                if options.insert_final_newline {
                    // Check if the document ends with a newline
                    if !last_line.is_empty() {
                        // Add a newline at the end of the document
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: last_line_number, character: last_line.len() as u32 },
                                end: Position { line: last_line_number, character: last_line.len() as u32 },
                            },
                            new_text: "\n".to_string(),
                        });
                    }
                } else if options.trim_final_newlines {
                    // Check if the document ends with empty lines
                    let mut empty_lines = 0;
                    
                    for i in (0..line_count).rev() {
                        if let Some(line) = document.get_line(i) {
                            if line.trim().is_empty() {
                                empty_lines += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    
                    if empty_lines > 0 {
                        // Remove empty lines at the end of the document
                        let start_line = line_count - empty_lines;
                        
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: start_line, character: 0 },
                                end: Position { line: line_count, character: 0 },
                            },
                            new_text: "".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Fix newline indentation
    fn fix_newline_indentation(
        &self,
        document: &Document,
        ast: &AstNode,
        position: Position,
        options: &FormattingOptions,
        edits: &mut Vec<TextEdit>
    ) -> Result<(), String> {
        // Get the previous line
        if position.line > 0 {
            let prev_line_number = position.line - 1;
            
            if let Some(prev_line) = document.get_line(prev_line_number) {
                // Calculate the indentation for the new line
                let mut indent_level = 0;
                
                // Check if the previous line ends with an opening brace
                if prev_line.trim_end().ends_with('{') {
                    // Increase indentation for the new line
                    indent_level = prev_line.chars().take_while(|c| c.is_whitespace()).count() / options.tab_size as usize + 1;
                } else {
                    // Keep the same indentation as the previous line
                    indent_level = prev_line.chars().take_while(|c| c.is_whitespace()).count() / options.tab_size as usize;
                }
                
                // Get the current indentation of the new line
                if let Some(current_line) = document.get_line(position.line) {
                    let current_indent = current_line.chars().take_while(|c| c.is_whitespace()).count();
                    let expected_indent = indent_level * options.tab_size as usize;
                    
                    if current_indent != expected_indent {
                        // Create an edit to fix indentation
                        let indent_str = if options.insert_spaces {
                            " ".repeat(expected_indent)
                        } else {
                            "\t".repeat(indent_level)
                        };
                        
                        edits.push(TextEdit {
                            range: Range {
                                start: Position { line: position.line, character: 0 },
                                end: Position { line: position.line, character: current_indent as u32 },
                            },
                            new_text: indent_str,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Merge overlapping edits
    fn merge_edits(&self, edits: Vec<TextEdit>) -> Vec<TextEdit> {
        // Sort edits by position
        let mut sorted_edits = edits;
        sorted_edits.sort_by(|a, b| {
            if a.range.start.line != b.range.start.line {
                a.range.start.line.cmp(&b.range.start.line)
            } else {
                a.range.start.character.cmp(&b.range.start.character)
            }
        });
        
        // Merge overlapping edits
        let mut merged_edits = Vec::new();
        
        for edit in sorted_edits {
            if let Some(last_edit) = merged_edits.last_mut() {
                if last_edit.range.end.line > edit.range.start.line ||
                   (last_edit.range.end.line == edit.range.start.line &&
                    last_edit.range.end.character >= edit.range.start.character) {
                    // Edits overlap, merge them
                    last_edit.range.end = edit.range.end;
                    last_edit.new_text = format!("{}{}", last_edit.new_text, edit.new_text);
                } else {
                    // No overlap, add the edit
                    merged_edits.push(edit);
                }
            } else {
                // First edit
                merged_edits.push(edit);
            }
        }
        
        merged_edits
    }
}

/// Shared formatting provider that can be used across threads
pub type SharedFormattingProvider = Arc<Mutex<FormattingProvider>>;

/// Create a new shared formatting provider
pub fn create_shared_formatting_provider(default_options: Option<FormattingOptions>) -> SharedFormattingProvider {
    Arc::new(Mutex::new(FormattingProvider::new(default_options)))
}
