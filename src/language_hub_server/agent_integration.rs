// Integration module for Prebuilt Agents and Language Hub Server
//
// This module provides integration between the Prebuilt Agents and
// the Language Hub Server components.

use crate::prebuilt_agents::code_generation::CodeGenerationAgentManager;
use crate::prebuilt_agents::pattern_implementation::PatternImplementationAgentManager;
use crate::prebuilt_agents::onboarding::OnboardingAgentManager;
use crate::language_hub_server::lsp::LspServer;
use crate::language_hub_server::repl::ReplService;
use crate::language_hub_server::build_pack::BuildPackTools;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Manager for integrating prebuilt agents with the Language Hub Server
pub struct AgentIntegrationManager {
    /// Code generation agent manager
    code_generation_manager: Arc<Mutex<CodeGenerationAgentManager>>,
    
    /// Pattern implementation agent manager
    pattern_implementation_manager: Arc<Mutex<PatternImplementationAgentManager>>,
    
    /// Onboarding agent manager
    onboarding_manager: Arc<Mutex<OnboardingAgentManager>>,
    
    /// LSP server reference
    lsp_server: Arc<Mutex<LspServer>>,
    
    /// REPL service reference
    repl_service: Arc<Mutex<ReplService>>,
    
    /// Build/Pack tools reference
    build_pack_tools: Arc<Mutex<BuildPackTools>>,
    
    /// Integration handlers
    handlers: HashMap<String, Box<dyn Fn(&str) -> String + Send + Sync>>,
}

impl AgentIntegrationManager {
    /// Create a new agent integration manager
    pub fn new(
        code_generation_manager: Arc<Mutex<CodeGenerationAgentManager>>,
        pattern_implementation_manager: Arc<Mutex<PatternImplementationAgentManager>>,
        onboarding_manager: Arc<Mutex<OnboardingAgentManager>>,
        lsp_server: Arc<Mutex<LspServer>>,
        repl_service: Arc<Mutex<ReplService>>,
        build_pack_tools: Arc<Mutex<BuildPackTools>>,
    ) -> Self {
        let mut manager = AgentIntegrationManager {
            code_generation_manager,
            pattern_implementation_manager,
            onboarding_manager,
            lsp_server,
            repl_service,
            build_pack_tools,
            handlers: HashMap::new(),
        };
        
        manager.register_handlers();
        
        manager
    }
    
    /// Register integration handlers
    fn register_handlers(&mut self) {
        // Register LSP integration handlers
        self.register_lsp_handlers();
        
        // Register REPL integration handlers
        self.register_repl_handlers();
        
        // Register Build/Pack integration handlers
        self.register_build_pack_handlers();
    }
    
