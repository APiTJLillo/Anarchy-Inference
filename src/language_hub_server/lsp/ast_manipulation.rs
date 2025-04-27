// AST manipulation endpoints module for LSP-like Component
//
// This module provides endpoints for AST manipulation and transformation
// in Anarchy Inference code, offering standardized interfaces for code refactoring.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use crate::language_hub_server::lsp::protocol::{Position, Range, TextEdit, WorkspaceEdit};
use crate::language_hub_server::lsp::document::{Document, DocumentManager, SharedDocumentManager};
use crate::language_hub_server::lsp::parser_integration::{AstNode, ParseResult};
use crate::language_hub_server::lsp::refactoring_provider::{RefactoringProvider, SharedRefactoringProvider};
use crate::language_hub_server::lsp::ast_utils::AstUtils;

/// AST transformation type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformationType {
    /// Rename symbol
    Rename,
    
    /// Extract function
    ExtractFunction,
    
    /// Extract variable
    ExtractVariable,
    
    /// Inline function
    InlineFunction,
    
    /// Inline variable
    InlineVariable,
    
    /// Move declaration
    MoveDeclaration,
    
    /// Change signature
    ChangeSignature,
    
    /// Convert to arrow function
    ConvertToArrowFunction,
    
    /// Convert to regular function
    ConvertToRegularFunction,
    
    /// Add parameter
    AddParameter,
    
    /// Remove parameter
    RemoveParameter,
    
    /// Reorder parameters
    ReorderParameters,
    
    /// Add import
    AddImport,
    
    /// Remove import
    RemoveImport,
    
    /// Organize imports
    OrganizeImports,
    
    /// Custom transformation
    Custom,
}

/// AST transformation options
#[derive(Debug, Clone)]
pub struct TransformationOptions {
    /// Whether to apply the transformation to all files
    pub apply_to_all_files: bool,
    
    /// Whether to preview the transformation
    pub preview: bool,
    
    /// Maximum number of files to transform
    pub max_files: usize,
}

impl Default for TransformationOptions {
    fn default() -> Self {
        TransformationOptions {
            apply_to_all_files: true,
            preview: false,
            max_files: 100,
        }
    }
}

/// AST transformation request
#[derive(Debug, Clone)]
pub struct TransformationRequest {
    /// The document URI
    pub document_uri: String,
    
    /// The position in the document
    pub position: Position,
    
    /// The transformation type
    pub transformation_type: TransformationType,
    
    /// The transformation options
    pub options: Option<TransformationOptions>,
    
    /// The AST of the document
    pub ast: Option<AstNode>,
    
    /// The parse result
    pub parse_result: Option<ParseResult>,
    
    /// Additional parameters
    pub parameters: HashMap<String, String>,
}

/// AST transformation response
#[derive(Debug, Clone)]
pub struct TransformationResponse {
    /// The workspace edit
    pub edit: WorkspaceEdit,
    
    /// Whether the transformation was successful
    pub success: bool,
    
    /// The error message if the transformation failed
    pub error_message: Option<String>,
    
    /// The number of files affected
    pub files_affected: usize,
    
    /// The number of edits
    pub edit_count: usize,
}

/// AST query type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryType {
    /// Find all references
    FindReferences,
    
    /// Find definition
    FindDefinition,
    
    /// Find implementations
    FindImplementations,
    
    /// Find type definition
    FindTypeDefinition,
    
    /// Find all symbols
    FindSymbols,
    
    /// Find all functions
    FindFunctions,
    
    /// Find all variables
    FindVariables,
    
    /// Find all classes
    FindClasses,
    
    /// Find all imports
    FindImports,
    
    /// Find all exports
    FindExports,
    
    /// Custom query
    Custom,
}

/// AST query options
#[derive(Debug, Clone)]
pub struct QueryOptions {
    /// Whether to include results from all files
    pub include_all_files: bool,
    
    /// Maximum number of results
    pub max_results: usize,
}

