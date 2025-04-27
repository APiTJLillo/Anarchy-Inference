// Package module for Build/Pack Tools
//
// This module provides functionality for defining, loading, and managing Anarchy Inference packages.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};

use crate::language_hub_server::build_pack::BuildPackConfig;

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,
    
    /// Package version
    pub version: String,
    
    /// Package description
    pub description: String,
    
    /// Package authors
    pub authors: Vec<String>,
    
    /// Package license
    pub license: String,
    
    /// Package repository
    pub repository: Option<String>,
    
    /// Package homepage
    pub homepage: Option<String>,
    
    /// Package documentation
    pub documentation: Option<String>,
    
    /// Package keywords
    pub keywords: Vec<String>,
    
    /// Package categories
    pub categories: Vec<String>,
}

/// Package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Package metadata
    pub metadata: PackageMetadata,
    
    /// Package dependencies
    pub dependencies: HashMap<String, String>,
    
    /// Development dependencies
    pub dev_dependencies: HashMap<String, String>,
    
    /// Build dependencies
    pub build_dependencies: HashMap<String, String>,
    
    /// Package entry points
    pub entry_points: HashMap<String, String>,
    
    /// Package assets
    pub assets: Vec<String>,
    
    /// Package build configuration
    pub build: BuildConfig,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Target platforms
    pub targets: Vec<String>,
    
    /// Optimization level
    pub optimization: OptimizationLevel,
    
    /// Whether to include debug symbols
    pub debug_symbols: bool,
    
    /// Custom compiler flags
    pub compiler_flags: Vec<String>,
    
    /// Custom linker flags
    pub linker_flags: Vec<String>,
}

/// Optimization level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimization
    #[serde(rename = "none")]
    None,
    
    /// Basic optimization
    #[serde(rename = "basic")]
    Basic,
    
    /// Full optimization
    #[serde(rename = "full")]
    Full,
    
    /// Size optimization
    #[serde(rename = "size")]
    Size,
}

/// Package
#[derive(Debug, Clone)]
pub struct Package {
    /// Package path
    pub path: PathBuf,
    
    /// Package configuration
    pub config: PackageConfig,
    
    /// Package metadata
    pub metadata: PackageMetadata,
}

/// Package manager
pub struct PackageManager {
    /// Configuration
    config: BuildPackConfig,
    
    /// Loaded packages
    loaded_packages: HashMap<String, Package>,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new(config: BuildPackConfig) -> Self {
        PackageManager {
            config,
            loaded_packages: HashMap::new(),
        }
    }
    
    /// Initialize a new package
    pub fn init_package(&self, name: &str, path: &Path) -> Result<Package, String> {
        // Check if the directory exists
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        // Check if the directory is empty
        if path.read_dir().map_err(|e| format!("Failed to read directory: {}", e))?.next().is_some() {
            return Err("Directory is not empty".to_string());
        }
        
        // Create package structure
        self.create_package_structure(path)?;
        
        // Create default package configuration
        let config = self.create_default_config(name);
        
        // Write package configuration
        self.write_package_config(path, &config)?;
        
        // Create the package
        let package = Package {
            path: path.to_path_buf(),
            config: config.clone(),
            metadata: config.metadata,
        };
        
        Ok(package)
    }
    
    /// Create package structure
    fn create_package_structure(&self, path: &Path) -> Result<(), String> {
        // Create directories
        for dir in &["src", "tests", "assets", "build"] {
            fs::create_dir_all(path.join(dir))
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
        }
        
        // Create basic files
        let main_file = path.join("src").join("main.a.i");
        fs::write(&main_file, b"// Main entry point\n\nm{\n  main() {\n    print(\"Hello, Anarchy Inference!\");\n    return 0;\n  }\n}\n")
            .map_err(|e| format!("Failed to create main file: {}", e))?;
        
        let test_file = path.join("tests").join("test_main.a.i");
        fs::write(&test_file, b"// Test file\n\nm{\n  test_main() {\n    assert(true, \"This test should pass\");\n    return 0;\n  }\n}\n")
            .map_err(|e| format!("Failed to create test file: {}", e))?;
        
        let readme_file = path.join("README.md");
        fs::write(&readme_file, b"# Anarchy Inference Package\n\nThis is an Anarchy Inference package.\n")
            .map_err(|e| format!("Failed to create README file: {}", e))?;
        
        let gitignore_file = path.join(".gitignore");
        fs::write(&gitignore_file, b"/build\n/.anarchy-cache\n")
            .map_err(|e| format!("Failed to create .gitignore file: {}", e))?;
        
        Ok(())
    }
    
