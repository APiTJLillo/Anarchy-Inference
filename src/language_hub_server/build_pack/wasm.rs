// WebAssembly compilation module for Build/Pack Tools
//
// This module provides functionality for compiling Anarchy Inference packages
// to WebAssembly (WASM) for use in browsers and other WASM-compatible environments.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::process::Command;

use crate::language_hub_server::build_pack::BuildPackConfig;
use crate::language_hub_server::build_pack::package::Package;

/// WASM target
#[derive(Debug, Clone)]
pub enum WasmTarget {
    /// Browser target
    Browser,
    
    /// Node.js target
    NodeJs,
    
    /// Generic target
    Generic,
}

/// WASM compilation options
#[derive(Debug, Clone)]
pub struct WasmCompilationOptions {
    /// Target
    pub target: WasmTarget,
    
    /// Optimization level (0-3)
    pub optimization_level: u8,
    
    /// Whether to include debug information
    pub debug_info: bool,
    
    /// Whether to generate TypeScript definitions
    pub typescript_defs: bool,
    
    /// Additional features to enable
    pub features: Vec<String>,
}

impl Default for WasmCompilationOptions {
    fn default() -> Self {
        WasmCompilationOptions {
            target: WasmTarget::Browser,
            optimization_level: 2,
            debug_info: false,
            typescript_defs: true,
            features: Vec::new(),
        }
    }
}

/// WASM compilation result
#[derive(Debug, Clone)]
pub struct WasmCompilationResult {
    /// Output directory
    pub output_dir: PathBuf,
    
    /// WASM file
    pub wasm_file: PathBuf,
    
    /// JavaScript file
    pub js_file: PathBuf,
    
    /// TypeScript definitions file (if generated)
    pub ts_defs_file: Option<PathBuf>,
    
    /// Size of WASM file in bytes
    pub wasm_size: u64,
}

/// WASM compiler
pub struct WasmCompiler {
    /// Configuration
    config: BuildPackConfig,
}

impl WasmCompiler {
    /// Create a new WASM compiler
    pub fn new(config: BuildPackConfig) -> Self {
        WasmCompiler {
            config,
        }
    }
    
    /// Compile package to WASM
    pub fn compile(&self, package: &Package, options: WasmCompilationOptions) -> Result<WasmCompilationResult, String> {
        println!("Compiling package {} to WebAssembly", package.metadata.name);
        
        // Create output directory
        let output_dir = package.path.join("target").join("wasm");
        fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        
        // Create WASM-specific package
        self.create_wasm_package(package, &output_dir, &options)?;
        
        // Run wasm-pack
        self.run_wasm_pack(package, &output_dir, &options)?;
        
        // Collect compilation results
        let result = self.collect_compilation_results(package, &output_dir, &options)?;
        
        println!("WebAssembly compilation completed successfully");
        println!("WASM size: {} bytes", result.wasm_size);
        
        Ok(result)
    }
    
    /// Create WASM-specific package
    fn create_wasm_package(&self, package: &Package, output_dir: &Path, options: &WasmCompilationOptions) -> Result<(), String> {
        // Create src directory
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src directory: {}", e))?;
        
        // Create lib.rs
        let lib_rs_content = self.generate_wasm_lib_rs(package, options);
        fs::write(src_dir.join("lib.rs"), lib_rs_content)
            .map_err(|e| format!("Failed to write lib.rs: {}", e))?;
        
        // Create Cargo.toml
        let cargo_toml_content = self.generate_wasm_cargo_toml(package, options);
        fs::write(output_dir.join("Cargo.toml"), cargo_toml_content)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        
        // Copy modules
        let modules_dir = output_dir.join("modules");
        fs::create_dir_all(&modules_dir)
            .map_err(|e| format!("Failed to create modules directory: {}", e))?;
        
        for module_path in &package.config.modules {
            let src_path = package.path.join(module_path);
            let dst_path = modules_dir.join(src_path.file_name().unwrap());
            
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy module {}: {}", src_path.display(), e))?;
        }
        
        Ok(())
    }
    
