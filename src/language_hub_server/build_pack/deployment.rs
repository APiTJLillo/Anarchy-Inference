// Deployment module for Build/Pack Tools
//
// This module provides functionality for deploying Anarchy Inference packages
// to various environments using standardized templates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::process::Command;
use serde::{Serialize, Deserialize};

use crate::language_hub_server::build_pack::BuildPackConfig;
use crate::language_hub_server::build_pack::package::Package;

/// Deployment template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentTemplate {
    /// Template name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Template type
    pub template_type: DeploymentType,
    
    /// Template files
    pub files: HashMap<String, String>,
    
    /// Template configuration
    pub config: HashMap<String, String>,
}

/// Deployment type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentType {
    /// Microservice deployment
    #[serde(rename = "microservice")]
    Microservice,
    
    /// Container deployment
    #[serde(rename = "container")]
    Container,
    
    /// Serverless deployment
    #[serde(rename = "serverless")]
    Serverless,
    
    /// Edge deployment
    #[serde(rename = "edge")]
    Edge,
    
    /// Desktop application deployment
    #[serde(rename = "desktop")]
    Desktop,
    
    /// Web application deployment
    #[serde(rename = "web")]
    Web,
    
    /// Library deployment
    #[serde(rename = "library")]
    Library,
}

/// Deployment manager
pub struct DeploymentManager {
    /// Configuration
    config: BuildPackConfig,
    
    /// Templates
    templates: HashMap<String, DeploymentTemplate>,
}

impl DeploymentManager {
    /// Create a new deployment manager
    pub fn new(config: BuildPackConfig) -> Self {
        let mut manager = DeploymentManager {
            config,
            templates: HashMap::new(),
        };
        
        // Load built-in templates
        manager.load_built_in_templates();
        
        manager
    }
    
    /// Load built-in templates
    fn load_built_in_templates(&mut self) {
        // Microservice template
        self.templates.insert(
            "microservice".to_string(),
            DeploymentTemplate {
                name: "microservice".to_string(),
                description: "Microservice deployment template".to_string(),
                template_type: DeploymentType::Microservice,
                files: self.get_microservice_template_files(),
                config: HashMap::new(),
            }
        );
        
        // Container template
        self.templates.insert(
            "container".to_string(),
            DeploymentTemplate {
                name: "container".to_string(),
                description: "Container deployment template".to_string(),
                template_type: DeploymentType::Container,
                files: self.get_container_template_files(),
                config: HashMap::new(),
            }
        );
        
        // Serverless template
        self.templates.insert(
            "serverless".to_string(),
            DeploymentTemplate {
                name: "serverless".to_string(),
                description: "Serverless deployment template".to_string(),
                template_type: DeploymentType::Serverless,
                files: self.get_serverless_template_files(),
                config: HashMap::new(),
            }
        );
        
        // Edge template
        self.templates.insert(
            "edge".to_string(),
            DeploymentTemplate {
                name: "edge".to_string(),
                description: "Edge deployment template".to_string(),
                template_type: DeploymentType::Edge,
                files: self.get_edge_template_files(),
                config: HashMap::new(),
            }
        );
    }
    
    /// Get microservice template files
    fn get_microservice_template_files(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();
        
        // Dockerfile
        files.insert(
            "Dockerfile".to_string(),
            r#"FROM rust:1.60 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/{{package_name}} /usr/src/app/{{package_name}}
COPY --from=builder /usr/src/app/config /usr/src/app/config

EXPOSE 8080

CMD ["./{{package_name}}"]
"#.to_string()
        );
        
        // docker-compose.yml
        files.insert(
            "docker-compose.yml".to_string(),
            r#"version: '3'

services:
  {{package_name}}:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
    volumes:
      - ./config:/usr/src/app/config
"#.to_string()
        );
        
        // main.rs
        files.insert(
            "src/main.rs".to_string(),
            r#"use std::path::Path;
use std::sync::Arc;
use warp::Filter;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Load Anarchy Inference module
    let runtime = {{package_name}}::Runtime::new()?;
    let runtime = Arc::new(runtime);
    
    // Define routes
    let api = warp::path("api")
        .and(warp::post())
        .and(warp::path("execute"))
        .and(warp::body::json())
        .and_then(move |request: serde_json::Value| {
            let runtime = runtime.clone();
            async move {
                // Execute the request
                let result = runtime.eval(&request["code"].as_str().unwrap_or(""));
                
                // Return the result
                match result {
                    Ok(value) => Ok(warp::reply::json(&value)),
                    Err(err) => Err(warp::reject::custom(err)),
                }
            }
        });
    
    // Start the server
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port.parse::<u16>().unwrap_or(8080);
    
    println!("Starting server on port {}", port);
    
    warp::serve(api)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}
"#.to_string()
        );
        
