// Documentation Agent module for Anarchy Inference
//
// This module provides contextual help, reference information, and examples
// based on user queries or code context.

use super::{
    OnboardingContext, 
    DocumentationTopic,
    CodeExample
};
use crate::ast::{Ast, AstNode};
use crate::parser::Parser;
use crate::lexer::Lexer;
use std::collections::{HashMap, HashSet};

/// Agent for providing documentation and contextual help
pub struct DocumentationAgent {
    /// Search engine for documentation
    search_engine: DocumentationSearchEngine,
    
    /// Context analyzer for code-aware documentation
    context_analyzer: ContextAnalyzer,
    
    /// Example provider for code examples
    example_provider: ExampleProvider,
}

/// Engine for searching documentation
struct DocumentationSearchEngine {
    /// Index of documentation topics
    topic_index: HashMap<String, Vec<String>>,
    
    /// Index of code examples
    example_index: HashMap<String, Vec<String>>,
}

/// Analyzer for code context
struct ContextAnalyzer {
    /// Symbol extractor
    symbol_extractor: SymbolExtractor,
    
    /// Pattern recognizer
    pattern_recognizer: PatternRecognizer,
}

/// Extractor for symbols in code
struct SymbolExtractor {
    /// Known symbols
    known_symbols: HashSet<String>,
}

/// Recognizer for patterns in code
struct PatternRecognizer {
    /// Known patterns
    known_patterns: HashMap<String, fn(&Ast) -> bool>,
}

/// Provider for code examples
struct ExampleProvider {
    /// Example generator
    example_generator: ExampleGenerator,
}

/// Generator for code examples
struct ExampleGenerator {
    /// Template-based examples
    templates: HashMap<String, String>,
}

/// Search result for documentation
pub struct DocumentationSearchResult {
    /// Matching topics
    pub topics: Vec<DocumentationTopic>,
    
    /// Matching examples
    pub examples: Vec<CodeExample>,
    
    /// Relevance scores
    pub relevance_scores: HashMap<String, f64>,
}

impl DocumentationAgent {
    /// Create a new documentation agent
    pub fn new() -> Self {
        DocumentationAgent {
            search_engine: DocumentationSearchEngine {
                topic_index: HashMap::new(),
                example_index: HashMap::new(),
            },
            context_analyzer: ContextAnalyzer {
                symbol_extractor: SymbolExtractor {
                    known_symbols: HashSet::new(),
                },
                pattern_recognizer: PatternRecognizer {
                    known_patterns: HashMap::new(),
                },
            },
            example_provider: ExampleProvider {
                example_generator: ExampleGenerator {
                    templates: HashMap::new(),
                },
            },
        }
    }
    
    /// Initialize the documentation agent
    pub fn initialize(&mut self, context: &OnboardingContext) {
        // Build search indices
        self.build_indices(context);
        
        // Initialize known symbols
        self.initialize_known_symbols();
        
        // Initialize pattern recognizers
        self.initialize_pattern_recognizers();
        
        // Initialize example templates
        self.initialize_example_templates();
    }
    
    /// Build search indices
    fn build_indices(&mut self, context: &OnboardingContext) {
        // Build topic index
        for (id, topic) in &context.knowledge_base.documentation {
            // Index title words
            for word in topic.title.split_whitespace() {
                let word = word.to_lowercase();
                let entry = self.search_engine.topic_index.entry(word).or_insert_with(Vec::new);
                if !entry.contains(id) {
                    entry.push(id.clone());
                }
            }
            
            // Index content words (simplified implementation)
            for word in topic.content.split_whitespace().take(100) {
                let word = word.to_lowercase();
                if word.len() > 3 { // Skip short words
                    let entry = self.search_engine.topic_index.entry(word).or_insert_with(Vec::new);
                    if !entry.contains(id) {
                        entry.push(id.clone());
                    }
                }
            }
        }
        
        // Build example index
        for (id, example) in &context.knowledge_base.code_examples {
            // Index title and description
            for text in [&example.title, &example.description] {
                for word in text.split_whitespace() {
                    let word = word.to_lowercase();
                    if word.len() > 3 { // Skip short words
                        let entry = self.search_engine.example_index.entry(word).or_insert_with(Vec::new);
                        if !entry.contains(id) {
                            entry.push(id.clone());
                        }
                    }
                }
            }
            
            // Index tags
            for tag in &example.tags {
                let entry = self.search_engine.example_index.entry(tag.to_lowercase()).or_insert_with(Vec::new);
                if !entry.contains(id) {
                    entry.push(id.clone());
                }
            }
        }
    }
    
    /// Initialize known symbols
    fn initialize_known_symbols(&mut self) {
        // Add built-in functions
        let builtins = vec![
            "print", "println", "input", "len", "type", "str", "int", "float", 
            "bool", "list", "dict", "set", "range", "map", "filter", "reduce"
        ];
        
        for symbol in builtins {
            self.context_analyzer.symbol_extractor.known_symbols.insert(symbol.to_string());
        }
    }
    
    /// Initialize pattern recognizers
    fn initialize_pattern_recognizers(&mut self) {
        // Add pattern recognizers
        self.context_analyzer.pattern_recognizer.known_patterns.insert(
            "loop".to_string(),
            |ast| {
                // Simplified implementation
                true
            }
        );
        
        // Add more pattern recognizers as needed
    }
    
    /// Initialize example templates
    fn initialize_example_templates(&mut self) {
        // Add example templates
        self.example_provider.example_generator.templates.insert(
            "function".to_string(),
            "fn {name}({params}) -> {return_type} {\n    {body}\n}".to_string()
        );
        
        // Add more templates as needed
    }
    
