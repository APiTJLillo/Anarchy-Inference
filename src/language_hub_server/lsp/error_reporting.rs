// Error reporting interface module for LSP-like Component
//
// This module provides a standardized interface for error reporting and management
// in Anarchy Inference code, offering JSON/gRPC interfaces for error handling.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::language_hub_server::lsp::protocol::{Position, Range, Diagnostic, DiagnosticSeverity};
use crate::language_hub_server::lsp::document::{Document, DocumentManager, SharedDocumentManager};
use crate::language_hub_server::lsp::checking_api::{CheckingApi, SharedCheckingApi, CheckingRequest, CheckingResponse};

/// Error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Fatal error that prevents execution
    Fatal = 0,
    
    /// Error that affects execution
    Error = 1,
    
    /// Warning that might affect execution
    Warning = 2,
    
    /// Information that doesn't affect execution
    Info = 3,
    
    /// Hint for improvement
    Hint = 4,
}

impl From<u8> for ErrorSeverity {
    fn from(value: u8) -> Self {
        match value {
            0 => ErrorSeverity::Fatal,
            1 => ErrorSeverity::Error,
            2 => ErrorSeverity::Warning,
            3 => ErrorSeverity::Info,
            4 => ErrorSeverity::Hint,
            _ => ErrorSeverity::Error,
        }
    }
}

impl From<ErrorSeverity> for u8 {
    fn from(severity: ErrorSeverity) -> Self {
        severity as u8
    }
}

impl From<DiagnosticSeverity> for ErrorSeverity {
    fn from(severity: DiagnosticSeverity) -> Self {
        match severity {
            DiagnosticSeverity::Error => ErrorSeverity::Error,
            DiagnosticSeverity::Warning => ErrorSeverity::Warning,
            DiagnosticSeverity::Information => ErrorSeverity::Info,
            DiagnosticSeverity::Hint => ErrorSeverity::Hint,
        }
    }
}

impl From<ErrorSeverity> for DiagnosticSeverity {
    fn from(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Fatal => DiagnosticSeverity::Error,
            ErrorSeverity::Error => DiagnosticSeverity::Error,
            ErrorSeverity::Warning => DiagnosticSeverity::Warning,
            ErrorSeverity::Info => DiagnosticSeverity::Information,
            ErrorSeverity::Hint => DiagnosticSeverity::Hint,
        }
    }
}

/// Error category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Syntax error
    Syntax,
    
    /// Semantic error
    Semantic,
    
    /// Type error
    Type,
    
    /// Runtime error
    Runtime,
    
    /// Import error
    Import,
    
    /// Performance issue
    Performance,
    
    /// Security issue
    Security,
    
    /// Style issue
    Style,
    
    /// Deprecation warning
    Deprecation,
    
    /// Other error
    Other,
}

impl ErrorCategory {
    /// Get the category from a diagnostic code
    pub fn from_code(code: Option<&str>) -> Self {
        match code {
            Some(code) if code.starts_with("syntax-") => ErrorCategory::Syntax,
            Some(code) if code.starts_with("semantic-") => ErrorCategory::Semantic,
            Some(code) if code.starts_with("type-") => ErrorCategory::Type,
            Some(code) if code.starts_with("runtime-") => ErrorCategory::Runtime,
            Some(code) if code.starts_with("import-") || code.contains("import") || code.contains("module") => ErrorCategory::Import,
            Some(code) if code.starts_with("perf-") || code.contains("performance") => ErrorCategory::Performance,
            Some(code) if code.starts_with("security-") || code.contains("security") => ErrorCategory::Security,
            Some(code) if code.starts_with("style-") || code.contains("style") => ErrorCategory::Style,
            Some(code) if code.starts_with("deprecated-") || code.contains("deprecated") => ErrorCategory::Deprecation,
            _ => ErrorCategory::Other,
        }
    }
    
    /// Get the category name
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::Syntax => "syntax",
            ErrorCategory::Semantic => "semantic",
            ErrorCategory::Type => "type",
            ErrorCategory::Runtime => "runtime",
            ErrorCategory::Import => "import",
            ErrorCategory::Performance => "performance",
            ErrorCategory::Security => "security",
            ErrorCategory::Style => "style",
            ErrorCategory::Deprecation => "deprecation",
            ErrorCategory::Other => "other",
        }
    }
}