    /// Register LSP integration handlers
    fn register_lsp_handlers(&mut self) {
        // Code completion handler
        let code_gen = Arc::clone(&self.code_generation_manager);
        let pattern_impl = Arc::clone(&self.pattern_implementation_manager);
        self.handlers.insert("lsp.completion".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get document and position
            let document = params_obj["document"].as_str().unwrap_or("");
            let line = params_obj["line"].as_u64().unwrap_or(0) as usize;
            let character = params_obj["character"].as_u64().unwrap_or(0) as usize;
            
            // Generate completions from code generation agent
            let mut completions = Vec::new();
            if let Ok(code_gen_guard) = code_gen.lock() {
                completions.extend(code_gen_guard.generate_completions(document, line, character));
            }
            
            // Generate completions from pattern implementation agent
            if let Ok(pattern_impl_guard) = pattern_impl.lock() {
                completions.extend(pattern_impl_guard.generate_pattern_completions(document, line, character));
            }
            
            // Return completions as JSON
            serde_json::to_string(&completions).unwrap_or_default()
        }));
        
        // Diagnostic handler
        let code_gen = Arc::clone(&self.code_generation_manager);
        let onboarding = Arc::clone(&self.onboarding_manager);
        self.handlers.insert("lsp.diagnostic".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get document
            let document = params_obj["document"].as_str().unwrap_or("");
            
            // Generate diagnostics from code generation agent
            let mut diagnostics = Vec::new();
            if let Ok(code_gen_guard) = code_gen.lock() {
                diagnostics.extend(code_gen_guard.generate_diagnostics(document));
            }
            
            // Generate diagnostics from best practices agent
            if let Ok(onboarding_guard) = onboarding.lock() {
                let best_practices_violations = onboarding_guard.best_practices_agent.check_code(&onboarding_guard.context, document);
                
                // Convert violations to diagnostics
                for violation in best_practices_violations {
                    diagnostics.push(serde_json::json!({
                        "range": {
                            "start": { "line": violation.location.0, "character": violation.location.1 },
                            "end": { "line": violation.location.0, "character": violation.location.1 + 1 }
                        },
                        "severity": match violation.severity {
                            crate::prebuilt_agents::onboarding::ViolationSeverity::Info => 3,
                            crate::prebuilt_agents::onboarding::ViolationSeverity::Warning => 2,
                            crate::prebuilt_agents::onboarding::ViolationSeverity::Error => 1,
                            crate::prebuilt_agents::onboarding::ViolationSeverity::Critical => 1,
                        },
                        "source": "anarchy-best-practices",
                        "message": violation.description,
                        "relatedInformation": [{
                            "message": violation.suggestion,
                            "location": {
                                "uri": "file:///path/to/document",
                                "range": {
                                    "start": { "line": violation.location.0, "character": violation.location.1 },
                                    "end": { "line": violation.location.0, "character": violation.location.1 + 1 }
                                }
                            }
                        }]
                    }));
                }
            }
            
            // Return diagnostics as JSON
            serde_json::to_string(&diagnostics).unwrap_or_default()
        }));
        
        // Code action handler
        let code_gen = Arc::clone(&self.code_generation_manager);
        let pattern_impl = Arc::clone(&self.pattern_implementation_manager);
        self.handlers.insert("lsp.codeAction".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get document and range
            let document = params_obj["document"].as_str().unwrap_or("");
            let start_line = params_obj["range"]["start"]["line"].as_u64().unwrap_or(0) as usize;
            let start_character = params_obj["range"]["start"]["character"].as_u64().unwrap_or(0) as usize;
            let end_line = params_obj["range"]["end"]["line"].as_u64().unwrap_or(0) as usize;
            let end_character = params_obj["range"]["end"]["character"].as_u64().unwrap_or(0) as usize;
            
            // Generate code actions from code generation agent
            let mut code_actions = Vec::new();
            if let Ok(code_gen_guard) = code_gen.lock() {
                code_actions.extend(code_gen_guard.generate_code_actions(
                    document, 
                    (start_line, start_character), 
                    (end_line, end_character)
                ));
            }
            
            // Generate code actions from pattern implementation agent
            if let Ok(pattern_impl_guard) = pattern_impl.lock() {
                code_actions.extend(pattern_impl_guard.generate_pattern_actions(
                    document, 
                    (start_line, start_character), 
                    (end_line, end_character)
                ));
            }
            
            // Return code actions as JSON
            serde_json::to_string(&code_actions).unwrap_or_default()
        }));
        
        // Documentation hover handler
        let onboarding = Arc::clone(&self.onboarding_manager);
        self.handlers.insert("lsp.hover".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get document and position
            let document = params_obj["document"].as_str().unwrap_or("");
            let line = params_obj["line"].as_u64().unwrap_or(0) as usize;
            let character = params_obj["character"].as_u64().unwrap_or(0) as usize;
            
            // Generate hover information from documentation agent
            let mut hover_info = String::new();
            if let Ok(onboarding_guard) = onboarding.lock() {
                let topics = onboarding_guard.documentation_agent.get_contextual_help(
                    &onboarding_guard.context,
                    document,
                    line * 80 + character // Simplified position calculation
                );
                
                if !topics.is_empty() {
                    // Use the first topic for hover
                    let topic = topics[0];
                    hover_info = format!("# {}\n\n{}", topic.title, topic.content);
                    
                    // Add examples if available
                    if !topic.examples.is_empty() {
                        hover_info.push_str("\n\n## Example\n\n```anarchy\n");
                        hover_info.push_str(&topic.examples[0].code);
                        hover_info.push_str("\n```");
                    }
                }
            }
            
            // Return hover information as JSON
            serde_json::to_string(&serde_json::json!({
                "contents": {
                    "kind": "markdown",
                    "value": hover_info
                }
            })).unwrap_or_default()
        }));
    }
    
    /// Register REPL integration handlers
    fn register_repl_handlers(&mut self) {
        // Tutorial execution handler
        let onboarding = Arc::clone(&self.onboarding_manager);
        self.handlers.insert("repl.executeTutorial".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get tutorial ID and code
            let tutorial_id = params_obj["tutorialId"].as_str().unwrap_or("");
            let code = params_obj["code"].as_str().unwrap_or("");
            
            // Execute tutorial step
            let mut result = serde_json::json!({
                "success": false,
                "message": "Failed to execute tutorial"
            });
            
            if let Ok(mut onboarding_guard) = onboarding.lock() {
                // Start tutorial if not already started
                if onboarding_guard.context.progress.current_tutorial.is_none() {
                    if let Ok(tutorial) = onboarding_guard.tutorial_agent.start_tutorial(&mut onboarding_guard.context, tutorial_id) {
                        result = serde_json::json!({
                            "success": true,
                            "message": format!("Started tutorial: {}", tutorial.title),
                            "currentStep": 0
                        });
                    }
                } else if let Some(current_tutorial_id) = &onboarding_guard.context.progress.current_tutorial {
                    if current_tutorial_id == tutorial_id {
                        // Get current step
                        if let Some(step) = onboarding_guard.tutorial_agent.get_current_step(&onboarding_guard.context) {
                            // If step has an exercise, validate the code
                            if step.exercise.is_some() {
                                if let Ok(validation) = onboarding_guard.tutorial_agent.submit_exercise(&mut onboarding_guard.context, code) {
                                    result = serde_json::json!({
                                        "success": true,
                                        "isCorrect": validation.is_correct,
                                        "feedback": validation.feedback,
                                        "issues": validation.issues
                                    });
                                    
                                    // If correct, move to next step
                                    if validation.is_correct {
                                        if let Ok(next_step) = onboarding_guard.tutorial_agent.next_step(&mut onboarding_guard.context) {
                                            if let Some(step) = next_step {
                                                result["nextStep"] = serde_json::json!({
                                                    "title": step.title,
                                                    "description": step.description,
                                                    "codeExample": step.code_example,
                                                    "hasExercise": step.exercise.is_some()
                                                });
                                            } else {
                                                result["tutorialCompleted"] = serde_json::json!(true);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // No exercise, just move to next step
                                if let Ok(next_step) = onboarding_guard.tutorial_agent.next_step(&mut onboarding_guard.context) {
                                    result = serde_json::json!({
                                        "success": true,
                                        "message": "Moving to next step"
                                    });
                                    
                                    if let Some(step) = next_step {
                                        result["nextStep"] = serde_json::json!({
                                            "title": step.title,
                                            "description": step.description,
                                            "codeExample": step.code_example,
                                            "hasExercise": step.exercise.is_some()
                                        });
                                    } else {
                                        result["tutorialCompleted"] = serde_json::json!(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Return result as JSON
            serde_json::to_string(&result).unwrap_or_default()
        }));
        
        // Documentation lookup handler
        let onboarding = Arc::clone(&self.onboarding_manager);
        self.handlers.insert("repl.lookupDocumentation".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get query
            let query = params_obj["query"].as_str().unwrap_or("");
            
            // Search documentation
            let mut result = serde_json::json!({
                "topics": []
            });
            
            if let Ok(onboarding_guard) = onboarding.lock() {
                let topics = onboarding_guard.documentation_agent.search_documentation(&onboarding_guard.context, query);
                
                let mut topic_list = Vec::new();
                for topic in topics {
                    let mut topic_json = serde_json::json!({
                        "id": topic.id,
                        "title": topic.title,
                        "content": topic.content,
                        "examples": []
                    });
                    
                    // Add examples
                    let mut examples_json = Vec::new();
                    for example in &topic.examples {
                        examples_json.push(serde_json::json!({
                            "title": example.title,
                            "description": example.description,
                            "code": example.code
                        }));
                    }
                    
                    topic_json["examples"] = serde_json::json!(examples_json);
                    topic_list.push(topic_json);
                }
                
                result["topics"] = serde_json::json!(topic_list);
            }
            
            // Return result as JSON
            serde_json::to_string(&result).unwrap_or_default()
        }));
    }
    
    /// Register Build/Pack integration handlers
    fn register_build_pack_handlers(&mut self) {
        // Project setup handler
        let onboarding = Arc::clone(&self.onboarding_manager);
        let build_pack = Arc::clone(&self.build_pack_tools);
        self.handlers.insert("buildPack.setupProject".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get template ID, project name, and output directory
            let template_id = params_obj["templateId"].as_str().unwrap_or("");
            let project_name = params_obj["projectName"].as_str().unwrap_or("");
            let output_dir = params_obj["outputDir"].as_str().unwrap_or("");
            
            // Create project
            let mut result = serde_json::json!({
                "success": false,
                "message": "Failed to create project"
            });
            
            if let Ok(onboarding_guard) = onboarding.lock() {
                if let Ok(()) = onboarding_guard.project_setup_agent.create_project(
                    &onboarding_guard.context,
                    template_id,
                    project_name,
                    &std::path::PathBuf::from(output_dir)
                ) {
                    result = serde_json::json!({
                        "success": true,
                        "message": format!("Created project '{}' using template '{}'", project_name, template_id),
                        "projectPath": format!("{}/{}", output_dir, project_name)
                    });
                    
                    // Initialize build/pack tools for the project
                    if let Ok(mut build_pack_guard) = build_pack.lock() {
                        if build_pack_guard.initialize_project(&format!("{}/{}", output_dir, project_name)) {
                            result["buildPackInitialized"] = serde_json::json!(true);
                        }
                    }
                }
            }
            
            // Return result as JSON
            serde_json::to_string(&result).unwrap_or_default()
        }));
        
        // Code generation integration handler
        let code_gen = Arc::clone(&self.code_generation_manager);
        let build_pack = Arc::clone(&self.build_pack_tools);
        self.handlers.insert("buildPack.optimizeCode".to_string(), Box::new(move |params| {
            // Parse parameters
            let params_obj: serde_json::Value = serde_json::from_str(params).unwrap_or_default();
            
            // Get project path and optimization type
            let project_path = params_obj["projectPath"].as_str().unwrap_or("");
            let optimization_type = params_obj["optimizationType"].as_str().unwrap_or("performance");
            
            // Optimize code
            let mut result = serde_json::json!({
                "success": false,
                "message": "Failed to optimize code"
            });
            
            if let Ok(code_gen_guard) = code_gen.lock() {
                if let Ok(build_pack_guard) = build_pack.lock() {
                    // Get project files
                    let files = build_pack_guard.get_project_files(project_path);
                    
                    // Optimize each file
                    let mut optimized_files = Vec::new();
                    for file in files {
                        if file.ends_with(".a.i") {
                            // Read file content
                            if let Ok(content) = std::fs::read_to_string(&file) {
                                // Optimize based on type
                                let optimized_content = match optimization_type {
                                    "performance" => code_gen_guard.performance_agent.optimize_code(&content),
                                    "security" => code_gen_guard.security_agent.secure_code(&content),
                                    _ => content
                                };
                                
                                // Write optimized content back
                                if let Ok(()) = std::fs::write(&file, optimized_content) {
                                    optimized_files.push(file);
                                }
                            }
                        }
                    }
                    
                    result = serde_json::json!({
                        "success": true,
                        "message": format!("Optimized {} files for {}", optimized_files.len(), optimization_type),
                        "optimizedFiles": optimized_files
                    });
                }
            }
            
            // Return result as JSON
            serde_json::to_string(&result).unwrap_or_default()
        }));
    }
    
    /// Handle a request
    pub fn handle_request(&self, request_type: &str, params: &str) -> String {
        if let Some(handler) = self.handlers.get(request_type) {
            handler(params)
        } else {
            serde_json::to_string(&serde_json::json!({
                "error": format!("Unknown request type: {}", request_type)
            })).unwrap_or_default()
        }
    }
    
    /// Initialize the integration manager
    pub fn initialize(&self) {
        // Register LSP commands
        if let Ok(mut lsp_server) = self.lsp_server.lock() {
            // Register code generation commands
            lsp_server.register_command("anarchy.refactor", Box::new(|params| {
                // Implementation omitted for brevity
                serde_json::json!({})
            }));
            
            lsp_server.register_command("anarchy.lint", Box::new(|params| {
                // Implementation omitted for brevity
                serde_json::json!({})
            }));
            
            lsp_server.register_command("anarchy.securityCheck", Box::new(|params| {
                // Implementation omitted for brevity
                serde_json::json!({})
            }));
            
            // Register pattern implementation commands
            lsp_server.register_command("anarchy.implementPattern", Box::new(|params| {
                // Implementation omitted for brevity
                serde_json::json!({})
            }));
            
            // Register onboarding commands
            lsp_server.register_command("anarchy.startTutorial", Box::new(|params| {
                // Implementation omitted for brevity
                serde_json::json!({})
            }));
            
            lsp_server.register_command("anarchy.showDocumentation", Box::new(|params| {
                // Implementation omitted for brevity
                serde_json::json!({})
            }));
        }
        
        // Register REPL commands
        if let Ok(mut repl_service) = self.repl_service.lock() {
            // Register tutorial commands
            repl_service.register_command("tutorial", Box::new(|args, session| {
                // Implementation omitted for brevity
                "Tutorial command executed".to_string()
            }));
            
            // Register documentation commands
            repl_service.register_command("help", Box::new(|args, session| {
                // Implementation omitted for brevity
                "Help command executed".to_string()
            }));
            
            repl_service.register_command("doc", Box::new(|args, session| {
                // Implementation omitted for brevity
                "Documentation command executed".to_string()
            }));
            
            // Register best practices commands
            repl_service.register_command("lint", Box::new(|args, session| {
                // Implementation omitted for brevity
                "Lint command executed".to_string()
            }));
        }
    }
}