impl Default for QueryOptions {
    fn default() -> Self {
        QueryOptions {
            include_all_files: true,
            max_results: 100,
        }
    }
}

/// AST query request
#[derive(Debug, Clone)]
pub struct QueryRequest {
    /// The document URI
    pub document_uri: String,
    
    /// The position in the document
    pub position: Position,
    
    /// The query type
    pub query_type: QueryType,
    
    /// The query options
    pub options: Option<QueryOptions>,
    
    /// The AST of the document
    pub ast: Option<AstNode>,
    
    /// The parse result
    pub parse_result: Option<ParseResult>,
    
    /// Additional parameters
    pub parameters: HashMap<String, String>,
}

/// AST query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// The document URI
    pub uri: String,
    
    /// The range in the document
    pub range: Range,
    
    /// The result type
    pub result_type: String,
    
    /// The result name
    pub name: String,
    
    /// The container name
    pub container_name: Option<String>,
    
    /// Additional data
    pub additional_data: HashMap<String, String>,
}

/// AST query response
#[derive(Debug, Clone)]
pub struct QueryResponse {
    /// The query results
    pub results: Vec<QueryResult>,
    
    /// Whether the query was successful
    pub success: bool,
    
    /// The error message if the query failed
    pub error_message: Option<String>,
    
    /// The number of results
    pub result_count: usize,
}

/// AST generation type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenerationType {
    /// Generate function
    Function,
    
    /// Generate class
    Class,
    
    /// Generate interface
    Interface,
    
    /// Generate enum
    Enum,
    
    /// Generate module
    Module,
    
    /// Generate test
    Test,
    
    /// Generate documentation
    Documentation,
    
    /// Custom generation
    Custom,
}

/// AST generation options
#[derive(Debug, Clone)]
pub struct GenerationOptions {
    /// Whether to overwrite existing code
    pub overwrite: bool,
    
    /// Whether to preview the generation
    pub preview: bool,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        GenerationOptions {
            overwrite: false,
            preview: true,
        }
    }
}

/// AST generation request
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    /// The document URI
    pub document_uri: String,
    
    /// The position in the document
    pub position: Position,
    
    /// The generation type
    pub generation_type: GenerationType,
    
    /// The generation options
    pub options: Option<GenerationOptions>,
    
    /// The AST of the document
    pub ast: Option<AstNode>,
    
    /// The parse result
    pub parse_result: Option<ParseResult>,
    
    /// Additional parameters
    pub parameters: HashMap<String, String>,
}

/// AST generation response
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    /// The workspace edit
    pub edit: WorkspaceEdit,
    
    /// Whether the generation was successful
    pub success: bool,
    
    /// The error message if the generation failed
    pub error_message: Option<String>,
    
    /// The generated code
    pub generated_code: String,
}

/// AST manipulation endpoints
pub struct AstManipulationEndpoints {
    /// The document manager
    document_manager: SharedDocumentManager,
    
    /// The refactoring provider
    refactoring_provider: SharedRefactoringProvider,
}

impl AstManipulationEndpoints {
    /// Create new AST manipulation endpoints
    pub fn new(
        document_manager: SharedDocumentManager,
        refactoring_provider: SharedRefactoringProvider
    ) -> Self {
        AstManipulationEndpoints {
            document_manager,
            refactoring_provider,
        }
    }
    
    /// Apply transformation
    pub fn apply_transformation(
        &self,
        request: TransformationRequest
    ) -> Result<TransformationResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get the options
        let options = request.options.unwrap_or_default();
        
        // Get or create the AST
        let ast = if let Some(ast_node) = request.ast {
            ast_node
        } else if let Some(parse_result) = &request.parse_result {
            parse_result.ast.clone()
        } else {
            // Parse the document
            let parse_result = self.parse_document(&document)?;
            parse_result.ast
        };
        
        // Apply the transformation
        let refactoring_provider = self.refactoring_provider.lock().unwrap();
        
