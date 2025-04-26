"""
Security Scanner for Anarchy Inference

This module provides security scanning capabilities to identify security vulnerabilities
in Anarchy Inference code.
"""

import os
import json
import re
import hashlib
from typing import List, Dict, Any, Optional, Set, Tuple

from .static_analyzer import ValidationResult

class SecurityRule:
    """Base class for security scanning rules."""
    
    def __init__(self, rule_id: str, description: str, severity: str = "critical", category: str = "security"):
        """
        Initialize a security rule.
        
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
        Check the content for security vulnerabilities.
        
        Args:
            file_path: Path to the file being checked
            content: Content of the file
            
        Returns:
            List of ValidationResult objects representing vulnerabilities
        """
        raise NotImplementedError("Subclasses must implement this method")


class PatternSecurityRule(SecurityRule):
    """Security rule that checks for vulnerable patterns using regular expressions."""
    
    def __init__(self, rule_id: str, description: str, pattern: str, 
                 message_template: str, suggestion_template: str = "",
                 severity: str = "critical", category: str = "security"):
        """
        Initialize a pattern security rule.
        
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
        """Check the content for vulnerable patterns."""
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
                    tool="SecurityScanner",
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


class TaintAnalysisRule(SecurityRule):
    """Security rule that performs taint analysis to track untrusted data."""
    
    def __init__(self, rule_id: str, description: str, 
                 source_patterns: List[str], sink_patterns: List[str],
                 sanitizer_patterns: List[str] = None,
                 severity: str = "critical", category: str = "taint_analysis"):
        """
        Initialize a taint analysis rule.
        
        Args:
            rule_id: Unique identifier for the rule
            description: Description of what the rule checks for
            source_patterns: Patterns that identify sources of untrusted data
            sink_patterns: Patterns that identify sinks where data is used
            sanitizer_patterns: Patterns that identify sanitization functions
            severity: Default severity level for violations of this rule
            category: Category of issues this rule detects
        """
        super().__init__(rule_id, description, severity, category)
        self.source_patterns = [re.compile(p) for p in source_patterns]
        self.sink_patterns = [re.compile(p) for p in sink_patterns]
        self.sanitizer_patterns = [re.compile(p) for p in (sanitizer_patterns or [])]
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Perform taint analysis on the content."""
        results = []
        lines = content.split('\n')
        
        # Simple taint analysis: identify variables that come from sources
        tainted_vars = set()
        
        # First pass: identify sources and track tainted variables
        for i, line in enumerate(lines):
            # Check for sources
            for pattern in self.source_patterns:
                for match in pattern.finditer(line):
                    # This is a simplification - in a real implementation,
                    # we would parse the code and track variable assignments
                    # For now, we'll just assume any variable on the left side of an assignment
                    # with a source on the right is tainted
                    assignment_match = re.search(r'(\w+)\s*=', line)
                    if assignment_match:
                        tainted_vars.add(assignment_match.group(1))
            
            # Check for sanitizers
            for pattern in self.sanitizer_patterns:
                for match in pattern.finditer(line):
                    # This is a simplification - in a real implementation,
                    # we would track which variables are sanitized
                    # For now, we'll just assume any variable on the left side of an assignment
                    # with a sanitizer on the right is no longer tainted
                    assignment_match = re.search(r'(\w+)\s*=', line)
                    if assignment_match and assignment_match.group(1) in tainted_vars:
                        tainted_vars.remove(assignment_match.group(1))
        
        # Second pass: check for tainted variables used in sinks
        for i, line in enumerate(lines):
            for pattern in self.sink_patterns:
                for match in pattern.finditer(line):
                    # Check if any tainted variable is used in this sink
                    for var in tainted_vars:
                        if re.search(r'\b' + re.escape(var) + r'\b', line):
                            # Get some context for the code snippet
                            start_line = max(0, i - 2)
                            end_line = min(len(lines), i + 3)
                            code_snippet = '\n'.join(lines[start_line:end_line])
                            
                            results.append(ValidationResult(
                                tool="SecurityScanner",
                                rule_id=self.rule_id,
                                message=f"Potential tainted data flow: variable '{var}' is used in a sink without proper sanitization",
                                file_path=file_path,
                                line_number=i + 1,
                                column=match.start() + 1,
                                severity=self.severity,
                                code_snippet=code_snippet,
                                suggestion="Sanitize the data before using it in this context",
                                category=self.category
                            ))
        
        return results