/// Error location
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    /// The document URI
    pub uri: String,
    
    /// The range in the document
    pub range: Range,
    
    /// The line text
    pub line_text: Option<String>,
}

/// Error fix
#[derive(Debug, Clone)]
pub struct ErrorFix {
    /// The fix title
    pub title: String,
    
    /// The fix description
    pub description: Option<String>,
    
    /// The document URI
    pub uri: String,
    
    /// The range to replace
    pub range: Range,
    
    /// The replacement text
    pub new_text: String,
    
    /// Whether the fix is preferred
    pub is_preferred: bool,
}

/// Error report
#[derive(Debug, Clone)]
pub struct ErrorReport {
    /// The error ID
    pub id: String,
    
    /// The error code
    pub code: String,
    
    /// The error message
    pub message: String,
    
    /// The error severity
    pub severity: ErrorSeverity,
    
    /// The error category
    pub category: ErrorCategory,
    
    /// The error location
    pub location: ErrorLocation,
    
    /// The error source
    pub source: String,
    
    /// The error timestamp
    pub timestamp: u64,
    
    /// The error fixes
    pub fixes: Vec<ErrorFix>,
    
    /// The related information
    pub related_information: Vec<ErrorReport>,
    
    /// Additional data
    pub additional_data: HashMap<String, String>,
}

impl ErrorReport {
    /// Create a new error report from a diagnostic
    pub fn from_diagnostic(
        diagnostic: &Diagnostic,
        document_uri: &str,
        line_text: Option<String>
    ) -> Self {
        // Generate a unique ID
        let id = format!("{}-{}-{}", 
            document_uri.replace("/", "-"),
            diagnostic.range.start.line,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        
        // Get the code
        let code = diagnostic.code.clone().unwrap_or_else(|| "unknown".to_string());
        
        // Get the category
        let category = ErrorCategory::from_code(diagnostic.code.as_deref());
        
        // Get the severity
        let severity = if let Some(severity) = diagnostic.severity {
            ErrorSeverity::from(severity)
        } else {
            ErrorSeverity::Error
        };
        
        // Create the location
        let location = ErrorLocation {
            uri: document_uri.to_string(),
            range: diagnostic.range,
            line_text,
        };
        
        // Get the source
        let source = diagnostic.source.clone().unwrap_or_else(|| "anarchy-inference".to_string());
        
        // Get the timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create the error report
        ErrorReport {
            id,
            code,
            message: diagnostic.message.clone(),
            severity,
            category,
            location,
            source,
            timestamp,
            fixes: Vec::new(),
            related_information: Vec::new(),
            additional_data: HashMap::new(),
        }
    }
    
    /// Convert to JSON
    pub fn to_json(&self) -> String {
        // This is a simplified implementation
        // In a real implementation, we would use serde to serialize to JSON
        
        let fixes_json = self.fixes.iter()
            .map(|fix| format!(
                r#"{{
                    "title": "{}", 
                    "description": {}, 
                    "uri": "{}", 
                    "range": {}, 
                    "newText": "{}", 
                    "isPreferred": {}
                }}"#,
                fix.title.replace("\"", "\\\""),
                fix.description.as_ref().map_or("null".to_string(), |d| format!("\"{}\"", d.replace("\"", "\\\""))),
                fix.uri,
                self.range_to_json(&fix.range),
                fix.new_text.replace("\"", "\\\""),
                fix.is_preferred
            ))
            .collect::<Vec<_>>()
            .join(",");
        
        let related_json = self.related_information.iter()
            .map(|related| related.to_json())
            .collect::<Vec<_>>()
            .join(",");
        
        let additional_data_json = self.additional_data.iter()
            .map(|(key, value)| format!("\"{}\":\"{}\",", key, value.replace("\"", "\\\"")))
            .collect::<Vec<_>>()
            .join("");
        
