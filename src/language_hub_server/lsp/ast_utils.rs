// AST traversal utilities module for LSP-like Component
//
// This module provides utilities for traversing and manipulating
// Abstract Syntax Trees (ASTs) for Anarchy Inference code.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range};
use crate::language_hub_server::lsp::parser_integration::AstNode;

/// AST visitor trait
pub trait AstVisitor {
    /// Visit an AST node before visiting its children
    fn visit_enter(&mut self, node: &AstNode) -> bool;
    
    /// Visit an AST node after visiting its children
    fn visit_leave(&mut self, node: &AstNode);
}

/// AST traversal function
pub fn traverse_ast<V: AstVisitor>(visitor: &mut V, node: &AstNode) {
    // Visit the node
    let should_visit_children = visitor.visit_enter(node);
    
    // Visit children if requested
    if should_visit_children {
        for child in &node.children {
            traverse_ast(visitor, child);
        }
    }
    
    // Visit the node again after children
    visitor.visit_leave(node);
}

/// AST node finder
pub struct AstNodeFinder {
    /// The position to find
    position: Position,
    
    /// The found node
    found_node: Option<AstNode>,
    
    /// The deepest node that contains the position
    deepest_node: Option<AstNode>,
    
    /// The current depth
    depth: usize,
    
    /// The maximum depth found
    max_depth: usize,
}

impl AstNodeFinder {
    /// Create a new AST node finder
    pub fn new(position: Position) -> Self {
        AstNodeFinder {
            position,
            found_node: None,
            deepest_node: None,
            depth: 0,
            max_depth: 0,
        }
    }
    
    /// Find a node at the specified position
    pub fn find(position: Position, root: &AstNode) -> Option<AstNode> {
        let mut finder = AstNodeFinder::new(position);
        traverse_ast(&mut finder, root);
        finder.found_node.or(finder.deepest_node)
    }
    
    /// Get the found node
    pub fn get_found_node(&self) -> Option<&AstNode> {
        self.found_node.as_ref()
    }
    
    /// Get the deepest node that contains the position
    pub fn get_deepest_node(&self) -> Option<&AstNode> {
        self.deepest_node.as_ref()
    }
}

impl AstVisitor for AstNodeFinder {
    fn visit_enter(&mut self, node: &AstNode) -> bool {
        self.depth += 1;
        
        // Check if the node contains the position
        if position_in_range(self.position, &node.range) {
            // Update the deepest node if this is deeper
            if self.depth > self.max_depth {
                self.max_depth = self.depth;
                self.deepest_node = Some(node.clone());
            }
            
            // If this is a leaf node or a specific node type we're looking for,
            // set it as the found node
            if node.children.is_empty() || is_identifier_or_literal(node) {
                self.found_node = Some(node.clone());
            }
            
            // Continue traversal
            return true;
        }
        
        // Position is not in this node, skip its children
        false
    }
    
    fn visit_leave(&mut self, _node: &AstNode) {
        self.depth -= 1;
    }
}

/// AST node collector
pub struct AstNodeCollector {
    /// The predicate to match nodes
    predicate: Box<dyn Fn(&AstNode) -> bool>,
    
    /// The collected nodes
    collected_nodes: Vec<AstNode>,
}

impl AstNodeCollector {
    /// Create a new AST node collector
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&AstNode) -> bool + 'static,
    {
        AstNodeCollector {
            predicate: Box::new(predicate),
            collected_nodes: Vec::new(),
        }
    }
    
    /// Collect nodes that match the predicate
    pub fn collect<F>(predicate: F, root: &AstNode) -> Vec<AstNode>
    where
        F: Fn(&AstNode) -> bool + 'static,
    {
        let mut collector = AstNodeCollector::new(predicate);
        traverse_ast(&mut collector, root);
        collector.collected_nodes
    }
    
    /// Get the collected nodes
    pub fn get_collected_nodes(&self) -> &[AstNode] {
        &self.collected_nodes
    }
}

impl AstVisitor for AstNodeCollector {
    fn visit_enter(&mut self, node: &AstNode) -> bool {
        // Check if the node matches the predicate
        if (self.predicate)(node) {
            self.collected_nodes.push(node.clone());
        }
        
        // Always continue traversal
        true
    }
    