        let result = match request.transformation_type {
            TransformationType::Rename => {
                let new_name = request.parameters.get("newName")
                    .ok_or_else(|| "Missing 'newName' parameter".to_string())?;
                
                refactoring_provider.rename(&document, request.position, new_name, options.apply_to_all_files)
            },
            TransformationType::ExtractFunction => {
                let function_name = request.parameters.get("functionName")
                    .ok_or_else(|| "Missing 'functionName' parameter".to_string())?;
                
                let selection_range = if let Some(range_str) = request.parameters.get("selectionRange") {
                    self.parse_range(range_str)?
                } else {
                    return Err("Missing 'selectionRange' parameter".to_string());
                };
                
                refactoring_provider.extract_function(&document, selection_range, function_name)
            },
            TransformationType::ExtractVariable => {
                let variable_name = request.parameters.get("variableName")
                    .ok_or_else(|| "Missing 'variableName' parameter".to_string())?;
                
                let selection_range = if let Some(range_str) = request.parameters.get("selectionRange") {
                    self.parse_range(range_str)?
                } else {
                    return Err("Missing 'selectionRange' parameter".to_string());
                };
                
                refactoring_provider.extract_variable(&document, selection_range, variable_name)
            },
            TransformationType::InlineFunction => {
                refactoring_provider.inline_function(&document, request.position)
            },
            TransformationType::InlineVariable => {
                refactoring_provider.inline_variable(&document, request.position)
            },
            TransformationType::MoveDeclaration => {
                let target_uri = request.parameters.get("targetUri")
                    .ok_or_else(|| "Missing 'targetUri' parameter".to_string())?;
                
                let target_position = if let Some(pos_str) = request.parameters.get("targetPosition") {
                    self.parse_position(pos_str)?
                } else {
                    return Err("Missing 'targetPosition' parameter".to_string());
                };
                
                refactoring_provider.move_declaration(&document, request.position, target_uri, target_position)
            },
            TransformationType::ChangeSignature => {
                let new_parameters = request.parameters.get("newParameters")
                    .ok_or_else(|| "Missing 'newParameters' parameter".to_string())?;
                
                refactoring_provider.change_signature(&document, request.position, new_parameters)
            },
            TransformationType::ConvertToArrowFunction => {
                refactoring_provider.convert_to_arrow_function(&document, request.position)
            },
            TransformationType::ConvertToRegularFunction => {
                refactoring_provider.convert_to_regular_function(&document, request.position)
            },
            TransformationType::AddParameter => {
                let parameter_name = request.parameters.get("parameterName")
                    .ok_or_else(|| "Missing 'parameterName' parameter".to_string())?;
                
                let parameter_type = request.parameters.get("parameterType").cloned();
                let default_value = request.parameters.get("defaultValue").cloned();
                
                refactoring_provider.add_parameter(&document, request.position, parameter_name, parameter_type, default_value)
            },
            TransformationType::RemoveParameter => {
                let parameter_index = if let Some(index_str) = request.parameters.get("parameterIndex") {
                    index_str.parse::<usize>().map_err(|_| "Invalid 'parameterIndex' parameter".to_string())?
                } else {
                    return Err("Missing 'parameterIndex' parameter".to_string());
                };
                
                refactoring_provider.remove_parameter(&document, request.position, parameter_index)
            },
            TransformationType::ReorderParameters => {
                let new_order = if let Some(order_str) = request.parameters.get("newOrder") {
                    // Parse the new order as a comma-separated list of indices
                    order_str.split(',')
                        .map(|s| s.trim().parse::<usize>())
                        .collect::<Result<Vec<usize>, _>>()
                        .map_err(|_| "Invalid 'newOrder' parameter".to_string())?
                } else {
                    return Err("Missing 'newOrder' parameter".to_string());
                };
                
                refactoring_provider.reorder_parameters(&document, request.position, &new_order)
            },
            TransformationType::AddImport => {
                let import_name = request.parameters.get("importName")
                    .ok_or_else(|| "Missing 'importName' parameter".to_string())?;
                
                let module_name = request.parameters.get("moduleName")
                    .ok_or_else(|| "Missing 'moduleName' parameter".to_string())?;
                
                let is_default = request.parameters.get("isDefault")
                    .map(|s| s == "true")
                    .unwrap_or(false);
                
                refactoring_provider.add_import(&document, import_name, module_name, is_default)
            },
            TransformationType::RemoveImport => {
                let import_name = request.parameters.get("importName")
                    .ok_or_else(|| "Missing 'importName' parameter".to_string())?;
                
                refactoring_provider.remove_import(&document, import_name)
            },
            TransformationType::OrganizeImports => {
                refactoring_provider.organize_imports(&document)
            },
            TransformationType::Custom => {
                let transformation_name = request.parameters.get("transformationName")
                    .ok_or_else(|| "Missing 'transformationName' parameter".to_string())?;
                
                match transformation_name.as_str() {
                    "sortMembers" => refactoring_provider.sort_members(&document, request.position),
                    "addConstructor" => refactoring_provider.add_constructor(&document, request.position),
                    "addGetter" => {
                        let property_name = request.parameters.get("propertyName")
                            .ok_or_else(|| "Missing 'propertyName' parameter".to_string())?;
                        
                        refactoring_provider.add_getter(&document, request.position, property_name)
                    },
                    "addSetter" => {
                        let property_name = request.parameters.get("propertyName")
                            .ok_or_else(|| "Missing 'propertyName' parameter".to_string())?;
                        
                        refactoring_provider.add_setter(&document, request.position, property_name)
                    },
                    _ => Err(format!("Unknown custom transformation: {}", transformation_name)),
                }
            },
        }?;
        