    /// Search documentation for a query
    pub fn search_documentation(&self, context: &OnboardingContext, query: &str) -> Vec<&DocumentationTopic> {
        let mut topic_scores: HashMap<&str, f64> = HashMap::new();
        
        // Process query words
        for word in query.split_whitespace() {
            let word = word.to_lowercase();
            
            // Look up in topic index
            if let Some(topic_ids) = self.search_engine.topic_index.get(&word) {
                for id in topic_ids {
                    let score = topic_scores.entry(id).or_insert(0.0);
                    *score += 1.0;
                }
            }
        }
        
        // Sort topics by score
        let mut scored_topics: Vec<(&str, f64)> = topic_scores.into_iter().collect();
        scored_topics.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Get top topics
        let mut result = Vec::new();
        for (id, _) in scored_topics.iter().take(5) {
            if let Some(topic) = context.knowledge_base.documentation.get(*id) {
                result.push(topic);
            }
        }
        
        result
    }
    
    /// Get documentation for a symbol
    pub fn get_symbol_documentation(&self, context: &OnboardingContext, symbol: &str) -> Option<&DocumentationTopic> {
        // Look for exact match in documentation
        for (id, topic) in &context.knowledge_base.documentation {
            if topic.title.to_lowercase() == symbol.to_lowercase() {
                return Some(topic);
            }
        }
        
        // If no exact match, search for partial matches
        let results = self.search_documentation(context, symbol);
        results.first().copied()
    }
    
    /// Get examples for a topic
    pub fn get_examples_for_topic(&self, context: &OnboardingContext, topic_id: &str) -> Vec<&CodeExample> {
        let mut result = Vec::new();
        
        // Get the topic
        let topic = match context.knowledge_base.documentation.get(topic_id) {
            Some(topic) => topic,
            None => return result,
        };
        
        // Get examples from the topic
        for example_id in topic.examples.iter().filter_map(|e| {
            if let Some(id) = e.id.as_ref() {
                Some(id.as_str())
            } else {
                None
            }
        }) {
            if let Some(example) = context.knowledge_base.code_examples.get(example_id) {
                result.push(example);
            }
        }
        
        result
    }
    
    /// Get contextual help for code
    pub fn get_contextual_help(&self, context: &OnboardingContext, code: &str, cursor_position: usize) -> Vec<&DocumentationTopic> {
        // Parse the code
        let lexer = Lexer::new(code);
        let parser = Parser::new(lexer);
        let ast = parser.parse();
        
        // Extract symbols around cursor position
        let symbols = self.extract_symbols_at_position(&ast, cursor_position);
        
        // Get documentation for symbols
        let mut result = Vec::new();
        for symbol in symbols {
            if let Some(topic) = self.get_symbol_documentation(context, &symbol) {
                result.push(topic);
            }
        }
        
        result
    }
    
    /// Extract symbols at a position in code
    fn extract_symbols_at_position(&self, ast: &Ast, position: usize) -> Vec<String> {
        // Simplified implementation
        vec!["example_symbol".to_string()]
    }
    
    /// Generate an example for a topic
    pub fn generate_example(&self, context: &OnboardingContext, topic: &str, complexity: usize) -> Option<String> {
        // Find relevant template
        let template = self.example_provider.example_generator.templates.get(topic)?;
        
        // Generate example (simplified implementation)
        let example = template
            .replace("{name}", "example_function")
            .replace("{params}", "x: int, y: int")
            .replace("{return_type}", "int")
            .replace("{body}", "    return x + y");
        
        Some(example)
    }
    
    /// Get related topics
    pub fn get_related_topics(&self, context: &OnboardingContext, topic_id: &str) -> Vec<&DocumentationTopic> {
        let mut result = Vec::new();
        
        // Get the topic
        let topic = match context.knowledge_base.documentation.get(topic_id) {
            Some(topic) => topic,
            None => return result,
        };
        
        // Get related topics
        for related_id in &topic.related_topics {
            if let Some(related_topic) = context.knowledge_base.documentation.get(related_id) {
                result.push(related_topic);
            }
        }
        
        result
    }
    
    /// Search for examples
    pub fn search_examples(&self, context: &OnboardingContext, query: &str) -> Vec<&CodeExample> {
        let mut example_scores: HashMap<&str, f64> = HashMap::new();
        
        // Process query words
        for word in query.split_whitespace() {
            let word = word.to_lowercase();
            
            // Look up in example index
            if let Some(example_ids) = self.search_engine.example_index.get(&word) {
                for id in example_ids {
                    let score = example_scores.entry(id).or_insert(0.0);
                    *score += 1.0;
                }
            }
        }
        
        // Sort examples by score
        let mut scored_examples: Vec<(&str, f64)> = example_scores.into_iter().collect();
        scored_examples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Get top examples
        let mut result = Vec::new();
        for (id, _) in scored_examples.iter().take(5) {
            if let Some(example) = context.knowledge_base.code_examples.get(*id) {
                result.push(example);
            }
        }
        
        result
    }
    
    /// Get FAQ for a topic
    pub fn get_faq(&self, context: &OnboardingContext, topic_id: &str) -> Vec<(String, String)> {
        // Get the topic
        let topic = match context.knowledge_base.documentation.get(topic_id) {
            Some(topic) => topic,
            None => return Vec::new(),
        };
        
        // Return FAQ
        topic.faq.clone()
    }
}