    fn visit_leave(&mut self, _node: &AstNode) {
        // Nothing to do
    }
}

/// AST node transformer
pub struct AstNodeTransformer {
    /// The transformation function
    transform: Box<dyn Fn(&AstNode) -> Option<AstNode>>,
    
    /// The transformed AST
    transformed_ast: Option<AstNode>,
}

impl AstNodeTransformer {
    /// Create a new AST node transformer
    pub fn new<F>(transform: F) -> Self
    where
        F: Fn(&AstNode) -> Option<AstNode> + 'static,
    {
        AstNodeTransformer {
            transform: Box::new(transform),
            transformed_ast: None,
        }
    }
    
    /// Transform an AST
    pub fn transform<F>(transform: F, root: &AstNode) -> AstNode
    where
        F: Fn(&AstNode) -> Option<AstNode> + 'static,
    {
        let mut transformer = AstNodeTransformer::new(transform);
        transformer.transformed_ast = Some(transformer.transform_node(root));
        transformer.transformed_ast.unwrap()
    }
    
    /// Transform a node and its children
    fn transform_node(&self, node: &AstNode) -> AstNode {
        // Transform children first
        let transformed_children: Vec<AstNode> = node.children.iter()
            .map(|child| self.transform_node(child))
            .collect();
        
        // Create a new node with transformed children
        let mut transformed_node = node.clone();
        transformed_node.children = transformed_children;
        
        // Apply the transformation function
        if let Some(result) = (self.transform)(&transformed_node) {
            result
        } else {
            transformed_node
        }
    }
}

/// AST path finder
pub struct AstPathFinder {
    /// The position to find
    position: Position,
    
    /// The path to the node
    path: Vec<AstNode>,
    
    /// The current path during traversal
    current_path: Vec<AstNode>,
}

impl AstPathFinder {
    /// Create a new AST path finder
    pub fn new(position: Position) -> Self {
        AstPathFinder {
            position,
            path: Vec::new(),
            current_path: Vec::new(),
        }
    }
    
    /// Find the path to a node at the specified position
    pub fn find_path(position: Position, root: &AstNode) -> Vec<AstNode> {
        let mut finder = AstPathFinder::new(position);
        traverse_ast(&mut finder, root);
        finder.path
    }
    
    /// Get the path to the node
    pub fn get_path(&self) -> &[AstNode] {
        &self.path
    }
}

impl AstVisitor for AstPathFinder {
    fn visit_enter(&mut self, node: &AstNode) -> bool {
        // Add the node to the current path
        self.current_path.push(node.clone());
        
        // Check if the node contains the position
        if position_in_range(self.position, &node.range) {
            // If this is a leaf node or a specific node type we're looking for,
            // save the current path
            if node.children.is_empty() || is_identifier_or_literal(node) {
                self.path = self.current_path.clone();
            }
            
            // Continue traversal
            return true;
        }
        
        // Position is not in this node, skip its children
        false
    }
    
    fn visit_leave(&mut self, _node: &AstNode) {
        // Remove the node from the current path
        self.current_path.pop();
    }
}

/// AST parent finder
pub struct AstParentFinder {
    /// The node to find the parent of
    target_node: AstNode,
    
    /// The parent node
    parent_node: Option<AstNode>,
    
    /// The current parent during traversal
    current_parent: Option<AstNode>,
}

impl AstParentFinder {
    /// Create a new AST parent finder
    pub fn new(target_node: AstNode) -> Self {
        AstParentFinder {
            target_node,
            parent_node: None,
            current_parent: None,
        }
    }
    
    /// Find the parent of a node
    pub fn find_parent(target_node: &AstNode, root: &AstNode) -> Option<AstNode> {
        let mut finder = AstParentFinder::new(target_node.clone());
        traverse_ast(&mut finder, root);
        finder.parent_node
    }
    
    /// Get the parent node
    pub fn get_parent_node(&self) -> Option<&AstNode> {
        self.parent_node.as_ref()
    }
}

