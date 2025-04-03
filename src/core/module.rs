// src/core/module.rs
// Module system implementation for Anarchy-Inference

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use crate::error::LangError;
use crate::core::value::Value;
use crate::ast::ASTNode;
use crate::parser::Parser;
use crate::lexer::Lexer;

/// Module cache to prevent duplicate loading
#[derive(Debug, Default)]
pub struct ModuleCache {
    /// Map of module paths to loaded modules
    modules: Mutex<HashMap<String, Arc<Module>>>,
}

/// Module representing a single source file
#[derive(Debug)]
pub struct Module {
    /// Name of the module
    name: String,
    /// Path to the module file
    path: PathBuf,
    /// Exported values
    exports: Mutex<HashMap<String, Value>>,
    /// AST nodes for the module
    ast: Vec<ASTNode>,
    /// Dependencies of this module
    dependencies: Vec<String>,
    /// Whether the module has been initialized
    initialized: Mutex<bool>,
}

impl ModuleCache {
    /// Create a new module cache
    pub fn new() -> Self {
        Self {
            modules: Mutex::new(HashMap::new()),
        }
    }
    
    /// Get a module by path, loading it if necessary
    pub fn get_module(&self, path: &str) -> Result<Arc<Module>, LangError> {
        let mut modules = self.modules.lock().unwrap();
        
        // Check if the module is already loaded
        if let Some(module) = modules.get(path) {
            return Ok(module.clone());
        }
        
        // Load the module
        let module = Module::load(path)?;
        let module_arc = Arc::new(module);
        
        // Cache the module
        modules.insert(path.to_string(), module_arc.clone());
        
        Ok(module_arc)
    }
    
    /// Clear the module cache
    pub fn clear(&self) {
        let mut modules = self.modules.lock().unwrap();
        modules.clear();
    }
    
    /// Get all loaded modules
    pub fn get_all_modules(&self) -> Vec<Arc<Module>> {
        let modules = self.modules.lock().unwrap();
        modules.values().cloned().collect()
    }
}

impl Module {
    /// Create a new module with name and empty exports
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path,
            exports: Mutex::new(HashMap::new()),
            ast: Vec::new(),
            dependencies: Vec::new(),
            initialized: Mutex::new(false),
        }
    }

    /// Load a module from a file
    pub fn load(path: &str) -> Result<Self, LangError> {
        // Read the file
        let source = fs::read_to_string(path)
            .map_err(|e| LangError::io_error(&format!("Failed to read module file: {}", e)))?;
        
        // Parse the file
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        // Extract module name from path
        let path_buf = PathBuf::from(path);
        let name = path_buf.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Extract dependencies
        let dependencies = Self::extract_dependencies(&ast);
        
        Ok(Self {
            name,
            path: path_buf,
            exports: Mutex::new(HashMap::new()),
            ast,
            dependencies,
            initialized: Mutex::new(false),
        })
    }
    
    /// Extract dependencies from AST
    fn extract_dependencies(ast: &[ASTNode]) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Scan for import statements
        for node in ast {
            if let Some(dep) = Self::extract_dependency_from_node(node) {
                dependencies.push(dep);
            }
        }
        
        dependencies
    }
    
    /// Extract a dependency from a single AST node
    fn extract_dependency_from_node(_node: &ASTNode) -> Option<String> {
        // This would need to be implemented based on the actual AST structure
        // For now, we'll return None as a placeholder
        None
    }
    
    /// Get the name of the module
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the path of the module
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Get the AST of the module
    pub fn ast(&self) -> &[ASTNode] {
        &self.ast
    }
    
    /// Get the dependencies of the module
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
    
    /// Check if the module has been initialized
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }
    
    /// Mark the module as initialized
    pub fn set_initialized(&self, initialized: bool) {
        let mut init = self.initialized.lock().unwrap();
        *init = initialized;
    }
    
    /// Export a value from the module
    pub fn export(&self, name: &str, value: Value) {
        let mut exports = self.exports.lock().unwrap();
        exports.insert(name.to_string(), value);
    }
    
    /// Get an exported value from the module
    pub fn get_export(&self, name: &str) -> Option<Value> {
        let exports = self.exports.lock().unwrap();
        exports.get(name).cloned()
    }
    
    /// Get all exports from the module
    pub fn get_all_exports(&self) -> HashMap<String, Value> {
        let exports = self.exports.lock().unwrap();
        exports.clone()
    }
}

/// Module resolver for finding modules in the file system
pub struct ModuleResolver {
    /// Base directory for resolving relative paths
    base_dir: PathBuf,
    /// Additional search paths
    search_paths: Vec<PathBuf>,
    /// Module cache
    cache: Arc<ModuleCache>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
            search_paths: Vec::new(),
            cache: Arc::new(ModuleCache::new()),
        }
    }
    
    /// Add a search path
    pub fn add_search_path(&mut self, path: &str) {
        self.search_paths.push(PathBuf::from(path));
    }
    
    /// Resolve a module path to a file path
    pub fn resolve(&self, module_path: &str) -> Result<String, LangError> {
        // Check if the path is absolute
        let path = PathBuf::from(module_path);
        if path.is_absolute() {
            return Ok(path.to_string_lossy().to_string());
        }
        
        // Try to resolve relative to the base directory
        let mut full_path = self.base_dir.clone();
        full_path.push(module_path);
        if full_path.exists() {
            return Ok(full_path.to_string_lossy().to_string());
        }
        
        // Try to resolve using search paths
        for search_path in &self.search_paths {
            let mut full_path = search_path.clone();
            full_path.push(module_path);
            if full_path.exists() {
                return Ok(full_path.to_string_lossy().to_string());
            }
        }
        
        // Add .a.i extension if not present
        if !module_path.ends_with(".a.i") {
            return self.resolve(&format!("{}.a.i", module_path));
        }
        
        Err(LangError::io_error(&format!("Module not found: {}", module_path)))
    }
    
    /// Load a module by path
    pub fn load_module(&self, module_path: &str) -> Result<Arc<Module>, LangError> {
        // Resolve the module path
        let resolved_path = self.resolve(module_path)?;
        
        // Load the module from the cache
        self.cache.get_module(&resolved_path)
    }
    
    /// Get the module cache
    pub fn cache(&self) -> Arc<ModuleCache> {
        self.cache.clone()
    }
}