        format!(
            r#"{{
                "id": "{}",
                "code": "{}",
                "message": "{}",
                "severity": {},
                "category": "{}",
                "location": {{
                    "uri": "{}",
                    "range": {},
                    "lineText": {}
                }},
                "source": "{}",
                "timestamp": {},
                "fixes": [{}],
                "relatedInformation": [{}],
                "additionalData": {{{}}}
            }}"#,
            self.id,
            self.code,
            self.message.replace("\"", "\\\""),
            u8::from(self.severity),
            self.category.name(),
            self.location.uri,
            self.range_to_json(&self.location.range),
            self.location.line_text.as_ref().map_or("null".to_string(), |t| format!("\"{}\"", t.replace("\"", "\\\""))),
            self.source,
            self.timestamp,
            fixes_json,
            related_json,
            additional_data_json
        )
    }
    
    /// Convert range to JSON
    fn range_to_json(&self, range: &Range) -> String {
        format!(
            r#"{{
                "start": {{ "line": {}, "character": {} }},
                "end": {{ "line": {}, "character": {} }}
            }}"#,
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character
        )
    }
}

/// Error statistics
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    /// Total number of errors
    pub total: usize,
    
    /// Number of errors by severity
    pub by_severity: HashMap<ErrorSeverity, usize>,
    
    /// Number of errors by category
    pub by_category: HashMap<ErrorCategory, usize>,
    
    /// Number of errors by document
    pub by_document: HashMap<String, usize>,
    
    /// Number of errors by code
    pub by_code: HashMap<String, usize>,
}

/// Error reporting options
#[derive(Debug, Clone)]
pub struct ErrorReportingOptions {
    /// Whether to include line text
    pub include_line_text: bool,
    
    /// Whether to include fixes
    pub include_fixes: bool,
    
    /// Whether to include related information
    pub include_related_information: bool,
    
    /// Whether to include additional data
    pub include_additional_data: bool,
    
    /// Maximum number of errors to report
    pub max_errors: usize,
    
    /// Minimum severity to report
    pub min_severity: ErrorSeverity,
}

impl Default for ErrorReportingOptions {
    fn default() -> Self {
        ErrorReportingOptions {
            include_line_text: true,
            include_fixes: true,
            include_related_information: true,
            include_additional_data: true,
            max_errors: 100,
            min_severity: ErrorSeverity::Warning,
        }
    }
}

/// Error reporting request
#[derive(Debug, Clone)]
pub struct ErrorReportingRequest {
    /// The document URI
    pub document_uri: String,
    
    /// The document text
    pub text: Option<String>,
    
    /// The error reporting options
    pub options: Option<ErrorReportingOptions>,
    
    /// The checking request
    pub checking_request: Option<CheckingRequest>,
}

/// Error reporting response
#[derive(Debug, Clone)]
pub struct ErrorReportingResponse {
    /// The error reports
    pub reports: Vec<ErrorReport>,
    
    /// The error statistics
    pub statistics: ErrorStatistics,
    
    /// Whether the document is valid
    pub is_valid: bool,
}

/// Error reporting interface
pub struct ErrorReportingInterface {
    /// The document manager
    document_manager: SharedDocumentManager,
    
    /// The checking API
    checking_api: SharedCheckingApi,
}

impl ErrorReportingInterface {
    /// Create a new error reporting interface
    pub fn new(
        document_manager: SharedDocumentManager,
        checking_api: SharedCheckingApi
    ) -> Self {
        ErrorReportingInterface {
            document_manager,
            checking_api,
        }
    }
    