impl AstVisitor for AstParentFinder {
    fn visit_enter(&mut self, node: &AstNode) -> bool {
        // Check if this node is the target
        if node_equals(node, &self.target_node) {
            // Save the current parent
            self.parent_node = self.current_parent.clone();
            // No need to traverse further
            return false;
        }
        
        // Save the current node as the parent for its children
        let previous_parent = self.current_parent.clone();
        self.current_parent = Some(node.clone());
        
        // Continue traversal
        true
    }
    
    fn visit_leave(&mut self, node: &AstNode) {
        // Restore the previous parent
        if let Some(parent) = &self.current_parent {
            if node_equals(node, parent) {
                self.current_parent = None;
            }
        }
    }
}

/// AST scope finder
pub struct AstScopeFinder {
    /// The position to find the scope for
    position: Position,
    
    /// The scope node
    scope_node: Option<AstNode>,
    
    /// The current scope during traversal
    current_scope: Option<AstNode>,
}

impl AstScopeFinder {
    /// Create a new AST scope finder
    pub fn new(position: Position) -> Self {
        AstScopeFinder {
            position,
            scope_node: None,
            current_scope: None,
        }
    }
    
    /// Find the scope at the specified position
    pub fn find_scope(position: Position, root: &AstNode) -> Option<AstNode> {
        let mut finder = AstScopeFinder::new(position);
        traverse_ast(&mut finder, root);
        finder.scope_node
    }
    
    /// Get the scope node
    pub fn get_scope_node(&self) -> Option<&AstNode> {
        self.scope_node.as_ref()
    }
}

impl AstVisitor for AstScopeFinder {
    fn visit_enter(&mut self, node: &AstNode) -> bool {
        // Check if the node is a scope node
        if is_scope_node(node) {
            // Check if the node contains the position
            if position_in_range(self.position, &node.range) {
                // Save the current scope
                self.current_scope = Some(node.clone());
                self.scope_node = Some(node.clone());
                
                // Continue traversal to find nested scopes
                return true;
            }
        } else if position_in_range(self.position, &node.range) {
            // Continue traversal
            return true;
        }
        
        // Position is not in this node, skip its children
        false
    }
    
    fn visit_leave(&mut self, node: &AstNode) {
        // Restore the previous scope
        if let Some(scope) = &self.current_scope {
            if node_equals(node, scope) {
                self.current_scope = None;
            }
        }
    }
}

/// AST symbol finder
pub struct AstSymbolFinder {
    /// The symbol name to find
    symbol_name: String,
    
    /// The found symbol nodes
    symbol_nodes: Vec<AstNode>,
}

impl AstSymbolFinder {
    /// Create a new AST symbol finder
    pub fn new(symbol_name: &str) -> Self {
        AstSymbolFinder {
            symbol_name: symbol_name.to_string(),
            symbol_nodes: Vec::new(),
        }
    }
    
    /// Find all occurrences of a symbol
    pub fn find_symbols(symbol_name: &str, root: &AstNode) -> Vec<AstNode> {
        let mut finder = AstSymbolFinder::new(symbol_name);
        traverse_ast(&mut finder, root);
        finder.symbol_nodes
    }
    
    /// Get the found symbol nodes
    pub fn get_symbol_nodes(&self) -> &[AstNode] {
        &self.symbol_nodes
    }
}

impl AstVisitor for AstSymbolFinder {
    fn visit_enter(&mut self, node: &AstNode) -> bool {
        // Check if the node is an identifier with the target name
        if node.node_type == "Identifier" {
            if let Some(name) = node.properties.get("name") {
                if let Some(name_str) = name.as_str() {
                    if name_str == self.symbol_name {
                        self.symbol_nodes.push(node.clone());
                    }
                }
            }
        }
        
        // Continue traversal
        true
    }
    