        // Count the number of files affected and edits
        let mut files_affected = 0;
        let mut edit_count = 0;
        
        for (_, edits) in &result.changes {
            files_affected += 1;
            edit_count += edits.len();
        }
        
        // Create the response
        let response = TransformationResponse {
            edit: result,
            success: true,
            error_message: None,
            files_affected,
            edit_count,
        };
        
        Ok(response)
    }
    
    /// Execute query
    pub fn execute_query(
        &self,
        request: QueryRequest
    ) -> Result<QueryResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get the options
        let options = request.options.unwrap_or_default();
        
        // Get or create the AST
        let ast = if let Some(ast_node) = request.ast {
            ast_node
        } else if let Some(parse_result) = &request.parse_result {
            parse_result.ast.clone()
        } else {
            // Parse the document
            let parse_result = self.parse_document(&document)?;
            parse_result.ast
        };
        
        // Execute the query
        let mut results = Vec::new();
        
        match request.query_type {
            QueryType::FindReferences => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let references = refactoring_provider.find_references(&document, request.position, options.include_all_files)?;
                
                for reference in references {
                    results.push(QueryResult {
                        uri: reference.uri.clone(),
                        range: reference.range,
                        result_type: "reference".to_string(),
                        name: reference.text.clone(),
                        container_name: reference.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindDefinition => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let definitions = refactoring_provider.find_definition(&document, request.position)?;
                
                for definition in definitions {
                    results.push(QueryResult {
                        uri: definition.uri.clone(),
                        range: definition.range,
                        result_type: "definition".to_string(),
                        name: definition.text.clone(),
                        container_name: definition.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindImplementations => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let implementations = refactoring_provider.find_implementations(&document, request.position)?;
                
                for implementation in implementations {
                    results.push(QueryResult {
                        uri: implementation.uri.clone(),
                        range: implementation.range,
                        result_type: "implementation".to_string(),
                        name: implementation.text.clone(),
                        container_name: implementation.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindTypeDefinition => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let type_definitions = refactoring_provider.find_type_definition(&document, request.position)?;
                
                for type_definition in type_definitions {
                    results.push(QueryResult {
                        uri: type_definition.uri.clone(),
                        range: type_definition.range,
                        result_type: "typeDefinition".to_string(),
                        name: type_definition.text.clone(),
                        container_name: type_definition.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindSymbols => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let symbols = refactoring_provider.find_symbols(&document)?;
                
                for symbol in symbols {
                    results.push(QueryResult {
                        uri: document.uri.clone(),
                        range: symbol.range,
                        result_type: symbol.kind.clone(),
                        name: symbol.name.clone(),
                        container_name: symbol.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindFunctions => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let functions = refactoring_provider.find_functions(&document)?;
                
                for function in functions {
                    results.push(QueryResult {
                        uri: document.uri.clone(),
                        range: function.range,
                        result_type: "function".to_string(),
                        name: function.name.clone(),
                        container_name: function.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindVariables => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let variables = refactoring_provider.find_variables(&document)?;
                
                for variable in variables {
                    results.push(QueryResult {
                        uri: document.uri.clone(),
                        range: variable.range,
                        result_type: "variable".to_string(),
                        name: variable.name.clone(),
                        container_name: variable.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindClasses => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let classes = refactoring_provider.find_classes(&document)?;
                
                for class in classes {
                    results.push(QueryResult {
                        uri: document.uri.clone(),
                        range: class.range,
                        result_type: "class".to_string(),
                        name: class.name.clone(),
                        container_name: class.container_name.clone(),
                        additional_data: HashMap::new(),
                    });
                }
            },
            QueryType::FindImports => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let imports = refactoring_provider.find_imports(&document)?;
                
                for import in imports {
                    let mut additional_data = HashMap::new();
                    additional_data.insert("moduleName".to_string(), import.module_name.clone());
                    additional_data.insert("isDefault".to_string(), import.is_default.to_string());
                    
                    results.push(QueryResult {
                        uri: document.uri.clone(),
                        range: import.range,
                        result_type: "import".to_string(),
                        name: import.name.clone(),
                        container_name: None,
                        additional_data,
                    });
                }
            },
            QueryType::FindExports => {
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                let exports = refactoring_provider.find_exports(&document)?;
                
                for export in exports {
                    let mut additional_data = HashMap::new();
                    additional_data.insert("isDefault".to_string(), export.is_default.to_string());
                    
                    results.push(QueryResult {
                        uri: document.uri.clone(),
                        range: export.range,
                        result_type: "export".to_string(),
                        name: export.name.clone(),
                        container_name: None,
                        additional_data,
                    });
                }
            },
            QueryType::Custom => {
                let query_name = request.parameters.get("queryName")
                    .ok_or_else(|| "Missing 'queryName' parameter".to_string())?;
                
                let refactoring_provider = self.refactoring_provider.lock().unwrap();
                
                match query_name.as_str() {
                    "findUnusedVariables" => {
                        let unused_variables = refactoring_provider.find_unused_variables(&document)?;
                        
                        for variable in unused_variables {
                            results.push(QueryResult {
                                uri: document.uri.clone(),
                                range: variable.range,
                                result_type: "unusedVariable".to_string(),
                                name: variable.name.clone(),
                                container_name: variable.container_name.clone(),
                                additional_data: HashMap::new(),
                            });
                        }
                    },
                    "findUnusedFunctions" => {
                        let unused_functions = refactoring_provider.find_unused_functions(&document)?;
                        
                        for function in unused_functions {
                            results.push(QueryResult {
                                uri: document.uri.clone(),
                                range: function.range,
                                result_type: "unusedFunction".to_string(),
                                name: function.name.clone(),
                                container_name: function.container_name.clone(),
                                additional_data: HashMap::new(),
                            });
                        }
                    },
                    "findDuplicateCode" => {
                        let duplicates = refactoring_provider.find_duplicate_code(&document)?;
                        
                        for duplicate in duplicates {
                            results.push(QueryResult {
                                uri: document.uri.clone(),
                                range: duplicate.range,
                                result_type: "duplicateCode".to_string(),
                                name: format!("Duplicate code block {}", results.len() + 1),
                                container_name: None,
                                additional_data: HashMap::new(),
                            });
                        }
                    },
                    _ => return Err(format!("Unknown custom query: {}", query_name)),
                }
            },
        }
        
        // Limit the number of results
        if results.len() > options.max_results {
            results.truncate(options.max_results);
        }
        
        // Create the response
        let response = QueryResponse {
            results,
            success: true,
            error_message: None,
            result_count: results.len(),
        };
        
        Ok(response)
    }
    
    /// Generate code
    pub fn generate_code(
        &self,
        request: GenerationRequest
    ) -> Result<GenerationResponse, String> {
        // Get the document
        let document = self.get_document(&request.document_uri)?;
        
        // Get the options
        let options = request.options.unwrap_or_default();
        
        // Get or create the AST
        let ast = if let Some(ast_node) = request.ast {
            ast_node
        } else if let Some(parse_result) = &request.parse_result {
            parse_result.ast.clone()
        } else {
            // Parse the document
            let parse_result = self.parse_document(&document)?;
            parse_result.ast
        };
        
        // Generate the code
        let refactoring_provider = self.refactoring_provider.lock().unwrap();
        
        let (edit, generated_code) = match request.generation_type {
            GenerationType::Function => {
                let function_name = request.parameters.get("functionName")
                    .ok_or_else(|| "Missing 'functionName' parameter".to_string())?;
                
                let parameters = request.parameters.get("parameters").cloned().unwrap_or_default();
                let return_type = request.parameters.get("returnType").cloned();
                
                refactoring_provider.generate_function(&document, request.position, function_name, &parameters, return_type.as_deref())?
            },
            GenerationType::Class => {
                let class_name = request.parameters.get("className")
                    .ok_or_else(|| "Missing 'className' parameter".to_string())?;
                
                let properties = request.parameters.get("properties").cloned().unwrap_or_default();
                let methods = request.parameters.get("methods").cloned().unwrap_or_default();
                
                refactoring_provider.generate_class(&document, request.position, class_name, &properties, &methods)?
            },
            GenerationType::Interface => {
                let interface_name = request.parameters.get("interfaceName")
                    .ok_or_else(|| "Missing 'interfaceName' parameter".to_string())?;
                
                let properties = request.parameters.get("properties").cloned().unwrap_or_default();
                let methods = request.parameters.get("methods").cloned().unwrap_or_default();
                
                refactoring_provider.generate_interface(&document, request.position, interface_name, &properties, &methods)?
            },
            GenerationType::Enum => {
                let enum_name = request.parameters.get("enumName")
                    .ok_or_else(|| "Missing 'enumName' parameter".to_string())?;
                
                let values = request.parameters.get("values").cloned().unwrap_or_default();
                
                refactoring_provider.generate_enum(&document, request.position, enum_name, &values)?
            },
            GenerationType::Module => {
                let module_name = request.parameters.get("moduleName")
                    .ok_or_else(|| "Missing 'moduleName' parameter".to_string())?;
                
                let exports = request.parameters.get("exports").cloned().unwrap_or_default();
                
                refactoring_provider.generate_module(&document, request.position, module_name, &exports)?
            },
            GenerationType::Test => {
                let test_name = request.parameters.get("testName")
                    .ok_or_else(|| "Missing 'testName' parameter".to_string())?;
                
                let test_framework = request.parameters.get("testFramework").cloned().unwrap_or_else(|| "jest".to_string());
                
                refactoring_provider.generate_test(&document, request.position, test_name, &test_framework)?
            },
            GenerationType::Documentation => {
                refactoring_provider.generate_documentation(&document, request.position)?
            },
            GenerationType::Custom => {
                let generation_name = request.parameters.get("generationName")
                    .ok_or_else(|| "Missing 'generationName' parameter".to_string())?;
                
                match generation_name.as_str() {
                    "generateGetter" => {
                        let property_name = request.parameters.get("propertyName")
                            .ok_or_else(|| "Missing 'propertyName' parameter".to_string())?;
                        
                        refactoring_provider.generate_getter(&document, request.position, property_name)?
                    },
                    "generateSetter" => {
                        let property_name = request.parameters.get("propertyName")
                            .ok_or_else(|| "Missing 'propertyName' parameter".to_string())?;
                        
                        refactoring_provider.generate_setter(&document, request.position, property_name)?
                    },
                    "generateConstructor" => {
                        let properties = request.parameters.get("properties").cloned().unwrap_or_default();
                        
                        refactoring_provider.generate_constructor(&document, request.position, &properties)?
                    },
                    _ => return Err(format!("Unknown custom generation: {}", generation_name)),
                }
            },
        };
        
        // Create the response
        let response = GenerationResponse {
            edit,
            success: true,
            error_message: None,
            generated_code,
        };
        
        Ok(response)
    }
    
    /// Get document
    fn get_document(&self, uri: &str) -> Result<Document, String> {
        let document_manager = self.document_manager.lock().unwrap();
        document_manager.get_document(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))
            .map(|doc| doc.clone())
    }
    
    /// Parse document
    fn parse_document(&self, document: &Document) -> Result<ParseResult, String> {
        // This is a simplified implementation
        // In a real implementation, we would use the parser to parse the document
        
        // For now, we'll just return a dummy parse result
        Ok(ParseResult {
            ast: AstNode {
                node_type: "Program".to_string(),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: document.line_count() as u32, character: 0 },
                },
                children: Vec::new(),
                properties: HashMap::new(),
            },
            errors: Vec::new(),
        })
    }
    
    /// Parse position from string
    fn parse_position(&self, position_str: &str) -> Result<Position, String> {
        // Parse position in format "line:character"
        let parts: Vec<&str> = position_str.split(':').collect();
        
        if parts.len() != 2 {
            return Err(format!("Invalid position format: {}", position_str));
        }
        
        let line = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid line number: {}", parts[0]))?;
        
        let character = parts[1].parse::<u32>()
            .map_err(|_| format!("Invalid character number: {}", parts[1]))?;
        
        Ok(Position { line, character })
    }
    
    /// Parse range from string
    fn parse_range(&self, range_str: &str) -> Result<Range, String> {
        // Parse range in format "startLine:startCharacter:endLine:endCharacter"
        let parts: Vec<&str> = range_str.split(':').collect();
        
        if parts.len() != 4 {
            return Err(format!("Invalid range format: {}", range_str));
        }
        
        let start_line = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid start line number: {}", parts[0]))?;
        
        let start_character = parts[1].parse::<u32>()
            .map_err(|_| format!("Invalid start character number: {}", parts[1]))?;
        
        let end_line = parts[2].parse::<u32>()
            .map_err(|_| format!("Invalid end line number: {}", parts[2]))?;
        
        let end_character = parts[3].parse::<u32>()
            .map_err(|_| format!("Invalid end character number: {}", parts[3]))?;
        
        Ok(Range {
            start: Position { line: start_line, character: start_character },
            end: Position { line: end_line, character: end_character },
        })
    }
}

/// Shared AST manipulation endpoints that can be used across threads
pub type SharedAstManipulationEndpoints = Arc<Mutex<AstManipulationEndpoints>>;

/// Create a new shared AST manipulation endpoints
pub fn create_shared_ast_manipulation_endpoints(
    document_manager: SharedDocumentManager,
    refactoring_provider: SharedRefactoringProvider
) -> SharedAstManipulationEndpoints {
    Arc::new(Mutex::new(AstManipulationEndpoints::new(
        document_manager,
        refactoring_provider
    )))
}