    /// Report errors in a document
    pub fn report_errors(
        &self,
        request: ErrorReportingRequest
    ) -> Result<ErrorReportingResponse, String> {
        // Get the document
        let document = if let Some(text) = &request.text {
            // Create a temporary document with the provided text
            Document::new(request.document_uri.clone(), text.clone())
        } else {
            // Get the document from the document manager
            self.get_document(&request.document_uri)?
        };
        
        // Get the options
        let options = request.options.unwrap_or_default();
        
        // Check the document
        let checking_request = if let Some(req) = request.checking_request {
            req
        } else {
            CheckingRequest {
                document_uri: request.document_uri.clone(),
                text: request.text.clone(),
                options: None,
                ast: None,
                parse_result: None,
            }
        };
        
        let checking_api = self.checking_api.lock().unwrap();
        let checking_response = checking_api.check_document(checking_request)?;
        
        // Convert diagnostics to error reports
        let mut reports = Vec::new();
        
        for diagnostic in &checking_response.diagnostics {
            // Skip diagnostics with severity below the minimum
            if let Some(severity) = diagnostic.severity {
                let error_severity = ErrorSeverity::from(severity);
                if error_severity > options.min_severity {
                    continue;
                }
            }
            
            // Get the line text if requested
            let line_text = if options.include_line_text {
                document.get_line(diagnostic.range.start.line)
            } else {
                None
            };
            
            // Create the error report
            let mut report = ErrorReport::from_diagnostic(diagnostic, &request.document_uri, line_text);
            
            // Add fixes if requested
            if options.include_fixes {
                report.fixes = self.get_fixes_for_diagnostic(&document, diagnostic);
            }
            
            // Add related information if requested
            if options.include_related_information && diagnostic.related_information.is_some() {
                for related in diagnostic.related_information.as_ref().unwrap() {
                    // Get the related document
                    let related_document = self.get_document(&related.location.uri).ok();
                    
                    // Get the line text if requested
                    let related_line_text = if options.include_line_text && related_document.is_some() {
                        related_document.as_ref().unwrap().get_line(related.location.range.start.line)
                    } else {
                        None
                    };
                    
                    // Create a related diagnostic
                    let related_diagnostic = Diagnostic {
                        range: related.location.range,
                        severity: None,
                        code: None,
                        source: None,
                        message: related.message.clone(),
                        related_information: None,
                        tags: None,
                    };
                    
                    // Create the related error report
                    let related_report = ErrorReport::from_diagnostic(
                        &related_diagnostic,
                        &related.location.uri,
                        related_line_text
                    );
                    
                    report.related_information.push(related_report);
                }
            }
            
            // Add additional data if requested
            if options.include_additional_data {
                // Add category-specific data
                match report.category {
                    ErrorCategory::Performance => {
                        report.additional_data.insert("impact".to_string(), "medium".to_string());
                        report.additional_data.insert("suggestion".to_string(), "Consider optimizing this code".to_string());
                    },
                    ErrorCategory::Security => {
                        report.additional_data.insert("risk".to_string(), "medium".to_string());
                        report.additional_data.insert("cwe".to_string(), "CWE-000".to_string());
                    },
                    ErrorCategory::Deprecation => {
                        report.additional_data.insert("since".to_string(), "1.0.0".to_string());
                        report.additional_data.insert("replacement".to_string(), "Use the new API instead".to_string());
                    },
                    _ => {}
                }
            }
            
            reports.push(report);
        }
        
        // Limit the number of reports
        if reports.len() > options.max_errors {
            // Sort by severity (most severe first)
            reports.sort_by(|a, b| a.severity.cmp(&b.severity));
            reports.truncate(options.max_errors);
        }
        
        // Calculate statistics
        let statistics = self.calculate_statistics(&reports);
        
        // Create the response
        let response = ErrorReportingResponse {
            reports,
            statistics,
            is_valid: checking_response.is_valid,
        };
        
        Ok(response)
    }
    
