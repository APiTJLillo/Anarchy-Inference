// Asset module for Build/Pack Tools
//
// This module provides functionality for managing and bundling assets
// for Anarchy Inference packages.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};

use crate::language_hub_server::build_pack::BuildPackConfig;
use crate::language_hub_server::build_pack::package::Package;

/// Asset type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    /// Binary asset
    #[serde(rename = "binary")]
    Binary,
    
    /// Text asset
    #[serde(rename = "text")]
    Text,
    
    /// Image asset
    #[serde(rename = "image")]
    Image,
    
    /// Audio asset
    #[serde(rename = "audio")]
    Audio,
    
    /// Video asset
    #[serde(rename = "video")]
    Video,
    
    /// Data asset
    #[serde(rename = "data")]
    Data,
    
    /// Configuration asset
    #[serde(rename = "config")]
    Config,
    
    /// Other asset
    #[serde(rename = "other")]
    Other,
}

/// Asset
#[derive(Debug, Clone)]
pub struct Asset {
    /// Asset path
    pub path: PathBuf,
    
    /// Asset type
    pub asset_type: AssetType,
    
    /// Asset size in bytes
    pub size: u64,
    
    /// Asset metadata
    pub metadata: HashMap<String, String>,
}

/// Asset bundle
#[derive(Debug, Clone)]
pub struct AssetBundle {
    /// Assets
    pub assets: Vec<Asset>,
    
    /// Total size in bytes
    pub total_size: u64,
}

/// Asset bundler
pub struct AssetBundler {
    /// Configuration
    config: BuildPackConfig,
}

impl AssetBundler {
    /// Create a new asset bundler
    pub fn new(config: BuildPackConfig) -> Self {
        AssetBundler {
            config,
        }
    }
    
    /// Bundle assets for a package
    pub fn bundle_assets(&self, package: &Package) -> Result<AssetBundle, String> {
        println!("Bundling assets for package: {}", package.metadata.name);
        
        let mut assets = Vec::new();
        let mut total_size = 0;
        
        // Process each asset path
        for asset_path in &package.config.assets {
            let path = package.path.join(asset_path);
            
            if path.is_dir() {
                // Process directory
                self.process_asset_directory(&path, &path, &mut assets, &mut total_size)?;
            } else if path.is_file() {
                // Process file
                let asset = self.process_asset_file(&path)?;
                total_size += asset.size;
                assets.push(asset);
            } else {
                return Err(format!("Asset not found: {}", path.display()));
            }
        }
        
        println!("Bundled {} assets ({} bytes)", assets.len(), total_size);
        
        Ok(AssetBundle {
            assets,
            total_size,
        })
    }
    