class SecretDetectionRule(SecurityRule):
    """Security rule that detects hardcoded secrets and credentials."""
    
    def __init__(self, rule_id: str = "SEC003", description: str = "Detects hardcoded secrets and credentials",
                 severity: str = "critical", category: str = "secret_detection"):
        """Initialize the secret detection rule."""
        super().__init__(rule_id, description, severity, category)
        
        # Patterns for different types of secrets
        self.patterns = {
            "API Key": [
                r'api[_-]?key[^a-zA-Z0-9]([a-zA-Z0-9]{16,64})',
                r'api[_-]?secret[^a-zA-Z0-9]([a-zA-Z0-9]{16,64})',
                r'access[_-]?key[^a-zA-Z0-9]([a-zA-Z0-9]{16,64})',
                r'access[_-]?secret[^a-zA-Z0-9]([a-zA-Z0-9]{16,64})'
            ],
            "Password": [
                r'password[^a-zA-Z0-9]([a-zA-Z0-9!@#$%^&*()_+]{8,32})',
                r'passwd[^a-zA-Z0-9]([a-zA-Z0-9!@#$%^&*()_+]{8,32})',
                r'pwd[^a-zA-Z0-9]([a-zA-Z0-9!@#$%^&*()_+]{8,32})'
            ],
            "Token": [
                r'token[^a-zA-Z0-9]([a-zA-Z0-9_\-.]{16,64})',
                r'auth[_-]?token[^a-zA-Z0-9]([a-zA-Z0-9_\-.]{16,64})'
            ],
            "Private Key": [
                r'-----BEGIN PRIVATE KEY-----',
                r'-----BEGIN RSA PRIVATE KEY-----',
                r'-----BEGIN DSA PRIVATE KEY-----',
                r'-----BEGIN EC PRIVATE KEY-----'
            ]
        }
        
        # Compile all patterns
        self.compiled_patterns = {}
        for secret_type, patterns in self.patterns.items():
            self.compiled_patterns[secret_type] = [re.compile(p, re.IGNORECASE) for p in patterns]
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Check the content for hardcoded secrets."""
        results = []
        lines = content.split('\n')
        
        for i, line in enumerate(lines):
            for secret_type, patterns in self.compiled_patterns.items():
                for pattern in patterns:
                    for match in pattern.finditer(line):
                        # Get some context for the code snippet
                        start_line = max(0, i - 2)
                        end_line = min(len(lines), i + 3)
                        
                        # Redact the actual secret in the code snippet
                        redacted_lines = lines[start_line:end_line].copy()
                        if i >= start_line and i < end_line:
                            # Replace the secret with asterisks
                            secret_start = match.start(1) if match.groups() else match.start()
                            secret_end = match.end(1) if match.groups() else match.end()
                            secret_length = secret_end - secret_start
                            
                            redacted_line = redacted_lines[i - start_line]
                            redacted_lines[i - start_line] = (
                                redacted_line[:secret_start] + 
                                '*' * secret_length + 
                                redacted_line[secret_end:]
                            )
                        
                        code_snippet = '\n'.join(redacted_lines)
                        
                        results.append(ValidationResult(
                            tool="SecurityScanner",
                            rule_id=self.rule_id,
                            message=f"Potential {secret_type} found in code",
                            file_path=file_path,
                            line_number=i + 1,
                            column=match.start() + 1,
                            severity=self.severity,
                            code_snippet=code_snippet,
                            suggestion="Store secrets in environment variables or a secure vault, not in code",
                            category=self.category
                        ))
        
        return results


class SecurityScanner:
    """Security scanner for Anarchy Inference."""
    
    def __init__(self):
        """Initialize the security scanner with default rules."""
        self.rules = []
        self._initialize_default_rules()
    
    def _initialize_default_rules(self):
        """Initialize the default set of security rules."""
        # SQL Injection rules
        self.rules.append(PatternSecurityRule(
            rule_id="SEC001",
            description="SQL Injection vulnerability",
            pattern=r'⇓\s*\(\s*[\'"].*\+.*[\'"]\s*\)',
            message_template="Potential SQL Injection: string concatenation in query",
            suggestion_template="Use parameterized queries instead of string concatenation",
            severity="critical",
            category="sql_injection"
        ))
        
        # XSS rules
        self.rules.append(PatternSecurityRule(
            rule_id="SEC002",
            description="Cross-Site Scripting (XSS) vulnerability",
            pattern=r'✎\s*\(\s*.*\+.*\)',
            message_template="Potential XSS: unescaped output to UI",
            suggestion_template="Use proper HTML escaping before outputting user data",
            severity="critical",
            category="xss"
        ))
        
        # Secret detection
        self.rules.append(SecretDetectionRule())
        
        # Taint analysis for command injection
        self.rules.append(TaintAnalysisRule(
            rule_id="SEC004",
            description="Command Injection vulnerability",
            source_patterns=[r'🎤\s*\(', r'⇓\s*\('],  # User input and network input
            sink_patterns=[r'exec\s*\(', r'system\s*\(', r'shell\s*\('],  # Command execution
            sanitizer_patterns=[r'sanitize_command\s*\(', r'escape_shell\s*\('],
            severity="critical",
            category="command_injection"
        ))
        
        # Insecure cryptography
        self.rules.append(PatternSecurityRule(
            rule_id="SEC005",
            description="Insecure cryptography",
            pattern=r'(md5|sha1)\s*\(',
            message_template="Use of weak cryptographic algorithm: {match}",
            suggestion_template="Use stronger algorithms like SHA-256 or SHA-3",
            severity="high",
            category="cryptography"
        ))
        
        # Insecure random number generation
        self.rules.append(PatternSecurityRule(
            rule_id="SEC006",
            description="Insecure random number generation",
            pattern=r'random\s*\(',
            message_template="Potential use of insecure random number generator",
            suggestion_template="Use cryptographically secure random number generation",
            severity="medium",
            category="cryptography"
        ))
        
        # Path traversal
        self.rules.append(PatternSecurityRule(
            rule_id="SEC007",
            description="Path Traversal vulnerability",
            pattern=r'open\s*\(\s*.*\+.*\)',
            message_template="Potential path traversal: concatenating strings in file path",
            suggestion_template="Validate and sanitize file paths before use",
            severity="high",
            category="path_traversal"
        ))
        
        # Insecure deserialization
        self.rules.append(PatternSecurityRule(
            rule_id="SEC008",
            description="Insecure Deserialization vulnerability",
            pattern=r'deserialize\s*\(',
            message_template="Potential insecure deserialization",
            suggestion_template="Validate and sanitize data before deserialization",
            severity="high",
            category="deserialization"
        ))
    
    def add_rule(self, rule: SecurityRule):
        """Add a custom rule to the scanner."""
        self.rules.append(rule)
    
    def scan_file(self, file_path: str) -> List[ValidationResult]:
        """
        Scan a single file for security vulnerabilities.
        
        Args:
            file_path: Path to the file to scan
            
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
                tool="SecurityScanner",
                rule_id="INTERNAL001",
                message=f"Error scanning file: {str(e)}",
                file_path=file_path,
                line_number=0,
                severity="error",
                category="internal"
            )]
    
    def scan_directory(self, directory_path: str, file_extensions: List[str] = ['.a.i', '.ai']) -> List[ValidationResult]:
        """
        Scan all files in a directory for security vulnerabilities.
        
        Args:
            directory_path: Path to the directory to scan
            file_extensions: List of file extensions to scan
            
        Returns:
            List of ValidationResult objects
        """
        results = []
        
        for root, _, files in os.walk(directory_path):
            for file in files:
                if any(file.endswith(ext) for ext in file_extensions):
                    file_path = os.path.join(root, file)
                    results.extend(self.scan_file(file_path))
        
        return results
    
    def generate_report(self, results: List[ValidationResult], output_format: str = "text") -> str:
        """
        Generate a report from the scan results.
        
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
                <title>Security Scan Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    .result { margin-bottom: 20px; padding: 10px; border: 1px solid #ddd; }
                    .critical { border-left: 5px solid #9c27b0; }
                    .high { border-left: 5px solid #f44336; }
                    .medium { border-left: 5px solid #ff9800; }
                    .low { border-left: 5px solid #2196F3; }
                    .code { background-color: #f5f5f5; padding: 10px; font-family: monospace; white-space: pre; }
                    .suggestion { background-color: #e8f5e9; padding: 10px; margin-top: 10px; }
                </style>
            </head>
            <body>
                <h1>Security Scan Report</h1>
                <p>Total vulnerabilities found: {}</p>
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
            report = "Security Scan Report\n"
            report += "====================\n\n"
            report += f"Total vulnerabilities found: {len(results)}\n\n"
            
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
    scanner = SecurityScanner()
    results = scanner.scan_directory("/path/to/code")
    report = scanner.generate_report(results, "text")
    print(report)