    /// Report errors as JSON
    pub fn report_errors_as_json(
        &self,
        request: ErrorReportingRequest
    ) -> Result<String, String> {
        // Get the error reports
        let response = self.report_errors(request)?;
        
        // Convert to JSON
        let reports_json = response.reports.iter()
            .map(|report| report.to_json())
            .collect::<Vec<_>>()
            .join(",");
        
        // Create the statistics JSON
        let by_severity_json = response.statistics.by_severity.iter()
            .map(|(severity, count)| format!("\"{}\":{}", u8::from(*severity), count))
            .collect::<Vec<_>>()
            .join(",");
        
        let by_category_json = response.statistics.by_category.iter()
            .map(|(category, count)| format!("\"{}\":{}", category.name(), count))
            .collect::<Vec<_>>()
            .join(",");
        
        let by_document_json = response.statistics.by_document.iter()
            .map(|(document, count)| format!("\"{}\":{}", document, count))
            .collect::<Vec<_>>()
            .join(",");
        
        let by_code_json = response.statistics.by_code.iter()
            .map(|(code, count)| format!("\"{}\":{}", code, count))
            .collect::<Vec<_>>()
            .join(",");
        
        // Create the response JSON
        let json = format!(
            r#"{{
                "reports": [{}],
                "statistics": {{
                    "total": {},
                    "bySeverity": {{{}}},
                    "byCategory": {{{}}},
                    "byDocument": {{{}}},
                    "byCode": {{{}}}
                }},
                "isValid": {}
            }}"#,
            reports_json,
            response.statistics.total,
            by_severity_json,
            by_category_json,
            by_document_json,
            by_code_json,
            response.is_valid
        );
        
        Ok(json)
    }
    
    /// Get fixes for a diagnostic
    fn get_fixes_for_diagnostic(
        &self,
        document: &Document,
        diagnostic: &Diagnostic
    ) -> Vec<ErrorFix> {
        // This is a simplified implementation
        // In a real implementation, we would analyze the diagnostic and provide appropriate fixes
        
        let mut fixes = Vec::new();
        
        // Add some generic fixes based on the diagnostic code
        if let Some(code) = &diagnostic.code {
            match code.as_str() {
                "syntax-error" => {
                    // For syntax errors, we can't provide automatic fixes
                },
                "unused-var" | "unused-import" | "unused-function" => {
                    // For unused variables, imports, or functions, we can suggest removing them
                    if let Some(line_text) = document.get_line(diagnostic.range.start.line) {
                        fixes.push(ErrorFix {
                            title: "Remove unused code".to_string(),
                            description: Some("Remove the unused declaration".to_string()),
                            uri: document.uri.clone(),
                            range: diagnostic.range,
                            new_text: String::new(),
                            is_preferred: true,
                        });
                    }
                },
                "missing-semicolon" => {
                    // For missing semicolons, we can suggest adding one
                    fixes.push(ErrorFix {
                        title: "Add semicolon".to_string(),
                        description: Some("Add a semicolon at the end of the line".to_string()),
                        uri: document.uri.clone(),
                        range: Range {
                            start: diagnostic.range.end,
                            end: diagnostic.range.end,
                        },
                        new_text: ";".to_string(),
                        is_preferred: true,
                    });
                },
                "undefined-var" => {
                    // For undefined variables, we can suggest declaring them
                    if let Some(line_text) = document.get_line(diagnostic.range.start.line) {
                        let var_name = line_text[diagnostic.range.start.character as usize..diagnostic.range.end.character as usize].to_string();
                        fixes.push(ErrorFix {
                            title: format!("Declare '{}'", var_name),
                            description: Some(format!("Declare the variable '{}'", var_name)),
                            uri: document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: diagnostic.range.start.line,
                                    character: 0,
                                },
                                end: Position {
                                    line: diagnostic.range.start.line,
                                    character: 0,
                                },
                            },
                            new_text: format!("let {} = undefined;\n", var_name),
                            is_preferred: true,
                        });
                    }
                },
                _ => {
                    // For other errors, we don't provide automatic fixes
                }
            }
        }
        
        fixes
    }
    
    /// Calculate statistics
    fn calculate_statistics(&self, reports: &[ErrorReport]) -> ErrorStatistics {
        let mut by_severity = HashMap::new();
        let mut by_category = HashMap::new();
        let mut by_document = HashMap::new();
        let mut by_code = HashMap::new();
        
        for report in reports {
            // Count by severity
            *by_severity.entry(report.severity).or_insert(0) += 1;
            
            // Count by category
            *by_category.entry(report.category).or_insert(0) += 1;
            
            // Count by document
            *by_document.entry(report.location.uri.clone()).or_insert(0) += 1;
            
            // Count by code
            *by_code.entry(report.code.clone()).or_insert(0) += 1;
        }
        
        ErrorStatistics {
            total: reports.len(),
            by_severity,
            by_category,
            by_document,
            by_code,
        }
    }
    
    /// Get document
    fn get_document(&self, uri: &str) -> Result<Document, String> {
        let document_manager = self.document_manager.lock().unwrap();
        document_manager.get_document(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))
            .map(|doc| doc.clone())
    }
}

/// Shared error reporting interface that can be used across threads
pub type SharedErrorReportingInterface = Arc<Mutex<ErrorReportingInterface>>;

/// Create a new shared error reporting interface
pub fn create_shared_error_reporting_interface(
    document_manager: SharedDocumentManager,
    checking_api: SharedCheckingApi
) -> SharedErrorReportingInterface {
    Arc::new(Mutex::new(ErrorReportingInterface::new(
        document_manager,
        checking_api
    )))
}