    /// Generate WASM lib.rs
    fn generate_wasm_lib_rs(&self, package: &Package, options: &WasmCompilationOptions) -> String {
        let mut content = String::new();
        
        // Add imports
        content.push_str("use wasm_bindgen::prelude::*;\n");
        content.push_str("use std::path::Path;\n");
        
        if matches!(options.target, WasmTarget::Browser) {
            content.push_str("use web_sys::console;\n");
        }
        
        content.push_str("\n");
        
        // Add module doc
        content.push_str(&format!("/// {} WebAssembly module\n", package.metadata.name));
        content.push_str("///\n");
        content.push_str(&format!("/// This module provides WebAssembly bindings for the {} package.\n", package.metadata.name));
        content.push_str("#[wasm_bindgen]\n");
        
        // Add struct
        let struct_name = self.to_camel_case(&package.metadata.name);
        content.push_str(&format!("pub struct {} {{\n", struct_name));
        content.push_str("    /// Runtime instance\n");
        content.push_str("    runtime: anarchy_inference::Runtime,\n");
        content.push_str("}\n\n");
        
        // Add implementation
        content.push_str("#[wasm_bindgen]\n");
        content.push_str(&format!("impl {} {{\n", struct_name));
        
        // Constructor
        content.push_str("    /// Create a new runtime instance\n");
        content.push_str("    #[wasm_bindgen(constructor)]\n");
        content.push_str(&format!("    pub fn new() -> Result<{}, JsValue> {{\n", struct_name));
        content.push_str("        // Initialize runtime\n");
        content.push_str("        let runtime = anarchy_inference::Runtime::new()\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Failed to create runtime: {}\", e)))?;\n");
        content.push_str("\n");
        content.push_str(&format!("        Ok({} {{ runtime }})\n", struct_name));
        content.push_str("    }\n\n");
        
        // Load module
        content.push_str("    /// Load a module\n");
        content.push_str("    #[wasm_bindgen]\n");
        content.push_str("    pub fn load_module(&mut self, name: &str, path: &str) -> Result<(), JsValue> {\n");
        content.push_str("        // Load the module\n");
        content.push_str("        self.runtime.load_module(name, Path::new(path))\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Failed to load module: {}\", e)))\n");
        content.push_str("    }\n\n");
        
        // Call function
        content.push_str("    /// Call a function\n");
        content.push_str("    #[wasm_bindgen]\n");
        content.push_str("    pub fn call_function(&self, module_name: &str, function_name: &str, args: &JsValue) -> Result<JsValue, JsValue> {\n");
        content.push_str("        // Convert JS args to Anarchy values\n");
        content.push_str("        let args: Vec<anarchy_inference::Value> = serde_wasm_bindgen::from_value(args.clone())\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Failed to parse arguments: {}\", e)))?;\n");
        content.push_str("\n");
        content.push_str("        // Call the function\n");
        content.push_str("        let result = self.runtime.call_function(module_name, function_name, &args)\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Function call error: {}\", e)))?;\n");
        content.push_str("\n");
        content.push_str("        // Convert result to JS value\n");
        content.push_str("        serde_wasm_bindgen::to_value(&result)\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Serialization error: {}\", e)))\n");
        content.push_str("    }\n\n");
        
        // Eval
        content.push_str("    /// Evaluate code\n");
        content.push_str("    #[wasm_bindgen]\n");
        content.push_str("    pub fn eval(&self, code: &str) -> Result<JsValue, JsValue> {\n");
        content.push_str("        // Evaluate the code\n");
        content.push_str("        let result = self.runtime.eval(code)\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Evaluation error: {}\", e)))?;\n");
        content.push_str("\n");
        content.push_str("        // Convert result to JS value\n");
        content.push_str("        serde_wasm_bindgen::to_value(&result)\n");
        content.push_str("            .map_err(|e| JsValue::from_str(&format!(\"Serialization error: {}\", e)))\n");
        content.push_str("    }\n");
        
        // Close implementation
        content.push_str("}\n\n");
        
        // Add initialization function
        content.push_str("// Initialize the WASM module\n");
        content.push_str("#[wasm_bindgen(start)]\n");
        content.push_str("pub fn init() {\n");
        
        if matches!(options.target, WasmTarget::Browser) {
            content.push_str("    // Set up panic hook for better error messages\n");
            content.push_str("    console_error_panic_hook::set_once();\n");
            content.push_str("\n");
            content.push_str("    // Log initialization\n");
            content.push_str("    console::log_1(&JsValue::from_str(\"Anarchy Inference WASM module initialized\"));\n");
        } else {
            content.push_str("    // Set up panic hook for better error messages\n");
            content.push_str("    console_error_panic_hook::set_once();\n");
        }
        
        content.push_str("}\n");
        
        content
    }
    