    /// Create default package configuration
    fn create_default_config(&self, name: &str) -> PackageConfig {
        PackageConfig {
            metadata: PackageMetadata {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: "An Anarchy Inference package".to_string(),
                authors: vec!["Your Name <your.email@example.com>".to_string()],
                license: "MIT".to_string(),
                repository: None,
                homepage: None,
                documentation: None,
                keywords: vec![],
                categories: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build_dependencies: HashMap::new(),
            entry_points: {
                let mut entry_points = HashMap::new();
                entry_points.insert("main".to_string(), "src/main.a.i".to_string());
                entry_points
            },
            assets: vec![],
            build: BuildConfig {
                targets: vec!["native".to_string()],
                optimization: OptimizationLevel::Basic,
                debug_symbols: true,
                compiler_flags: vec![],
                linker_flags: vec![],
            },
        }
    }
    
    /// Write package configuration
    fn write_package_config(&self, path: &Path, config: &PackageConfig) -> Result<(), String> {
        let config_path = path.join("anarchy-package.json");
        
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize package configuration: {}", e))?;
        
        fs::write(&config_path, config_json)
            .map_err(|e| format!("Failed to write package configuration: {}", e))?;
        
        Ok(())
    }
    
    /// Load a package
    pub fn load_package(&self, path: &Path) -> Result<Package, String> {
        // Check if the directory exists
        if !path.exists() {
            return Err(format!("Directory does not exist: {}", path.display()));
        }
        
        // Check if the package configuration exists
        let config_path = path.join("anarchy-package.json");
        if !config_path.exists() {
            return Err(format!("Package configuration not found: {}", config_path.display()));
        }
        
        // Read the package configuration
        let config_json = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read package configuration: {}", e))?;
        
        // Parse the package configuration
        let config: PackageConfig = serde_json::from_str(&config_json)
            .map_err(|e| format!("Failed to parse package configuration: {}", e))?;
        
        // Create the package
        let package = Package {
            path: path.to_path_buf(),
            config: config.clone(),
            metadata: config.metadata,
        };
        
        Ok(package)
    }
    
    /// Save a package
    pub fn save_package(&self, package: &Package) -> Result<(), String> {
        // Write the package configuration
        self.write_package_config(&package.path, &package.config)?;
        
        Ok(())
    }
    
    /// Get package dependencies
    pub fn get_dependencies(&self, package: &Package, include_dev: bool) -> HashMap<String, String> {
        let mut dependencies = package.config.dependencies.clone();
        
        if include_dev {
            for (name, version) in &package.config.dev_dependencies {
                dependencies.insert(name.clone(), version.clone());
            }
        }
        
        dependencies
    }
    
    /// Validate a package
    pub fn validate_package(&self, package: &Package) -> Result<(), String> {
        // Check if the package name is valid
        if package.metadata.name.is_empty() {
            return Err("Package name cannot be empty".to_string());
        }
        
        // Check if the package version is valid
        if package.metadata.version.is_empty() {
            return Err("Package version cannot be empty".to_string());
        }
        
        // Check if the entry points exist
        for (name, path) in &package.config.entry_points {
            let entry_point_path = package.path.join(path);
            if !entry_point_path.exists() {
                return Err(format!("Entry point not found: {}", entry_point_path.display()));
            }
        }
        
        // Check if the assets exist
        for asset in &package.config.assets {
            let asset_path = package.path.join(asset);
            if !asset_path.exists() {
                return Err(format!("Asset not found: {}", asset_path.display()));
            }
        }
        
        Ok(())
    }
    
    /// List packages in a directory
    pub fn list_packages(&self, dir: &Path) -> Result<Vec<Package>, String> {
        let mut packages = Vec::new();
        
        // Check if the directory exists
        if !dir.exists() {
            return Err(format!("Directory does not exist: {}", dir.display()));
        }
        
        // Iterate over subdirectories
        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Check if the subdirectory contains a package configuration
                let config_path = path.join("anarchy-package.json");
                if config_path.exists() {
                    // Load the package
                    match self.load_package(&path) {
                        Ok(package) => packages.push(package),
                        Err(e) => eprintln!("Warning: Failed to load package at {}: {}", path.display(), e),
                    }
                }
            }
        }
        
        Ok(packages)
    }
    
    /// Search for packages
    pub fn search_packages(&self, query: &str) -> Result<Vec<PackageMetadata>, String> {
        // This is a simplified implementation
        // In a real implementation, this would search a package registry
        
        // For now, just return an empty list
        Ok(Vec::new())
    }
}
