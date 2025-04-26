"""
Static Analyzer for Anarchy Inference

This module provides static code analysis capabilities to identify potential issues
in Anarchy Inference code without executing it.
"""

import os
import json
import re
from typing import List, Dict, Any, Optional, Set, Tuple

class ValidationResult:
    """Represents a validation finding from any validation tool."""
    
    def __init__(self, 
                 tool: str,
                 rule_id: str, 
                 message: str, 
                 file_path: str, 
                 line_number: int, 
                 column: int = 0,
                 severity: str = "warning",
                 code_snippet: str = "",
                 suggestion: str = "",
                 category: str = "general"):
        """
        Initialize a validation result.
        
        Args:
            tool: The validation tool that generated this result
            rule_id: Unique identifier for the rule that triggered this result
            message: Description of the issue
            file_path: Path to the file containing the issue
            line_number: Line number where the issue was found
            column: Column number where the issue was found
            severity: Severity level (info, warning, error, critical)
            code_snippet: The relevant code snippet
            suggestion: Suggested fix for the issue
            category: Category of the issue (e.g., security, performance)
        """
        self.tool = tool
        self.rule_id = rule_id
        self.message = message
        self.file_path = file_path
        self.line_number = line_number
        self.column = column
        self.severity = severity
        self.code_snippet = code_snippet
        self.suggestion = suggestion
        self.category = category
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the result to a dictionary."""
        return {
            "tool": self.tool,
            "rule_id": self.rule_id,
            "message": self.message,
            "file_path": self.file_path,
            "line_number": self.line_number,
            "column": self.column,
            "severity": self.severity,
            "code_snippet": self.code_snippet,
            "suggestion": self.suggestion,
            "category": self.category
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ValidationResult':
        """Create a ValidationResult from a dictionary."""
        return cls(
            tool=data.get("tool", "unknown"),
            rule_id=data.get("rule_id", "unknown"),
            message=data.get("message", ""),
            file_path=data.get("file_path", ""),
            line_number=data.get("line_number", 0),
            column=data.get("column", 0),
            severity=data.get("severity", "warning"),
            code_snippet=data.get("code_snippet", ""),
            suggestion=data.get("suggestion", ""),
            category=data.get("category", "general")
        )
    
    def __str__(self) -> str:
        """String representation of the validation result."""
        return f"[{self.severity.upper()}] {self.file_path}:{self.line_number} - {self.message}"


class Rule:
    """Base class for static analysis rules."""
    
    def __init__(self, rule_id: str, description: str, severity: str = "warning", category: str = "general"):
        """
        Initialize a rule.
        
        Args:
            rule_id: Unique identifier for the rule
            description: Description of what the rule checks for
            severity: Default severity level for violations of this rule
            category: Category of issues this rule detects
        """
        self.rule_id = rule_id
        self.description = description
        self.severity = severity
        self.category = category
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """
        Check the content for violations of this rule.
        
        Args:
            file_path: Path to the file being checked
            content: Content of the file
            
        Returns:
            List of ValidationResult objects representing violations
        """
        raise NotImplementedError("Subclasses must implement this method")


class PatternRule(Rule):
    """Rule that checks for patterns in the code using regular expressions."""
    
    def __init__(self, rule_id: str, description: str, pattern: str, 
                 message_template: str, suggestion_template: str = "",
                 severity: str = "warning", category: str = "general"):
        """
        Initialize a pattern rule.
        
        Args:
            rule_id: Unique identifier for the rule
            description: Description of what the rule checks for
            pattern: Regular expression pattern to search for
            message_template: Template for the message (can use {match} placeholder)
            suggestion_template: Template for the suggestion (can use {match} placeholder)
            severity: Default severity level for violations of this rule
            category: Category of issues this rule detects
        """
        super().__init__(rule_id, description, severity, category)
        self.pattern = re.compile(pattern)
        self.message_template = message_template
        self.suggestion_template = suggestion_template
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Check the content for pattern matches."""
        results = []
        lines = content.split('\n')
        
        for i, line in enumerate(lines):
            for match in self.pattern.finditer(line):
                message = self.message_template.format(match=match.group(0))
                suggestion = self.suggestion_template.format(match=match.group(0)) if self.suggestion_template else ""
                
                # Get some context for the code snippet
                start_line = max(0, i - 2)
                end_line = min(len(lines), i + 3)
                code_snippet = '\n'.join(lines[start_line:end_line])
                
                results.append(ValidationResult(
                    tool="StaticAnalyzer",
                    rule_id=self.rule_id,
                    message=message,
                    file_path=file_path,
                    line_number=i + 1,
                    column=match.start() + 1,
                    severity=self.severity,
                    code_snippet=code_snippet,
                    suggestion=suggestion,
                    category=self.category
                ))
        
        return results


