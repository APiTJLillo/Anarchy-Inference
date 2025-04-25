// src/external_tools/search.rs - Search interface for external tools

use std::collections::HashMap;
use std::path::Path;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use crate::value::Value;
use super::common::{ExternalTool, ToolParams, ToolResult, ToolStatus, ToolError, ToolContext};

/// Search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Time range (e.g., "day", "week", "month", "year")
    pub time_range: Option<String>,
    
    /// Site restriction (e.g., "example.com")
    pub site: Option<String>,
    
    /// File type (e.g., "pdf", "doc")
    pub file_type: Option<String>,
    
    /// Language (e.g., "en", "fr")
    pub language: Option<String>,
    
    /// Safe search (true for enabled)
    pub safe_search: Option<bool>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Title
    pub title: String,
    
    /// URL
    pub url: String,
    
    /// Snippet
    pub snippet: String,
    
    /// Source
    pub source: String,
}

/// Search results
#[derive(Debug, Clone)]
pub struct SearchResults {
    /// Query
    pub query: String,
    
    /// Results
    pub results: Vec<SearchResult>,
    
    /// Total count
    pub total_count: usize,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Web search client
pub struct WebSearchClient {
    /// HTTP client
    http_client: reqwest::Client,
    
    /// API key
    api_key: Option<String>,
    
    /// API endpoint
    api_endpoint: String,
}

impl WebSearchClient {
    /// Create a new web search client
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_key: None,
            api_endpoint: "https://api.search.example.com".to_string(),
        }
    }
    
    /// Set the API key
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    /// Set the API endpoint
    pub fn with_api_endpoint<S: Into<String>>(mut self, api_endpoint: S) -> Self {
        self.api_endpoint = api_endpoint.into();
        self
    }
    
    /// Search the web
    pub async fn search(&self, query: &str, max_results: Option<usize>, filters: Option<SearchFilters>) -> Result<SearchResults, ToolError> {
        // Check if API key is set
        if self.api_key.is_none() {
            return Err(ToolError::new(401, "API key not set for web search"));
        }
        
        // Build request URL
        let url = format!("{}/search", self.api_endpoint);
        
        // Build request body
        let mut body = HashMap::new();
        body.insert("query", query);
        body.insert("max_results", &max_results.unwrap_or(10).to_string());
        
        if let Some(filters) = filters {
            if let Some(time_range) = filters.time_range {
                body.insert("time_range", &time_range);
            }
            if let Some(site) = filters.site {
                body.insert("site", &site);
            }
            if let Some(file_type) = filters.file_type {
                body.insert("file_type", &file_type);
            }
            if let Some(language) = filters.language {
                body.insert("language", &language);
            }
            if let Some(safe_search) = filters.safe_search {
                body.insert("safe_search", &safe_search.to_string());
            }
        }
        
        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        if let Some(api_key) = &self.api_key {
            headers.insert("X-API-Key", HeaderValue::from_str(api_key).unwrap());
        }
        
        // Send request
        let response = self.http_client.post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ToolError::new(500, format!("Failed to send search request: {}", e)))?;
        
        // Check status
        if !response.status().is_success() {
            return Err(ToolError::new(
                response.status().as_u16() as u32,
                format!("Search API returned error: {}", response.status())
            ));
        }
        
        // Parse response
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| ToolError::new(500, format!("Failed to parse search response: {}", e)))?;
        
        // Extract results
        let results = response_json["results"].as_array()
            .ok_or_else(|| ToolError::new(500, "Invalid search response format"))?;
        
        let mut search_results = Vec::new();
        for result in results {
            let title = result["title"].as_str().unwrap_or("").to_string();
            let url = result["url"].as_str().unwrap_or("").to_string();
            let snippet = result["snippet"].as_str().unwrap_or("").to_string();
            let source = result["source"].as_str().unwrap_or("web").to_string();
            
            search_results.push(SearchResult {
                title,
                url,
                snippet,
                source,
            });
        }
        
        // Extract metadata
        let mut metadata = HashMap::new();
        if let Some(meta) = response_json["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(value_str) = value.as_str() {
                    metadata.insert(key.clone(), value_str.to_string());
                }
            }
        }
        
        // Create search results
        let total_count = response_json["total_count"].as_u64().unwrap_or(0) as usize;
        
        Ok(SearchResults {
            query: query.to_string(),
            results: search_results,
            total_count,
            metadata,
        })
    }
}

