// Debug Agent - Fix Suggester Module
// This module provides functionality for suggesting fixes for common errors

use crate::error::Error;
use crate::ast::AstNode;
use crate::debug::error_analyzer::{ErrorInfo, ErrorType, ErrorAnalysis, SourceLocation};
use std::collections::HashMap;
use std::fmt;

/// Fix suggestion for an error
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub id: FixId,
    pub error_info: ErrorInfo,
    pub description: String,
    pub code_change: CodeChange,
    pub confidence: FixConfidence,
    pub explanation: String,
}

/// Unique identifier for fix suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixId(pub usize);

/// Code change to apply
#[derive(Debug, Clone)]
pub enum CodeChange {
    /// Replace code at a specific location
    Replace {
        location: SourceLocation,
        old_code: String,
        new_code: String,
    },
    /// Insert code at a specific location
    Insert {
        location: SourceLocation,
        code: String,
    },
    /// Delete code at a specific location
    Delete {
        location: SourceLocation,
        code: String,
    },
    /// Multiple changes
    Multiple(Vec<CodeChange>),
}

/// Confidence level for a fix suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixConfidence {
    Low,
    Medium,
    High,
}

/// Fix pattern for matching errors
#[derive(Debug, Clone)]
pub struct FixPattern {
    pub pattern_id: String,
    pub error_type: ErrorType,
    pub message_pattern: String,
    pub code_pattern: Option<String>,
    pub fix_generator: FixGeneratorType,
    pub description: String,
    pub confidence: FixConfidence,
}

/// Type of fix generator
#[derive(Debug, Clone)]
pub enum FixGeneratorType {
    /// Simple replacement
    SimpleReplacement {
        old_pattern: String,
        new_pattern: String,
    },
    /// Add missing symbol
    AddMissingSymbol {
        symbol: String,
    },
    /// Fix variable reference
    FixVariableReference,
    /// Fix type mismatch
    FixTypeMismatch,
    /// Custom fix (requires special handling)
    Custom(String),
}

/// Context for analyzing code
#[derive(Debug, Clone)]
pub struct ContextAnalyzer {
    pub available_variables: HashMap<String, String>, // Variable name -> type
    pub available_functions: HashMap<String, String>, // Function name -> signature
    pub imported_modules: Vec<String>,
    pub language_features_used: Vec<String>,
}

/// Applied fix information
#[derive(Debug, Clone)]
pub struct AppliedFix {
    pub fix_id: FixId,
    pub timestamp: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Error that can occur when applying a fix
#[derive(Debug, Clone)]
pub enum FixError {
    InvalidLocation,
    CodeMismatch,
    CannotGenerateFix,
    ApplicationFailed(String),
}

impl fmt::Display for FixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixError::InvalidLocation => write!(f, "Invalid source location"),
            FixError::CodeMismatch => write!(f, "Code at location doesn't match expected code"),
            FixError::CannotGenerateFix => write!(f, "Cannot generate a fix for this error"),
            FixError::ApplicationFailed(msg) => write!(f, "Fix application failed: {}", msg),
        }
    }
}

/// Fix Suggester component
pub struct FixSuggester {
    /// Fix pattern database
    fix_patterns: HashMap<String, FixPattern>,
    /// Code context analyzer
    context_analyzer: ContextAnalyzer,
    /// Fix history
    applied_fixes: Vec<AppliedFix>,
    /// Next fix ID
    next_fix_id: usize,
    /// Current timestamp
    timestamp: u64,
}

impl FixSuggester {
    /// Create a new fix suggester
    pub fn new() -> Self {
        Self {
            fix_patterns: Self::initialize_fix_patterns(),
            context_analyzer: Self::initialize_context_analyzer(),
            applied_fixes: Vec::new(),
            next_fix_id: 1,
            timestamp: 0,
        }
    }