    /// Generate WASM Cargo.toml
    fn generate_wasm_cargo_toml(&self, package: &Package, options: &WasmCompilationOptions) -> String {
        let mut content = String::new();
        
        // Package section
        content.push_str("[package]\n");
        content.push_str(&format!("name = \"{}-wasm\"\n", package.metadata.name));
        content.push_str(&format!("version = \"{}\"\n", package.metadata.version));
        content.push_str(&format!("description = \"WebAssembly bindings for {}\"\n", package.metadata.name));
        content.push_str(&format!("authors = [{}]\n", package.metadata.authors.iter().map(|a| format!("\"{}\"", a)).collect::<Vec<_>>().join(", ")));
        content.push_str("edition = \"2021\"\n\n");
        
        // Lib section
        content.push_str("[lib]\n");
        content.push_str("crate-type = [\"cdylib\"]\n\n");
        
        // Dependencies
        content.push_str("[dependencies]\n");
        content.push_str(&format!("anarchy_inference = {{ path = \"../..\" }}\n"));
        content.push_str("wasm-bindgen = \"0.2\"\n");
        content.push_str("serde = { version = \"1.0\", features = [\"derive\"] }\n");
        content.push_str("serde-wasm-bindgen = \"0.4\"\n");
        content.push_str("console_error_panic_hook = \"0.1\"\n");
        
        // Target-specific dependencies
        match options.target {
            WasmTarget::Browser => {
                content.push_str("\n[dependencies.web-sys]\n");
                content.push_str("version = \"0.3\"\n");
                content.push_str("features = [\n");
                content.push_str("  \"console\",\n");
                content.push_str("  \"Window\",\n");
                content.push_str("  \"Document\",\n");
                content.push_str("  \"Element\",\n");
                content.push_str("  \"HtmlElement\",\n");
                content.push_str("  \"Node\",\n");
                content.push_str("  \"Performance\",\n");
                content.push_str("]\n");
            }
            WasmTarget::NodeJs => {
                content.push_str("\n[dependencies.js-sys]\n");
                content.push_str("version = \"0.3\"\n");
            }
            _ => {}
        }
        
        // Profile settings
        content.push_str("\n[profile.release]\n");
        content.push_str(&format!("opt-level = {}\n", options.optimization_level));
        content.push_str("lto = true\n");
        
        if !options.debug_info {
            content.push_str("debug = false\n");
        }
        
        content
    }
    