/// Local search index
pub struct LocalSearchIndex {
    /// Index path
    index_path: std::path::PathBuf,
    
    /// Indexed files
    indexed_files: HashMap<String, Vec<String>>,
}

impl LocalSearchIndex {
    /// Create a new local search index
    pub fn new<P: AsRef<Path>>(index_path: P) -> Self {
        Self {
            index_path: index_path.as_ref().to_path_buf(),
            indexed_files: HashMap::new(),
        }
    }
    
    /// Index a file
    pub fn index_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ToolError> {
        let path = path.as_ref();
        
        // Read file
        let content = std::fs::read_to_string(path)
            .map_err(|e| ToolError::new(500, format!("Failed to read file: {}", e)))?;
        
        // Tokenize content
        let tokens: Vec<String> = content
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        
        // Store in index
        let path_str = path.to_string_lossy().to_string();
        self.indexed_files.insert(path_str, tokens);
        
        Ok(())
    }
    
    /// Search the index
    pub fn search(&self, query: &str, max_results: Option<usize>) -> Result<SearchResults, ToolError> {
        let query = query.to_lowercase();
        let query_tokens: Vec<&str> = query.split_whitespace().collect();
        
        // Search each file
        let mut results = Vec::new();
        for (path, tokens) in &self.indexed_files {
            let mut matches = 0;
            for query_token in &query_tokens {
                if tokens.iter().any(|t| t.contains(query_token)) {
                    matches += 1;
                }
            }
            
            // If at least one token matches, add to results
            if matches > 0 {
                // Read file to get a snippet
                let content = match std::fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(_) => continue,
                };
                
                // Find a snippet containing the query
                let mut snippet = String::new();
                let lines: Vec<&str> = content.lines().collect();
                for line in &lines {
                    if line.to_lowercase().contains(&query) {
                        snippet = line.to_string();
                        break;
                    }
                }
                
                // If no specific line contains the query, use the first few lines
                if snippet.is_empty() && !lines.is_empty() {
                    snippet = lines.iter().take(3).map(|&s| s.to_string()).collect::<Vec<String>>().join(" ");
                }
                
                // Truncate snippet if too long
                if snippet.len() > 200 {
                    snippet = format!("{}...", &snippet[0..197]);
                }
                
                results.push(SearchResult {
                    title: Path::new(path).file_name().unwrap_or_default().to_string_lossy().to_string(),
                    url: format!("file://{}", path),
                    snippet,
                    source: "local".to_string(),
                });
            }
        }
        
        // Sort results by number of matches (descending)
        results.sort_by(|a, b| {
            let a_matches = query_tokens.iter().filter(|&t| a.snippet.to_lowercase().contains(t)).count();
            let b_matches = query_tokens.iter().filter(|&t| b.snippet.to_lowercase().contains(t)).count();
            b_matches.cmp(&a_matches)
        });
        
        // Limit results
        if let Some(max) = max_results {
            results.truncate(max);
        }
        
        Ok(SearchResults {
            query: query.to_string(),
            results,
            total_count: results.len(),
            metadata: HashMap::new(),
        })
    }
}

/// Knowledge base client
pub struct KnowledgeBaseClient {
    /// HTTP client
    http_client: reqwest::Client,
    
    /// API key
    api_key: Option<String>,
    
    /// API endpoint
    api_endpoint: String,
}