    /// Initialize the fix patterns database
    fn initialize_fix_patterns() -> HashMap<String, FixPattern> {
        let mut patterns = HashMap::new();
        
        // Add some common fix patterns
        
        // Undefined variable -> suggest similar variables
        patterns.insert(
            "undefined_variable_suggestion".to_string(),
            FixPattern {
                pattern_id: "undefined_variable_suggestion".to_string(),
                error_type: ErrorType::Reference,
                message_pattern: "undefined variable".to_string(),
                code_pattern: None,
                fix_generator: FixGeneratorType::FixVariableReference,
                description: "Suggest similar variable names".to_string(),
                confidence: FixConfidence::Medium,
            },
        );
        
        // Missing semicolon
        patterns.insert(
            "missing_semicolon".to_string(),
            FixPattern {
                pattern_id: "missing_semicolon".to_string(),
                error_type: ErrorType::Syntax,
                message_pattern: "expected ;".to_string(),
                code_pattern: None,
                fix_generator: FixGeneratorType::AddMissingSymbol {
                    symbol: ";".to_string(),
                },
                description: "Add missing semicolon".to_string(),
                confidence: FixConfidence::High,
            },
        );
        
        // Missing closing parenthesis
        patterns.insert(
            "missing_closing_paren".to_string(),
            FixPattern {
                pattern_id: "missing_closing_paren".to_string(),
                error_type: ErrorType::Syntax,
                message_pattern: "expected )".to_string(),
                code_pattern: None,
                fix_generator: FixGeneratorType::AddMissingSymbol {
                    symbol: ")".to_string(),
                },
                description: "Add missing closing parenthesis".to_string(),
                confidence: FixConfidence::High,
            },
        );
        
        // Type mismatch
        patterns.insert(
            "type_mismatch".to_string(),
            FixPattern {
                pattern_id: "type_mismatch".to_string(),
                error_type: ErrorType::Type,
                message_pattern: "expected type".to_string(),
                code_pattern: None,
                fix_generator: FixGeneratorType::FixTypeMismatch,
                description: "Fix type mismatch".to_string(),
                confidence: FixConfidence::Medium,
            },
        );
        
        patterns
    }

    /// Initialize the context analyzer
    fn initialize_context_analyzer() -> ContextAnalyzer {
        ContextAnalyzer {
            available_variables: HashMap::new(),
            available_functions: HashMap::new(),
            imported_modules: Vec::new(),
            language_features_used: Vec::new(),
        }
    }

    /// Suggest fixes for an error
    pub fn suggest_fixes(&mut self, error_analysis: &ErrorAnalysis) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();
        
        // Match error against fix patterns
        for pattern in self.fix_patterns.values() {
            if self.matches_fix_pattern(&error_analysis.error_info, pattern) {
                if let Some(suggestion) = self.generate_fix_suggestion(&error_analysis.error_info, pattern) {
                    suggestions.push(suggestion);
                }
            }
        }
        