    fn visit_leave(&mut self, _node: &AstNode) {
        // Nothing to do
    }
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

/// Check if a node is an identifier or literal
fn is_identifier_or_literal(node: &AstNode) -> bool {
    node.node_type == "Identifier" || node.node_type == "Literal"
}

/// Check if a node is a scope node
fn is_scope_node(node: &AstNode) -> bool {
    match node.node_type.as_str() {
        "Program" | "ModuleDeclaration" | "FunctionDeclaration" | "BlockStatement" => true,
        _ => false,
    }
}

/// Check if two nodes are equal
fn node_equals(a: &AstNode, b: &AstNode) -> bool {
    // Check node type
    if a.node_type != b.node_type {
        return false;
    }
    
    // Check range
    if a.range.start.line != b.range.start.line || 
       a.range.start.character != b.range.start.character ||
       a.range.end.line != b.range.end.line || 
       a.range.end.character != b.range.end.character {
        return false;
    }
    
    // For simplicity, we'll consider nodes equal if they have the same type and range
    // In a real implementation, we might want to check properties and children as well
    
    true
}

/// AST utilities
pub struct AstUtils;

impl AstUtils {
    /// Find a node at a specific position
    pub fn find_node_at_position(root: &AstNode, position: Position) -> Option<AstNode> {
        AstNodeFinder::find(position, root)
    }
    
    /// Find the path to a node at a specific position
    pub fn find_path_to_position(root: &AstNode, position: Position) -> Vec<AstNode> {
        AstPathFinder::find_path(position, root)
    }
    
    /// Find the parent of a node
    pub fn find_parent(root: &AstNode, node: &AstNode) -> Option<AstNode> {
        AstParentFinder::find_parent(node, root)
    }
    
    /// Find the scope at a specific position
    pub fn find_scope_at_position(root: &AstNode, position: Position) -> Option<AstNode> {
        AstScopeFinder::find_scope(position, root)
    }
    
    /// Find all occurrences of a symbol
    pub fn find_symbol_occurrences(root: &AstNode, symbol_name: &str) -> Vec<AstNode> {
        AstSymbolFinder::find_symbols(symbol_name, root)
    }
    
    /// Collect nodes that match a predicate
    pub fn collect_nodes<F>(root: &AstNode, predicate: F) -> Vec<AstNode>
    where
        F: Fn(&AstNode) -> bool + 'static,
    {
        AstNodeCollector::collect(predicate, root)
    }
    
    /// Transform an AST
    pub fn transform_ast<F>(root: &AstNode, transform: F) -> AstNode
    where
        F: Fn(&AstNode) -> Option<AstNode> + 'static,
    {
        AstNodeTransformer::transform(transform, root)
    }
    
    /// Get all identifiers in an AST
    pub fn get_all_identifiers(root: &AstNode) -> Vec<AstNode> {
        Self::collect_nodes(root, |node| node.node_type == "Identifier")
    }
    
    /// Get all function declarations in an AST
    pub fn get_all_function_declarations(root: &AstNode) -> Vec<AstNode> {
        Self::collect_nodes(root, |node| node.node_type == "FunctionDeclaration")
    }
    
    /// Get all variable declarations in an AST
    pub fn get_all_variable_declarations(root: &AstNode) -> Vec<AstNode> {
        Self::collect_nodes(root, |node| node.node_type == "VariableDeclaration")
    }
    
    /// Get all module declarations in an AST
    pub fn get_all_module_declarations(root: &AstNode) -> Vec<AstNode> {
        Self::collect_nodes(root, |node| node.node_type == "ModuleDeclaration")
    }
    
    /// Get the definition of a symbol at a specific position
    pub fn get_symbol_definition(root: &AstNode, symbol_name: &str) -> Option<AstNode> {
        // Find all variable and function declarations
        let declarations = Self::collect_nodes(root, |node| {
            (node.node_type == "VariableDeclaration" || node.node_type == "FunctionDeclaration") &&
            node.properties.get("name").and_then(|v| v.as_str()) == Some(symbol_name)
        });
        
        // Return the first declaration found
        declarations.into_iter().next()
    }
    
    /// Get all references to a symbol
    pub fn get_symbol_references(root: &AstNode, symbol_name: &str) -> Vec<AstNode> {
        Self::collect_nodes(root, |node| {
            node.node_type == "Identifier" &&
            node.properties.get("name").and_then(|v| v.as_str()) == Some(symbol_name)
        })
    }
}

/// Shared AST utilities that can be used across threads
pub type SharedAstUtils = Arc<AstUtils>;

/// Create a new shared AST utilities
pub fn create_shared_ast_utils() -> SharedAstUtils {
    Arc::new(AstUtils)
}