impl KnowledgeBaseClient {
    /// Create a new knowledge base client
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_key: None,
            api_endpoint: "https://api.kb.example.com".to_string(),
        }
    }
    
    /// Set the API key
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    /// Set the API endpoint
    pub fn with_api_endpoint<S: Into<String>>(mut self, api_endpoint: S) -> Self {
        self.api_endpoint = api_endpoint.into();
        self
    }
    
    /// Search the knowledge base
    pub async fn search(&self, query: &str, kb_id: &str, max_results: Option<usize>) -> Result<SearchResults, ToolError> {
        // Check if API key is set
        if self.api_key.is_none() {
            return Err(ToolError::new(401, "API key not set for knowledge base search"));
        }
        
        // Build request URL
        let url = format!("{}/search", self.api_endpoint);
        
        // Build request body
        let mut body = HashMap::new();
        body.insert("query", query);
        body.insert("kb_id", kb_id);
        body.insert("max_results", &max_results.unwrap_or(10).to_string());
        
        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        if let Some(api_key) = &self.api_key {
            headers.insert("X-API-Key", HeaderValue::from_str(api_key).unwrap());
        }
        
        // Send request
        let response = self.http_client.post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| ToolError::new(500, format!("Failed to send knowledge base search request: {}", e)))?;
        
        // Check status
        if !response.status().is_success() {
            return Err(ToolError::new(
                response.status().as_u16() as u32,
                format!("Knowledge base API returned error: {}", response.status())
            ));
        }
        
        // Parse response
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| ToolError::new(500, format!("Failed to parse knowledge base search response: {}", e)))?;
        
        // Extract results
        let results = response_json["results"].as_array()
            .ok_or_else(|| ToolError::new(500, "Invalid knowledge base search response format"))?;
        
        let mut search_results = Vec::new();
        for result in results {
            let title = result["title"].as_str().unwrap_or("").to_string();
            let url = result["url"].as_str().unwrap_or("").to_string();
            let snippet = result["snippet"].as_str().unwrap_or("").to_string();
            
            search_results.push(SearchResult {
                title,
                url,
                snippet,
                source: "knowledge_base".to_string(),
            });
        }
        
        // Extract metadata
        let mut metadata = HashMap::new();
        if let Some(meta) = response_json["metadata"].as_object() {
            for (key, value) in meta {
                if let Some(value_str) = value.as_str() {
                    metadata.insert(key.clone(), value_str.to_string());
                }
            }
        }
        
        // Create search results
        let total_count = response_json["total_count"].as_u64().unwrap_or(0) as usize;
        
        Ok(SearchResults {
            query: query.to_string(),
            results: search_results,
            total_count,
            metadata,
        })
    }
}

/// Search tool for web and local content search
pub struct SearchTool {
    /// Web search client
    web_search_client: WebSearchClient,
    
    /// Local search index
    local_search_index: Option<LocalSearchIndex>,
    
    /// Knowledge base client
    knowledge_base_client: Option<KnowledgeBaseClient>,
}

impl SearchTool {
    /// Create a new search tool
    pub fn new() -> Self {
        Self {
            web_search_client: WebSearchClient::new(),
            local_search_index: None,
            knowledge_base_client: None,
        }
    }
    
    /// Set the web search API key
    pub fn with_web_search_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.web_search_client = self.web_search_client.with_api_key(api_key);
        self
    }
    
    /// Set the web search API endpoint
    pub fn with_web_search_api_endpoint<S: Into<String>>(mut self, api_endpoint: S) -> Self {
        self.web_search_client = self.web_search_client.with_api_endpoint(api_endpoint);
        self
    }
    
    /// Set the local search index
    pub fn with_local_search_index<P: AsRef<Path>>(mut self, index_path: P) -> Self {
        self.local_search_index = Some(LocalSearchIndex::new(index_path));
        self
    }
    
    /// Set the knowledge base client
    pub fn with_knowledge_base_client(mut self, client: KnowledgeBaseClient) -> Self {
        self.knowledge_base_client = Some(client);
        self
    }
    
    /// Search the web
    pub async fn search_web(&self, query: &str, max_results: Option<usize>, filters: Option<SearchFilters>) -> Result<SearchResults, ToolError> {
        self.web_search_client.search(query, max_results, filters).await
    }
    
    /// Search local content
    pub fn search_local(&self, query: &str, max_results: Option<usize>) -> Result<SearchResults, ToolError> {
        match &self.local_search_index {
            Some(index) => index.search(query, max_results),
            None => Err(ToolError::new(400, "Local search index not initialized")),
        }
    }
    
    /// Search knowledge base
    pub async fn search_knowledge_base(&self, query: &str, kb_id: &str, max_results: Option<usize>) -> Result<SearchResults, ToolError> {
        match &self.knowledge_base_client {
            Some(client) => client.search(query, kb_id, max_results).await,
            None => Err(ToolError::new(400, "Knowledge base client not initialized")),
        }
    }
}