        // Sort suggestions by confidence
        suggestions.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        
        suggestions
    }
    
    /// Check if an error matches a fix pattern
    fn matches_fix_pattern(&self, error_info: &ErrorInfo, pattern: &FixPattern) -> bool {
        // Check error type
        if error_info.error_type != pattern.error_type {
            return false;
        }
        
        // Check message pattern
        let error_message = format!("{:?}", error_info.error).to_lowercase();
        if !error_message.contains(&pattern.message_pattern.to_lowercase()) {
            return false;
        }
        
        // Check code pattern if present
        if let Some(code_pattern) = &pattern.code_pattern {
            if let Some(code_snippet) = &error_info.context.code_snippet {
                if !code_snippet.contains(code_pattern) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    /// Generate a fix suggestion for an error
    fn generate_fix_suggestion(&mut self, error_info: &ErrorInfo, pattern: &FixPattern) -> Option<FixSuggestion> {
        let fix_id = FixId(self.next_fix_id);
        self.next_fix_id += 1;
        
        let location = error_info.location.clone()?;
        let code_snippet = error_info.context.code_snippet.clone()?;
        
        let (code_change, explanation) = match &pattern.fix_generator {
            FixGeneratorType::SimpleReplacement { old_pattern, new_pattern } => {
                let new_code = code_snippet.replace(old_pattern, new_pattern);
                
                (
                    CodeChange::Replace {
                        location: location.clone(),
                        old_code: code_snippet.clone(),
                        new_code,
                    },
                    format!("Replace '{}' with '{}'", old_pattern, new_pattern),
                )
            },
            FixGeneratorType::AddMissingSymbol { symbol } => {
                (
                    CodeChange::Insert {
                        location: location.clone(),
                        code: symbol.clone(),
                    },
                    format!("Add missing '{}'", symbol),
                )
            },
            FixGeneratorType::FixVariableReference => {
                // Find similar variable names
                let error_message = format!("{:?}", error_info.error);
                let var_name = self.extract_variable_name(&error_message)?;
                let similar_vars = self.find_similar_variables(var_name);
                
                if let Some(best_match) = similar_vars.first() {
                    (
                        CodeChange::Replace {
                            location: location.clone(),
                            old_code: var_name.to_string(),
                            new_code: best_match.clone(),
                        },
                        format!("Replace '{}' with '{}'", var_name, best_match),
                    )
                } else {
                    return None;
                }
            },
            FixGeneratorType::FixTypeMismatch => {
                // This would require more context to implement properly
                // For now, just suggest a generic type conversion
                (
                    CodeChange::Replace {
                        location: location.clone(),
                        old_code: code_snippet.clone(),
                        new_code: format!("convert({})", code_snippet),
                    },
                    "Add type conversion".to_string(),
                )
            },
            FixGeneratorType::Custom(custom_id) => {
                // Handle custom fix generators
                match custom_id.as_str() {
                    // Add custom fix generators here
                    _ => return None,
                }
            },
        };
        
        Some(FixSuggestion {
            id: fix_id,
            error_info: error_info.clone(),
            description: pattern.description.clone(),
            code_change,
            confidence: pattern.confidence,
            explanation,
        })
    }
    
    /// Extract variable name from error message
    fn extract_variable_name(&self, error_message: &str) -> Option<&str> {
        // This is a simplified implementation
        // In a real implementation, we would use regex or more sophisticated parsing
        
        if let Some(idx = error_message.find("undefined variable")) {
            let start = idx + "undefined variable".len();
            if let Some(quote_start) = error_message[start..].find("'") {
                let name_start = start + quote_start + 1;
                if let Some(quote_end) = error_message[name_start..].find("'") {
                    return Some(&error_message[name_start..name_start + quote_end]);
                }
            }
        }
        
        None
    }
    
    /// Find similar variables to the given name
    fn find_similar_variables(&self, name: &str) -> Vec<String> {
        let mut similar_vars = Vec::new();
        
        for var_name in self.context_analyzer.available_variables.keys() {
            if self.is_similar(name, var_name) {
                similar_vars.push(var_name.clone());
            }
        }
        
        // Sort by similarity (most similar first)
        similar_vars.sort_by(|a, b| {
            let a_dist = self.levenshtein_distance(name, a);
            let b_dist = self.levenshtein_distance(name, b);
            a_dist.cmp(&b_dist)
        });
        
        similar_vars
    }
    
    /// Check if two strings are similar
    fn is_similar(&self, a: &str, b: &str) -> bool {
        let distance = self.levenshtein_distance(a, b);
        let max_len = a.len().max(b.len());
        
        // Consider similar if the distance is less than 1/3 of the length
        distance <= max_len / 3
    }
    
    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        
        let m = a_chars.len();
        let n = b_chars.len();
        
        let mut dp = vec![vec![0; n + 1]; m + 1];
        
        for i in 0..=m {
            dp[i][0] = i;
        }
        
        for j in 0..=n {
            dp[0][j] = j;
        }
        
        for i in 1..=m {
            for j in 1..=n {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }
        
        dp[m][n]
    }
    
    /// Apply a fix suggestion
    pub fn apply_fix(&mut self, suggestion: &FixSuggestion) -> Result<(), FixError> {
        self.timestamp += 1;
        
        // In a real implementation, this would modify the source code
        // For now, just record that the fix was applied
        
        let applied_fix = AppliedFix {
            fix_id: suggestion.id,
            timestamp: self.timestamp,
            success: true,
            error_message: None,
        };
        
        self.applied_fixes.push(applied_fix);
        
        Ok(())
    }
    
    /// Update the context analyzer with available variables
    pub fn update_available_variables(&mut self, variables: HashMap<String, String>) {
        self.context_analyzer.available_variables = variables;
    }
    
    /// Update the context analyzer with available functions
    pub fn update_available_functions(&mut self, functions: HashMap<String, String>) {
        self.context_analyzer.available_functions = functions;
    }
    
    /// Update the context analyzer with imported modules
    pub fn update_imported_modules(&mut self, modules: Vec<String>) {
        self.context_analyzer.imported_modules = modules;
    }
    
    /// Update the context analyzer with language features used
    pub fn update_language_features_used(&mut self, features: Vec<String>) {
        self.context_analyzer.language_features_used = features;
    }
    
    /// Get the applied fixes history
    pub fn get_applied_fixes(&self) -> &[AppliedFix] {
        &self.applied_fixes
    }
    
    /// Add a custom fix pattern
    pub fn add_fix_pattern(&mut self, pattern: FixPattern) {
        self.fix_patterns.insert(pattern.pattern_id.clone(), pattern);
    }
    
    /// Remove a fix pattern
    pub fn remove_fix_pattern(&mut self, pattern_id: &str) -> bool {
        self.fix_patterns.remove(pattern_id).is_some()
    }
    
    /// Get all fix patterns
    pub fn get_fix_patterns(&self) -> Vec<&FixPattern> {
        self.fix_patterns.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Add tests for fix suggestion
}