        // README.md
        files.insert(
            "README.md".to_string(),
            r#"# {{package_name}} Microservice

This is a microservice built with Anarchy Inference.

## Running the service

```bash
docker-compose up
```

## API

### Execute code

```
POST /api/execute
```

Request body:

```json
{
  "code": "1 + 2"
}
```

Response:

```json
{
  "result": 3
}
```
"#.to_string()
        );
        
        files
    }
    
    /// Get container template files
    fn get_container_template_files(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();
        
        // Dockerfile
        files.insert(
            "Dockerfile".to_string(),
            r#"FROM rust:1.60 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/{{package_name}} /usr/src/app/{{package_name}}
COPY --from=builder /usr/src/app/modules /usr/src/app/modules

ENTRYPOINT ["./{{package_name}}"]
"#.to_string()
        );
        
        // .dockerignore
        files.insert(
            ".dockerignore".to_string(),
            r#"target/
Dockerfile
.git/
.gitignore
"#.to_string()
        );
        
        // main.rs
        files.insert(
            "src/main.rs".to_string(),
            r#"use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Initialize runtime
    let mut runtime = {{package_name}}::Runtime::new()?;
    
    // Load main module
    runtime.load_module("main", Path::new("modules/main.a.i"))?;
    
    // Call main function with arguments
    let result = runtime.call_function("main", "main", &args[1..])?;
    
    println!("{:?}", result);
    
    Ok(())
}
"#.to_string()
        );
        
        // README.md
        files.insert(
            "README.md".to_string(),
            r#"# {{package_name}} Container

This is a containerized application built with Anarchy Inference.

## Building the container

```bash
docker build -t {{package_name}} .
```

## Running the container

```bash
docker run {{package_name}} [args]
```
"#.to_string()
        );
        
        files
    }
    
    /// Get serverless template files
    fn get_serverless_template_files(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();
        
        // serverless.yml
        files.insert(
            "serverless.yml".to_string(),
            r#"service: {{package_name}}

provider:
  name: aws
  runtime: provided.al2
  architecture: arm64
  stage: ${opt:stage, 'dev'}
  region: ${opt:region, 'us-east-1'}

package:
  individually: true

functions:
  execute:
    handler: bootstrap
    package:
      artifact: target/lambda/{{package_name}}/bootstrap.zip
    events:
      - http:
          path: execute
          method: post
          cors: true
"#.to_string()
        );
        
        // main.rs
        files.insert(
            "src/main.rs".to_string(),
            r#"use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use std::path::Path;

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // Initialize runtime
    let mut runtime = {{package_name}}::Runtime::new()?;
    
    // Load main module
    runtime.load_module("main", Path::new("/opt/modules/main.a.i"))?;
    
    // Extract code from the event
    let code = event.payload["code"].as_str().unwrap_or("");
    
    // Evaluate the code
    let result = runtime.eval(code)?;
    
    // Return the result
    Ok(json!({
        "statusCode": 200,
        "body": json!({
            "result": result
        }).to_string()
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    
    // Start the Lambda runtime
    lambda_runtime::run(service_fn(function_handler)).await?;
    
    Ok(())
}
"#.to_string()
        );
        
        // Cargo.toml additions
        files.insert(
            "Cargo.toml.additions".to_string(),
            r#"[dependencies]
lambda_runtime = "0.5"
tokio = { version = "1", features = ["macros"] }
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", default-features = false, features = ["fmt"] }
"#.to_string()
        );
        
        // README.md
        files.insert(
            "README.md".to_string(),
            r#"# {{package_name}} Serverless Function

This is a serverless function built with Anarchy Inference.

## Deploying the function

```bash
# Install the Serverless Framework
npm install -g serverless

# Build the Lambda function
cargo lambda build --release

# Deploy the function
serverless deploy
```

## Invoking the function

```bash
curl -X POST \
  https://<api-id>.execute-api.<region>.amazonaws.com/dev/execute \
  -H 'Content-Type: application/json' \
  -d '{"code":"1 + 2"}'
```
"#.to_string()
        );
        
        files
    }
    
    /// Get edge template files
    fn get_edge_template_files(&self) -> HashMap<String, String> {
        let mut files = HashMap::new();
        
        // worker.js
        files.insert(
            "worker.js".to_string(),
            r#"import { {{package_name}} } from './pkg/{{package_name}}.js';

addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  try {
    // Initialize the runtime
    const runtime = new {{package_name}}();
    
    // Parse the request body
    const { code } = await request.json();
    
    // Evaluate the code
    const result = runtime.eval(code);
    
    // Return the result
    return new Response(JSON.stringify({ result }), {
      headers: { 'Content-Type': 'application/json' }
    });
  } catch (error) {
    return new Response(JSON.stringify({ error: error.message }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}
"#.to_string()
        );
        
        // wrangler.toml
        files.insert(
            "wrangler.toml".to_string(),
            r#"name = "{{package_name}}"
type = "javascript"
account_id = ""
workers_dev = true
route = ""
zone_id = ""

[build]
command = "wasm-pack build --target web"

[build.upload]
format = "service-worker"
"#.to_string()
        );
        
        // lib.rs
        files.insert(
            "src/lib.rs".to_string(),
            r#"use wasm_bindgen::prelude::*;
use std::path::Path;

#[wasm_bindgen]
pub struct {{package_name_camel}} {
    runtime: {{package_name}}::Runtime,
}

#[wasm_bindgen]
impl {{package_name_camel}} {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<{{package_name_camel}}, JsValue> {
        // Initialize runtime
        let runtime = {{package_name}}::Runtime::new()
            .map_err(|e| JsValue::from_str(&format!("Failed to create runtime: {}", e)))?;
        
        Ok({{package_name_camel}} { runtime })
    }
    
    #[wasm_bindgen]
    pub fn eval(&self, code: &str) -> Result<JsValue, JsValue> {
        // Evaluate the code
        let result = self.runtime.eval(code)
            .map_err(|e| JsValue::from_str(&format!("Evaluation error: {}", e)))?;
        
        // Convert the result to a JsValue
        Ok(serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?)
    }
}
"#.to_string()
        );
        
        // README.md
        files.insert(
            "README.md".to_string(),
            r#"# {{package_name}} Edge Worker

This is an edge worker built with Anarchy Inference.

## Deploying the worker

```bash
# Install Wrangler
npm install -g @cloudflare/wrangler

# Build the worker
wasm-pack build --target web

# Deploy the worker
wrangler publish
```

## Invoking the worker

```bash
curl -X POST \
  https://{{package_name}}.<your-subdomain>.workers.dev/ \
  -H 'Content-Type: application/json' \
  -d '{"code":"1 + 2"}'
```
"#.to_string()
        );
        
        files
    }
    
    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
    
    /// Get template
    pub fn get_template(&self, name: &str) -> Option<&DeploymentTemplate> {
        self.templates.get(name)
    }
    
    /// Deploy package using template
    pub fn deploy_package(&self, package: &Package, template_name: &str) -> Result<(), String> {
        println!("Deploying package {} using template {}", package.metadata.name, template_name);
        
        // Get the template
        let template = self.templates.get(template_name)
            .ok_or_else(|| format!("Template not found: {}", template_name))?;
        
        // Create deployment directory
        let deploy_dir = package.path.join("deploy").join(template_name);
        fs::create_dir_all(&deploy_dir)
            .map_err(|e| format!("Failed to create deployment directory: {}", e))?;
        
        // Process template files
        for (file_path, content) in &template.files {
            // Create parent directories
            let full_path = deploy_dir.join(file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            
            // Process template variables
            let processed_content = self.process_template_content(content, package);
            
            // Write the file
            fs::write(&full_path, processed_content)
                .map_err(|e| format!("Failed to write file {}: {}", file_path, e))?;
        }
        
        // Run post-deployment steps based on template type
        match template.template_type {
            DeploymentType::Microservice => {
                self.post_deploy_microservice(package, &deploy_dir)?;
            }
            DeploymentType::Container => {
                self.post_deploy_container(package, &deploy_dir)?;
            }
            DeploymentType::Serverless => {
                self.post_deploy_serverless(package, &deploy_dir)?;
            }
            DeploymentType::Edge => {
                self.post_deploy_edge(package, &deploy_dir)?;
            }
            _ => {
                // No specific post-deployment steps
            }
        }
        
        println!("Deployment completed successfully: {}", deploy_dir.display());
        
        Ok(())
    }
    
    /// Process template content
    fn process_template_content(&self, content: &str, package: &Package) -> String {
        let mut processed = content.to_string();
        
        // Replace package name
        processed = processed.replace("{{package_name}}", &package.metadata.name);
        
        // Replace package name in camel case
        let camel_case = self.to_camel_case(&package.metadata.name);
        processed = processed.replace("{{package_name_camel}}", &camel_case);
        
        // Replace version
        processed = processed.replace("{{version}}", &package.metadata.version);
        
        // Replace description
        processed = processed.replace("{{description}}", &package.metadata.description);
        
        // Replace authors
        let authors = package.metadata.authors.join(", ");
        processed = processed.replace("{{authors}}", &authors);
        
        processed
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
    
    /// Post-deployment steps for microservice
    fn post_deploy_microservice(&self, package: &Package, deploy_dir: &Path) -> Result<(), String> {
        // Copy modules
        let modules_dir = deploy_dir.join("modules");
        fs::create_dir_all(&modules_dir)
            .map_err(|e| format!("Failed to create modules directory: {}", e))?;
        
        for module_path in &package.config.modules {
            let src_path = package.path.join(module_path);
            let dst_path = modules_dir.join(src_path.file_name().unwrap());
            
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy module {}: {}", src_path.display(), e))?;
        }
        
        // Create config directory
        let config_dir = deploy_dir.join("config");
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        
        // Create config file
        let config_content = format!(
            r#"{{
  "name": "{}",
  "version": "{}",
  "modules_dir": "/usr/src/app/modules"
}}
"#,
            package.metadata.name,
            package.metadata.version
        );
        
        fs::write(config_dir.join("config.json"), config_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(())
    }
    
    /// Post-deployment steps for container
    fn post_deploy_container(&self, package: &Package, deploy_dir: &Path) -> Result<(), String> {
        // Copy modules
        let modules_dir = deploy_dir.join("modules");
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
    
    /// Post-deployment steps for serverless
    fn post_deploy_serverless(&self, package: &Package, deploy_dir: &Path) -> Result<(), String> {
        // Create modules directory
        let modules_dir = deploy_dir.join("modules");
        fs::create_dir_all(&modules_dir)
            .map_err(|e| format!("Failed to create modules directory: {}", e))?;
        
        // Copy modules
        for module_path in &package.config.modules {
            let src_path = package.path.join(module_path);
            let dst_path = modules_dir.join(src_path.file_name().unwrap());
            
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy module {}: {}", src_path.display(), e))?;
        }
        
        // Update Cargo.toml
        let cargo_toml_path = deploy_dir.join("Cargo.toml");
        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2021"

{}
"#,
            package.metadata.name,
            package.metadata.version,
            self.templates.get("serverless").unwrap().files.get("Cargo.toml.additions").unwrap()
        );
        
        fs::write(&cargo_toml_path, cargo_toml_content)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        
        Ok(())
    }
    
    /// Post-deployment steps for edge
    fn post_deploy_edge(&self, package: &Package, deploy_dir: &Path) -> Result<(), String> {
        // Create pkg directory
        let pkg_dir = deploy_dir.join("pkg");
        fs::create_dir_all(&pkg_dir)
            .map_err(|e| format!("Failed to create pkg directory: {}", e))?;
        
        // Update Cargo.toml
        let cargo_toml_path = deploy_dir.join("Cargo.toml");
        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = {{ version = "1.0", features = ["derive"] }}
serde-wasm-bindgen = "0.4"
"#,
            package.metadata.name,
            package.metadata.version
        );
        
        fs::write(&cargo_toml_path, cargo_toml_content)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        
        Ok(())
    }
    
    /// Create custom template
    pub fn create_custom_template(&mut self, name: &str, template_type: DeploymentType, files: HashMap<String, String>) -> Result<(), String> {
        if self.templates.contains_key(name) {
            return Err(format!("Template already exists: {}", name));
        }
        
        let template = DeploymentTemplate {
            name: name.to_string(),
            description: format!("Custom {} template", name),
            template_type,
            files,
            config: HashMap::new(),
        };
        
        self.templates.insert(name.to_string(), template);
        
        Ok(())
    }
    
    /// Save template to file
    pub fn save_template(&self, name: &str, path: &Path) -> Result<(), String> {
        let template = self.templates.get(name)
            .ok_or_else(|| format!("Template not found: {}", name))?;
        
        let json = serde_json::to_string_pretty(template)
            .map_err(|e| format!("Failed to serialize template: {}", e))?;
        
        fs::write(path, json)
            .map_err(|e| format!("Failed to write template file: {}", e))?;
        
        Ok(())
    }
    
    /// Load template from file
    pub fn load_template(&mut self, path: &Path) -> Result<String, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read template file: {}", e))?;
        
        let template: DeploymentTemplate = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse template: {}", e))?;
        
        let name = template.name.clone();
        self.templates.insert(name.clone(), template);
        
        Ok(name)
    }
}