impl ExternalTool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }
    
    fn description(&self) -> &str {
        "Search tool for web, local content, and knowledge base search"
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn execute(&self, params: &ToolParams) -> Result<ToolResult, ToolError> {
        // Get command
        let command = params.command.as_str();
        
        // Execute command
        match command {
            "web" => {
                // Get parameters
                let query = params.get_string("query").ok_or_else(|| ToolError::new(400, "Missing query parameter"))?;
                let max_results = params.get::<usize>("max_results");
                
                // Get filters
                let mut filters = SearchFilters {
                    time_range: params.get_string("time_range"),
                    site: params.get_string("site"),
                    file_type: params.get_string("file_type"),
                    language: params.get_string("language"),
                    safe_search: params.get::<bool>("safe_search"),
                };
                
                // Create future for async execution
                let future = self.search_web(&query, max_results, Some(filters));
                
                // Execute future
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::new(500, format!("Failed to create runtime: {}", e)))?;
                
                let results = runtime.block_on(future)?;
                
                // Convert results to tool result
                let mut result_data = HashMap::new();
                result_data.insert("query".to_string(), Value::string(results.query));
                result_data.insert("total_count".to_string(), Value::number(results.total_count as f64));
                
                let result_items = results.results.iter().map(|r| {
                    let mut item = HashMap::new();
                    item.insert("title".to_string(), Value::string(r.title.clone()));
                    item.insert("url".to_string(), Value::string(r.url.clone()));
                    item.insert("snippet".to_string(), Value::string(r.snippet.clone()));
                    item.insert("source".to_string(), Value::string(r.source.clone()));
                    Value::object(item)
                }).collect::<Vec<Value>>();
                
                result_data.insert("results".to_string(), Value::array(result_items));
                
                let metadata = results.metadata.iter().map(|(k, v)| {
                    (k.clone(), Value::string(v.clone()))
                }).collect();
                
                Ok(ToolResult::success(Value::object(result_data))
                    .with_metadata("metadata", Value::object(metadata)))
            },
            "local" => {
                // Get parameters
                let query = params.get_string("query").ok_or_else(|| ToolError::new(400, "Missing query parameter"))?;
                let max_results = params.get::<usize>("max_results");
                
                // Search local content
                let results = self.search_local(&query, max_results)?;
                
                // Convert results to tool result
                let mut result_data = HashMap::new();
                result_data.insert("query".to_string(), Value::string(results.query));
                result_data.insert("total_count".to_string(), Value::number(results.total_count as f64));
                
                let result_items = results.results.iter().map(|r| {
                    let mut item = HashMap::new();
                    item.insert("title".to_string(), Value::string(r.title.clone()));
                    item.insert("url".to_string(), Value::string(r.url.clone()));
                    item.insert("snippet".to_string(), Value::string(r.snippet.clone()));
                    item.insert("source".to_string(), Value::string(r.source.clone()));
                    Value::object(item)
                }).collect::<Vec<Value>>();
                
                result_data.insert("results".to_string(), Value::array(result_items));
                
                Ok(ToolResult::success(Value::object(result_data)))
            },
            "knowledge_base" => {
                // Get parameters
                let query = params.get_string("query").ok_or_else(|| ToolError::new(400, "Missing query parameter"))?;
                let kb_id = params.get_string("kb_id").ok_or_else(|| ToolError::new(400, "Missing kb_id parameter"))?;
                let max_results = params.get::<usize>("max_results");
                
                // Create future for async execution
                let future = self.search_knowledge_base(&query, &kb_id, max_results);
                
                // Execute future
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::new(500, format!("Failed to create runtime: {}", e)))?;
                
                let results = runtime.block_on(future)?;
                
                // Convert results to tool result
                let mut result_data = HashMap::new();
                result_data.insert("query".to_string(), Value::string(results.query));
                result_data.insert("total_count".to_string(), Value::number(results.total_count as f64));
                
                let result_items = results.results.iter().map(|r| {
                    let mut item = HashMap::new();
                    item.insert("title".to_string(), Value::string(r.title.clone()));
                    item.insert("url".to_string(), Value::string(r.url.clone()));
                    item.insert("snippet".to_string(), Value::string(r.snippet.clone()));
                    item.insert("source".to_string(), Value::string(r.source.clone()));
                    Value::object(item)
                }).collect::<Vec<Value>>();
                
                result_data.insert("results".to_string(), Value::array(result_items));
                
                let metadata = results.metadata.iter().map(|(k, v)| {
                    (k.clone(), Value::string(v.clone()))
                }).collect();
                
                Ok(ToolResult::success(Value::object(result_data))
                    .with_metadata("metadata", Value::object(metadata)))
            },
            _ => Err(ToolError::new(400, format!("Unknown command: {}", command))),
        }
    }
}