    /// Run wasm-pack
    fn run_wasm_pack(&self, package: &Package, output_dir: &Path, options: &WasmCompilationOptions) -> Result<(), String> {
        // Determine target
        let target = match options.target {
            WasmTarget::Browser => "web",
            WasmTarget::NodeJs => "nodejs",
            WasmTarget::Generic => "bundler",
        };
        
        // Build command
        let mut cmd = Command::new("wasm-pack");
        cmd.current_dir(output_dir)
            .arg("build")
            .arg("--target")
            .arg(target);
        
        // Add optimization level
        if options.optimization_level > 0 {
            cmd.arg("--release");
        } else {
            cmd.arg("--dev");
        }
        
        // Add TypeScript option
        if options.typescript_defs {
            cmd.arg("--typescript");
        }
        
        // Run the command
        let output = cmd.output()
            .map_err(|e| format!("Failed to run wasm-pack: {}", e))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("wasm-pack failed: {}", error));
        }
        
        Ok(())
    }
    
    /// Collect compilation results
    fn collect_compilation_results(&self, package: &Package, output_dir: &Path, options: &WasmCompilationOptions) -> Result<WasmCompilationResult, String> {
        // Determine pkg directory
        let pkg_dir = output_dir.join("pkg");
        
        // Find WASM file
        let wasm_file = pkg_dir.join(format!("{}_wasm_bg.wasm", package.metadata.name.replace("-", "_")));
        if !wasm_file.exists() {
            return Err(format!("WASM file not found: {}", wasm_file.display()));
        }
        
        // Find JS file
        let js_file = pkg_dir.join(format!("{}_wasm.js", package.metadata.name.replace("-", "_")));
        if !js_file.exists() {
            return Err(format!("JavaScript file not found: {}", js_file.display()));
        }
        
        // Find TypeScript definitions file
        let ts_defs_file = if options.typescript_defs {
            let file = pkg_dir.join(format!("{}_wasm.d.ts", package.metadata.name.replace("-", "_")));
            if file.exists() {
                Some(file)
            } else {
                None
            }
        } else {
            None
        };
        
        // Get WASM file size
        let wasm_size = fs::metadata(&wasm_file)
            .map_err(|e| format!("Failed to get WASM file metadata: {}", e))?
            .len();
        
        Ok(WasmCompilationResult {
            output_dir: pkg_dir,
            wasm_file,
            js_file,
            ts_defs_file,
            wasm_size,
        })
    }
    
    /// Convert string to camel case
    fn to_camel_case(&self, s: &str) -> String {
        let mut camel_case = String::new();
        let mut capitalize_next = true;
        
        for c in s.chars() {
            if c == '-' || c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                camel_case.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                camel_case.push(c);
            }
        }
        
        camel_case
    }
    
    /// Check if wasm-pack is installed
    pub fn check_wasm_pack_installed(&self) -> bool {
        Command::new("wasm-pack")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Install wasm-pack
    pub fn install_wasm_pack(&self) -> Result<(), String> {
        println!("Installing wasm-pack...");
        
        let output = Command::new("cargo")
            .arg("install")
            .arg("wasm-pack")
            .output()
            .map_err(|e| format!("Failed to run cargo install: {}", e))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to install wasm-pack: {}", error));
        }
        
        println!("wasm-pack installed successfully");
        
        Ok(())
    }
    
    /// Create example HTML file
    pub fn create_example_html(&self, package: &Package, result: &WasmCompilationResult) -> Result<PathBuf, String> {
        let html_path = result.output_dir.join("index.html");
        
        let js_file_name = result.js_file.file_name()
            .ok_or_else(|| "Invalid JS file path".to_string())?
            .to_string_lossy();
        
        let wasm_file_name = result.wasm_file.file_name()
            .ok_or_else(|| "Invalid WASM file path".to_string())?
            .to_string_lossy();
        
        let struct_name = self.to_camel_case(&package.metadata.name);
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} WebAssembly Example</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        textarea {{
            width: 100%;
            height: 100px;
            margin-bottom: 10px;
        }}
        button {{
            padding: 8px 16px;
            background-color: #4CAF50;
            color: white;
            border: none;
            cursor: pointer;
        }}
        #result {{
            margin-top: 20px;
            padding: 10px;
            border: 1px solid #ddd;
            background-color: #f9f9f9;
        }}
    </style>
</head>
<body>
    <h1>{} WebAssembly Example</h1>
    
    <h2>Code Execution</h2>
    <textarea id="code">1 + 2</textarea>
    <button id="execute">Execute</button>
    
    <div id="result">
        <h3>Result:</h3>
        <pre id="output"></pre>
    </div>
    
    <script type="module">
        import init, {{ {} }} from './{}';
        
        async function run() {{
            // Initialize the WASM module
            await init();
            
            // Create a new runtime instance
            const runtime = new {}();
            
            // Set up the execute button
            document.getElementById('execute').addEventListener('click', () => {{
                const code = document.getElementById('code').value;
                try {{
                    const result = runtime.eval(code);
                    document.getElementById('output').textContent = JSON.stringify(result, null, 2);
                }} catch (error) {{
                    document.getElementById('output').textContent = `Error: ${{error.message}}`;
                }}
            }});
        }}
        
        run();
    </script>
</body>
</html>
"#,
            package.metadata.name,
            package.metadata.name,
            struct_name,
            js_file_name,
            struct_name
        );
        
        fs::write(&html_path, html_content)
            .map_err(|e| format!("Failed to write HTML file: {}", e))?;
        
        Ok(html_path)
    }
}