class ASTRule(Rule):
    """Rule that analyzes the Abstract Syntax Tree of the code."""
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Check the AST for violations."""
        # This would use the Anarchy Inference parser to get the AST
        # and then analyze it for issues
        # For now, we'll return an empty list as a placeholder
        return []


class StaticAnalyzer:
    """Static code analyzer for Anarchy Inference."""
    
    def __init__(self):
        """Initialize the static analyzer with default rules."""
        self.rules = []
        self._initialize_default_rules()
    
    def _initialize_default_rules(self):
        """Initialize the default set of rules."""
        # Syntax rules
        self.rules.append(PatternRule(
            rule_id="SYNTAX001",
            description="Missing semicolon at end of statement",
            pattern=r"[^;{}\s]\s*$",
            message_template="Possible missing semicolon at the end of the line",
            suggestion_template="Add a semicolon at the end of the line",
            severity="error",
            category="syntax"
        ))
        
        # Naming rules
        self.rules.append(PatternRule(
            rule_id="NAME001",
            description="Variable names should be descriptive",
            pattern=r"\bι\s+([a-z]|[A-Z])\b",
            message_template="Single-letter variable name detected: {match}",
            suggestion_template="Use a more descriptive variable name",
            severity="warning",
            category="naming"
        ))
        
        # Performance rules
        self.rules.append(PatternRule(
            rule_id="PERF001",
            description="Inefficient string concatenation in loop",
            pattern=r"for.*\{.*\+\+.*\}",
            message_template="Possible inefficient string concatenation in a loop",
            suggestion_template="Consider using a string builder or buffer for concatenation in loops",
            severity="warning",
            category="performance"
        ))
        
        # Security rules
        self.rules.append(PatternRule(
            rule_id="SEC001",
            description="Potential SQL injection",
            pattern=r"⇓\s*\(\s*.*\+.*\)",
            message_template="Potential SQL injection: concatenating strings in a query",
            suggestion_template="Use parameterized queries instead of string concatenation",
            severity="critical",
            category="security"
        ))
        
        # Best practice rules
        self.rules.append(PatternRule(
            rule_id="BP001",
            description="Missing error handling",
            pattern=r"⇓\s*\([^÷]*\)",
            message_template="Network operation without error handling",
            suggestion_template="Add error handling with the ÷ operator",
            severity="warning",
            category="best_practice"
        ))
    
    def add_rule(self, rule: Rule):
        """Add a custom rule to the analyzer."""
        self.rules.append(rule)
    
    def analyze_file(self, file_path: str) -> List[ValidationResult]:
        """
        Analyze a single file.
        
        Args:
            file_path: Path to the file to analyze
            
        Returns:
            List of ValidationResult objects
        """
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            results = []
            for rule in self.rules:
                results.extend(rule.check(file_path, content))
            
            return results
        except Exception as e:
            # Return a validation result for the error
            return [ValidationResult(
                tool="StaticAnalyzer",
                rule_id="INTERNAL001",
                message=f"Error analyzing file: {str(e)}",
                file_path=file_path,
                line_number=0,
                severity="error",
                category="internal"
            )]
    
    def analyze_directory(self, directory_path: str, file_extensions: List[str] = ['.a.i', '.ai']) -> List[ValidationResult]:
        """
        Analyze all files in a directory with the specified extensions.
        
        Args:
            directory_path: Path to the directory to analyze
            file_extensions: List of file extensions to analyze
            
        Returns:
            List of ValidationResult objects
        """
        results = []
        
        for root, _, files in os.walk(directory_path):
            for file in files:
                if any(file.endswith(ext) for ext in file_extensions):
                    file_path = os.path.join(root, file)
                    results.extend(self.analyze_file(file_path))
        
        return results
    
    def generate_report(self, results: List[ValidationResult], output_format: str = "text") -> str:
        """
        Generate a report from the analysis results.
        
        Args:
            results: List of ValidationResult objects
            output_format: Format of the report (text, json, html)
            
        Returns:
            Report as a string
        """
        if output_format == "json":
            return json.dumps([r.to_dict() for r in results], indent=2)
        
        elif output_format == "html":
            html = """
            <!DOCTYPE html>
            <html>
            <head>
                <title>Static Analysis Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    .result { margin-bottom: 20px; padding: 10px; border: 1px solid #ddd; }
                    .error { border-left: 5px solid #f44336; }
                    .warning { border-left: 5px solid #ff9800; }
                    .info { border-left: 5px solid #2196F3; }
                    .critical { border-left: 5px solid #9c27b0; }
                    .code { background-color: #f5f5f5; padding: 10px; font-family: monospace; white-space: pre; }
                    .suggestion { background-color: #e8f5e9; padding: 10px; margin-top: 10px; }
                </style>
            </head>
            <body>
                <h1>Static Analysis Report</h1>
                <p>Total issues found: {}</p>
            """.format(len(results))
            
            for result in results:
                html += """
                <div class="result {}">
                    <h3>{} - {}</h3>
                    <p><strong>File:</strong> {}:{}</p>
                    <p><strong>Rule:</strong> {}</p>
                    <p><strong>Category:</strong> {}</p>
                    <div class="code">{}</div>
                """.format(
                    result.severity,
                    result.severity.upper(),
                    result.message,
                    result.file_path,
                    result.line_number,
                    result.rule_id,
                    result.category,
                    result.code_snippet.replace("<", "&lt;").replace(">", "&gt;")
                )
                
                if result.suggestion:
                    html += """
                    <div class="suggestion">
                        <strong>Suggestion:</strong> {}
                    </div>
                    """.format(result.suggestion)
                
                html += "</div>"
            
            html += """
            </body>
            </html>
            """
            
            return html
        
        else:  # text format
            report = "Static Analysis Report\n"
            report += "=====================\n\n"
            report += f"Total issues found: {len(results)}\n\n"
            
            for result in results:
                report += f"[{result.severity.upper()}] {result.file_path}:{result.line_number} - {result.message}\n"
                report += f"Rule: {result.rule_id} ({result.category})\n"
                report += f"Code:\n{result.code_snippet}\n"
                if result.suggestion:
                    report += f"Suggestion: {result.suggestion}\n"
                report += "\n"
            
            return report


# Example usage
if __name__ == "__main__":
    analyzer = StaticAnalyzer()
    results = analyzer.analyze_directory("/path/to/code")
    report = analyzer.generate_report(results, "text")
    print(report)