    /// Process an asset directory
    fn process_asset_directory(
        &self,
        base_path: &Path,
        dir_path: &Path,
        assets: &mut Vec<Asset>,
        total_size: &mut u64
    ) -> Result<(), String> {
        // Iterate over directory entries
        for entry in fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Process subdirectory
                self.process_asset_directory(base_path, &path, assets, total_size)?;
            } else if path.is_file() {
                // Process file
                let asset = self.process_asset_file(&path)?;
                *total_size += asset.size;
                assets.push(asset);
            }
        }
        
        Ok(())
    }
    
    /// Process an asset file
    fn process_asset_file(&self, path: &Path) -> Result<Asset, String> {
        // Get file metadata
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        
        // Determine asset type
        let asset_type = self.determine_asset_type(path);
        
        // Create asset
        let asset = Asset {
            path: path.to_path_buf(),
            asset_type,
            size: metadata.len(),
            metadata: HashMap::new(),
        };
        
        Ok(asset)
    }
    
    /// Determine asset type
    fn determine_asset_type(&self, path: &Path) -> AssetType {
        if let Some(extension) = path.extension() {
            let extension = extension.to_string_lossy().to_lowercase();
            
            match extension.as_str() {
                // Text files
                "txt" | "md" | "json" | "yaml" | "yml" | "toml" | "xml" | "html" | "css" | "js" => AssetType::Text,
                
                // Image files
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => AssetType::Image,
                
                // Audio files
                "mp3" | "wav" | "ogg" | "flac" | "aac" => AssetType::Audio,
                
                // Video files
                "mp4" | "webm" | "avi" | "mov" | "mkv" => AssetType::Video,
                
                // Data files
                "csv" | "tsv" | "dat" => AssetType::Data,
                
                // Configuration files
                "cfg" | "conf" | "config" | "ini" => AssetType::Config,
                
                // Default to binary
                _ => AssetType::Binary,
            }
        } else {
            AssetType::Binary
        }
    }
    
    /// Copy assets to a directory
    pub fn copy_assets(&self, bundle: &AssetBundle, target_dir: &Path) -> Result<(), String> {
        println!("Copying {} assets to {}", bundle.assets.len(), target_dir.display());
        
        // Create the assets directory
        let assets_dir = target_dir.join("assets");
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create assets directory: {}", e))?;
        
        // Copy each asset
        for asset in &bundle.assets {
            // Determine the target path
            let relative_path = asset.path.file_name()
                .ok_or_else(|| format!("Invalid asset path: {}", asset.path.display()))?;
            
            let target_path = assets_dir.join(relative_path);
            
            // Copy the asset
            fs::copy(&asset.path, &target_path)
                .map_err(|e| format!("Failed to copy asset: {}", e))?;
        }
        
        println!("Assets copied successfully");
        
        Ok(())
    }
    
    /// Get asset metadata
    pub fn get_asset_metadata(&self, asset: &Asset) -> Result<HashMap<String, String>, String> {
        let mut metadata = HashMap::new();
        
        // Add basic metadata
        metadata.insert("path".to_string(), asset.path.to_string_lossy().to_string());
        metadata.insert("size".to_string(), asset.size.to_string());
        metadata.insert("type".to_string(), format!("{:?}", asset.asset_type));
        
        // Add file-specific metadata based on asset type
        match asset.asset_type {
            AssetType::Image => {
                // In a real implementation, this would extract image dimensions, format, etc.
                metadata.insert("format".to_string(), "unknown".to_string());
                metadata.insert("width".to_string(), "0".to_string());
                metadata.insert("height".to_string(), "0".to_string());
            }
            AssetType::Audio => {
                // In a real implementation, this would extract audio duration, bitrate, etc.
                metadata.insert("format".to_string(), "unknown".to_string());
                metadata.insert("duration".to_string(), "0".to_string());
                metadata.insert("bitrate".to_string(), "0".to_string());
            }
            AssetType::Video => {
                // In a real implementation, this would extract video duration, resolution, etc.
                metadata.insert("format".to_string(), "unknown".to_string());
                metadata.insert("duration".to_string(), "0".to_string());
                metadata.insert("width".to_string(), "0".to_string());
                metadata.insert("height".to_string(), "0".to_string());
            }
            _ => {}
        }
        
        Ok(metadata)
    }
    
    /// Generate asset manifest
    pub fn generate_asset_manifest(&self, bundle: &AssetBundle, output_path: &Path) -> Result<(), String> {
        println!("Generating asset manifest: {}", output_path.display());
        
        // Create manifest data
        let mut manifest = HashMap::new();
        manifest.insert("assets".to_string(), bundle.assets.len().to_string());
        manifest.insert("total_size".to_string(), bundle.total_size.to_string());
        
        let mut assets_data = Vec::new();
        for asset in &bundle.assets {
            let metadata = self.get_asset_metadata(asset)?;
            assets_data.push(metadata);
        }
        
        manifest.insert("assets_data".to_string(), serde_json::to_string(&assets_data)
            .map_err(|e| format!("Failed to serialize assets data: {}", e))?);
        
        // Write manifest
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        
        fs::write(output_path, manifest_json)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;
        
        println!("Asset manifest generated successfully");
        
        Ok(())
    }
}
