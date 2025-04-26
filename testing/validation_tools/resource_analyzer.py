"""
Resource Usage Analyzer for Anarchy Inference

This module provides resource usage analysis capabilities to identify performance issues
and resource consumption patterns in Anarchy Inference code.
"""

import os
import json
import re
import time
import tracemalloc
from typing import List, Dict, Any, Optional, Set, Tuple, Callable

from .static_analyzer import ValidationResult

class ResourceMetrics:
    """Class for storing resource usage metrics."""
    
    def __init__(self):
        """Initialize resource metrics."""
        self.memory_usage = 0  # in bytes
        self.token_usage = 0  # number of tokens
        self.execution_time = 0  # in seconds
        self.io_operations = 0  # number of I/O operations
        self.network_requests = 0  # number of network requests
        self.cpu_time = 0  # in seconds
        
    def to_dict(self) -> Dict[str, Any]:
        """Convert metrics to a dictionary."""
        return {
            "memory_usage": self.memory_usage,
            "token_usage": self.token_usage,
            "execution_time": self.execution_time,
            "io_operations": self.io_operations,
            "network_requests": self.network_requests,
            "cpu_time": self.cpu_time
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ResourceMetrics':
        """Create ResourceMetrics from a dictionary."""
        metrics = cls()
        metrics.memory_usage = data.get("memory_usage", 0)
        metrics.token_usage = data.get("token_usage", 0)
        metrics.execution_time = data.get("execution_time", 0)
        metrics.io_operations = data.get("io_operations", 0)
        metrics.network_requests = data.get("network_requests", 0)
        metrics.cpu_time = data.get("cpu_time", 0)
        return metrics


class ResourceRule:
    """Base class for resource usage analysis rules."""
    
    def __init__(self, rule_id: str, description: str, severity: str = "warning", category: str = "resource"):
        """
        Initialize a resource rule.
        
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
        Check the content for resource usage issues.
        
        Args:
            file_path: Path to the file being checked
            content: Content of the file
            
        Returns:
            List of ValidationResult objects representing issues
        """
        raise NotImplementedError("Subclasses must implement this method")


class PatternResourceRule(ResourceRule):
    """Resource rule that checks for resource usage patterns using regular expressions."""
    
    def __init__(self, rule_id: str, description: str, pattern: str, 
                 message_template: str, suggestion_template: str = "",
                 severity: str = "warning", category: str = "resource"):
        """
        Initialize a pattern resource rule.
        
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
        """Check the content for resource usage patterns."""
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
                    tool="ResourceAnalyzer",
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


class ComplexityAnalysisRule(ResourceRule):
    """Resource rule that analyzes code complexity."""
    
    def __init__(self, rule_id: str = "RES005", description: str = "Analyzes code complexity",
                 severity: str = "warning", category: str = "complexity"):
        """Initialize the complexity analysis rule."""
        super().__init__(rule_id, description, severity, category)
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Analyze code complexity."""
        results = []
        lines = content.split('\n')
        
        # Count nested loops and conditionals
        loop_stack = []
        conditional_stack = []
        
        for i, line in enumerate(lines):
            # Check for loop start
            if re.search(r'\bfor\b|\bwhile\b', line):
                loop_stack.append(i)
            
            # Check for loop end
            if re.search(r'\}', line) and loop_stack:
                loop_start = loop_stack.pop()
                
                # If this is a deeply nested loop (depth > 2), report it
                if len(loop_stack) >= 2:
                    # Get some context for the code snippet
                    start_line = max(0, loop_start - 1)
                    end_line = min(len(lines), i + 2)
                    code_snippet = '\n'.join(lines[start_line:end_line])
                    
                    results.append(ValidationResult(
                        tool="ResourceAnalyzer",
                        rule_id=self.rule_id,
                        message=f"Deeply nested loop detected (depth {len(loop_stack) + 1})",
                        file_path=file_path,
                        line_number=loop_start + 1,
                        severity=self.severity,
                        code_snippet=code_snippet,
                        suggestion="Consider refactoring to reduce nesting depth",
                        category=self.category
                    ))
            
            # Check for conditional start
            if re.search(r'\bif\b|\bswitch\b|\bcase\b', line):
                conditional_stack.append(i)
            
            # Check for conditional end
            if re.search(r'\}', line) and conditional_stack:
                conditional_start = conditional_stack.pop()
                
                # If this is a deeply nested conditional (depth > 3), report it
                if len(conditional_stack) >= 3:
                    # Get some context for the code snippet
                    start_line = max(0, conditional_start - 1)
                    end_line = min(len(lines), i + 2)
                    code_snippet = '\n'.join(lines[start_line:end_line])
                    
                    results.append(ValidationResult(
                        tool="ResourceAnalyzer",
                        rule_id=self.rule_id,
                        message=f"Deeply nested conditional detected (depth {len(conditional_stack) + 1})",
                        file_path=file_path,
                        line_number=conditional_start + 1,
                        severity=self.severity,
                        code_snippet=code_snippet,
                        suggestion="Consider refactoring to reduce nesting depth",
                        category=self.category
                    ))
            
            # Check for long functions
            function_match = re.search(r'\bλ\s+(\w+)\s*\(', line)
            if function_match:
                # Find the end of the function
                function_start = i
                brace_count = 0
                function_end = i
                
                for j in range(i, len(lines)):
                    brace_count += lines[j].count('{') - lines[j].count('}')
                    if brace_count <= 0:
                        function_end = j
                        break
                
                function_length = function_end - function_start + 1
                
                # If the function is too long (> 50 lines), report it
                if function_length > 50:
                    results.append(ValidationResult(
                        tool="ResourceAnalyzer",
                        rule_id=self.rule_id,
                        message=f"Long function detected ({function_length} lines): {function_match.group(1)}",
                        file_path=file_path,
                        line_number=function_start + 1,
                        severity=self.severity,
                        code_snippet=f"Function {function_match.group(1)} from line {function_start + 1} to {function_end + 1}",
                        suggestion="Consider breaking this function into smaller, more focused functions",
                        category=self.category
                    ))
        
        return results


class TokenUsageAnalysisRule(ResourceRule):
    """Resource rule that analyzes token usage."""
    
    def __init__(self, rule_id: str = "RES001", description: str = "Analyzes token usage efficiency",
                 severity: str = "warning", category: str = "token_usage"):
        """Initialize the token usage analysis rule."""
        super().__init__(rule_id, description, severity, category)
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Analyze token usage efficiency."""
        results = []
        lines = content.split('\n')
        
        # Simple token counting - in a real implementation, this would use
        # the actual tokenizer from the Anarchy Inference interpreter
        total_tokens = 0
        token_heavy_lines = []
        
        for i, line in enumerate(lines):
            # Simplified token counting - split by whitespace and punctuation
            tokens = re.findall(r'\b\w+\b|[^\w\s]', line)
            line_tokens = len(tokens)
            total_tokens += line_tokens
            
            # If this line has a lot of tokens, record it
            if line_tokens > 20:  # Arbitrary threshold
                token_heavy_lines.append((i, line_tokens))
        
        # Report overall token usage if it's high
        if total_tokens > 1000:  # Arbitrary threshold
            results.append(ValidationResult(
                tool="ResourceAnalyzer",
                rule_id=self.rule_id,
                message=f"High overall token usage: {total_tokens} tokens",
                file_path=file_path,
                line_number=1,
                severity=self.severity,
                code_snippet="",
                suggestion="Consider refactoring to reduce token usage",
                category=self.category
            ))
        
        # Report token-heavy lines
        for i, line_tokens in token_heavy_lines:
            # Get some context for the code snippet
            start_line = max(0, i - 2)
            end_line = min(len(lines), i + 3)
            code_snippet = '\n'.join(lines[start_line:end_line])
            
            results.append(ValidationResult(
                tool="ResourceAnalyzer",
                rule_id=self.rule_id,
                message=f"Token-heavy line: {line_tokens} tokens",
                file_path=file_path,
                line_number=i + 1,
                severity=self.severity,
                code_snippet=code_snippet,
                suggestion="Consider breaking this line into multiple lines or simplifying expressions",
                category=self.category
            ))
        
        return results


class MemoryLeakDetectionRule(ResourceRule):
    """Resource rule that detects potential memory leaks."""
    
    def __init__(self, rule_id: str = "RES002", description: str = "Detects potential memory leaks",
                 severity: str = "warning", category: str = "memory_usage"):
        """Initialize the memory leak detection rule."""
        super().__init__(rule_id, description, severity, category)
    
    def check(self, file_path: str, content: str) -> List[ValidationResult]:
        """Detect potential memory leaks."""
        results = []
        lines = content.split('\n')
        
        # Track resource allocations and deallocations
        allocations = {}  # Maps variable names to line numbers where they were allocated
        
        for i, line in enumerate(lines):
            # Check for resource allocations
            alloc_match = re.search(r'(\w+)\s*=\s*new\s+(\w+)', line)
            if alloc_match:
                var_name = alloc_match.group(1)
                resource_type = alloc_match.group(2)
                allocations[var_name] = (i, resource_type)
            
            # Check for resource deallocations
            dealloc_match = re.search(r'free\s*\(\s*(\w+)\s*\)', line)
            if dealloc_match:
                var_name = dealloc_match.group(1)
                if var_name in allocations:
                    del allocations[var_name]
        
        # Report any resources that were allocated but not freed
        for var_name, (line_num, resource_type) in allocations.items():
            # Get some context for the code snippet
            start_line = max(0, line_num - 2)
            end_line = min(len(lines), line_num + 3)
            code_snippet = '\n'.join(lines[start_line:end_line])
            
            results.append(ValidationResult(
                tool="ResourceAnalyzer",
                rule_id=self.rule_id,
                message=f"Potential memory leak: {resource_type} '{var_name}' allocated but not freed",
                file_path=file_path,
                line_number=line_num + 1,
                severity=self.severity,
                code_snippet=code_snippet,
                suggestion=f"Ensure '{var_name}' is properly freed after use",
                category=self.category
            ))
        
        return results


class ResourceUsageAnalyzer:
    """Resource usage analyzer for Anarchy Inference."""
    
    def __init__(self):
        """Initialize the resource usage analyzer with default rules."""
        self.rules = []
        self._initialize_default_rules()
    
    def _initialize_default_rules(self):
        """Initialize the default set of resource rules."""
        # Token usage analysis
        self.rules.append(TokenUsageAnalysisRule())
        
        # Memory leak detection
        self.rules.append(MemoryLeakDetectionRule())
        
        # Inefficient string operations
        self.rules.append(PatternResourceRule(
            rule_id="RES003",
            description="Inefficient string concatenation in loop",
            pattern=r'for.*\{.*\+\+.*\}',
            message_template="Potential inefficient string concatenation in a loop",
            suggestion_template="Use a string builder or buffer for concatenation in loops",
            severity="warning",
            category="performance"
        ))
        
        # Unclosed resources
        self.rules.append(PatternResourceRule(
            rule_id="RES004",
            description="Unclosed resources",
            pattern=r'open\s*\(',
            message_template="Potential unclosed resource: file opened but not explicitly closed",
            suggestion_template="Ensure all opened resources are properly closed",
            severity="warning",
            category="resource_leak"
        ))
        
        # Code complexity analysis
        self.rules.append(ComplexityAnalysisRule())
        
        # Inefficient data structures
        self.rules.append(PatternResourceRule(
            rule_id="RES006",
            description="Inefficient data structure usage",
            pattern=r'array\s+\w+\s*=',
            message_template="Consider using more efficient data structures than arrays for large datasets",
            suggestion_template="Use hash maps or sets for faster lookups when appropriate",
            severity="info",
            category="performance"
        ))
        
        # Excessive network requests
        self.rules.append(PatternResourceRule(
            rule_id="RES007",
            description="Excessive network requests",
            pattern=r'⇓\s*\(',
            message_template="Potential excessive network requests",
            suggestion_template="Consider batching network requests or caching results",
            severity="warning",
            category="network"
        ))
    
    def add_rule(self, rule: ResourceRule):
        """Add a custom rule to the analyzer."""
        self.rules.append(rule)
    
    def analyze_file(self, file_path: str) -> List[ValidationResult]:
        """
        Analyze a single file for resource usage issues.
        
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
                tool="ResourceAnalyzer",
                rule_id="INTERNAL001",
                message=f"Error analyzing file: {str(e)}",
                file_path=file_path,
                line_number=0,
                severity="error",
                category="internal"
            )]
    
    def analyze_directory(self, directory_path: str, file_extensions: List[str] = ['.a.i', '.ai']) -> List[ValidationResult]:
        """
        Analyze all files in a directory for resource usage issues.
        
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
    
    def profile_execution(self, func: Callable, *args, **kwargs) -> Tuple[Any, ResourceMetrics]:
        """
        Profile the execution of a function.
        
        Args:
            func: Function to profile
            *args: Arguments to pass to the function
            **kwargs: Keyword arguments to pass to the function
            
        Returns:
            Tuple of (function result, ResourceMetrics)
        """
        # Start tracking memory
        tracemalloc.start()
        
        # Record start time
        start_time = time.time()
        
        # Execute the function
        result = func(*args, **kwargs)
        
        # Record end time
        end_time = time.time()
        
        # Get memory usage
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        # Create metrics
        metrics = ResourceMetrics()
        metrics.memory_usage = peak
        metrics.execution_time = end_time - start_time
        
        # In a real implementation, we would also track:
        # - Token usage from the interpreter
        # - I/O operations by monitoring file and network activity
        # - CPU time using more precise measurements
        
        return result, metrics
    
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
                <title>Resource Usage Analysis Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    .result { margin-bottom: 20px; padding: 10px; border: 1px solid #ddd; }
                    .error { border-left: 5px solid #f44336; }
                    .warning { border-left: 5px solid #ff9800; }
                    .info { border-left: 5px solid #2196F3; }
                    .code { background-color: #f5f5f5; padding: 10px; font-family: monospace; white-space: pre; }
                    .suggestion { background-color: #e8f5e9; padding: 10px; margin-top: 10px; }
                </style>
            </head>
            <body>
                <h1>Resource Usage Analysis Report</h1>
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
            report = "Resource Usage Analysis Report\n"
            report += "==============================\n\n"
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
    analyzer = ResourceUsageAnalyzer()
    results = analyzer.analyze_directory("/path/to/code")
    report = analyzer.generate_report(results, "text")
    print(report)
