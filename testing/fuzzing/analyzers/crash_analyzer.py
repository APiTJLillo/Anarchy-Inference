#!/usr/bin/env python3
"""
Crash Analyzer for Anarchy Inference Fuzzing.

This module provides functionality for analyzing crashes to determine their cause and severity
for fuzzing the Anarchy Inference language implementation.
"""

import os
import sys
import time
import re
import logging
import hashlib
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass

# Add parent directory to path to import fuzzing framework
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import from fuzzing framework
from fuzzing.fuzzing_framework import TestResult

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("crash_analyzer")

class CrashAnalyzer:
    """Analyzes crashes to determine their cause and severity."""
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the crash analyzer.
        
        Args:
            config: Optional configuration for the analyzer
        """
        self.config = config or {}
        
        # Default configuration values
        self.min_similarity_threshold = self.config.get("min_similarity_threshold", 0.8)
        self.max_crashes_to_track = self.config.get("max_crashes_to_track", 100)
        
        # Crash tracking
        self.crashes: List[Dict[str, Any]] = []
        self.crash_signatures: Set[str] = set()
        
        # Common error patterns
        self.error_patterns = [
            # Syntax errors
            (r"syntax error", "SYNTAX_ERROR"),
            (r"unexpected token", "SYNTAX_ERROR"),
            (r"expected .* but found", "SYNTAX_ERROR"),
            (r"unterminated string", "SYNTAX_ERROR"),
            (r"unterminated comment", "SYNTAX_ERROR"),
            
            # Type errors
            (r"type error", "TYPE_ERROR"),
            (r"expected .* but got", "TYPE_ERROR"),
            (r"cannot convert .* to", "TYPE_ERROR"),
            (r"incompatible types", "TYPE_ERROR"),
            
            # Reference errors
            (r"undefined variable", "REFERENCE_ERROR"),
            (r"undefined function", "REFERENCE_ERROR"),
            (r"undefined symbol", "REFERENCE_ERROR"),
            
            # Runtime errors
            (r"division by zero", "RUNTIME_ERROR"),
            (r"index out of bounds", "RUNTIME_ERROR"),
            (r"stack overflow", "RUNTIME_ERROR"),
            (r"memory limit exceeded", "RUNTIME_ERROR"),
            
            # Assertion failures
            (r"assertion failed", "ASSERTION_FAILURE"),
            (r"internal error", "ASSERTION_FAILURE"),
            (r"panic", "ASSERTION_FAILURE"),
            
            # Segmentation faults
            (r"segmentation fault", "SEGMENTATION_FAULT"),
            (r"access violation", "SEGMENTATION_FAULT"),
            (r"memory access error", "SEGMENTATION_FAULT"),
            
            # Other crashes
            (r"abort", "ABORT"),
            (r"terminated", "TERMINATED"),
            (r"killed", "KILLED")
        ]
    
    def analyze(self, result: TestResult) -> Dict[str, Any]:
        """Analyze a crash.
        
        Args:
            result: Test result with a crash
            
        Returns:
            Analysis results
        """
        # Check if it's actually a crash
        if not result.crash:
            return {"is_crash": False}
        
        # Extract crash information
        crash_info = self._extract_crash_info(result)
        
        # Generate a crash signature
        signature = self._generate_crash_signature(crash_info)
        
        # Check if we've seen this crash before
        is_duplicate = signature in self.crash_signatures
        
        # If it's not a duplicate, add it to our tracking
        if not is_duplicate:
            self.crash_signatures.add(signature)
            
            # Add to crashes list, limiting the size
            self.crashes.append({
                "test_case_id": result.test_case.id,
                "signature": signature,
                "info": crash_info,
                "timestamp": time.time()
            })
            
            # Trim the crashes list if it's too long
            if len(self.crashes) > self.max_crashes_to_track:
                self.crashes.pop(0)
        
        # Determine severity
        severity = self._determine_severity(crash_info)
        
        # Create analysis results
        analysis = {
            "is_crash": True,
            "signature": signature,
            "is_duplicate": is_duplicate,
            "error_type": crash_info["error_type"],
            "error_message": crash_info["error_message"],
            "stack_trace": crash_info["stack_trace"],
            "severity": severity,
            "timestamp": time.time()
        }
        
        # Log the analysis
        logger.info(f"Crash analysis for test case {result.test_case.id}: {analysis['error_type']} (severity: {severity})")
        
        return analysis
    
    def _extract_crash_info(self, result: TestResult) -> Dict[str, Any]:
        """Extract crash information from a test result.
        
        Args:
            result: Test result with a crash
            
        Returns:
            Extracted crash information
        """
        # Initialize crash info
        crash_info = {
            "exit_code": result.exit_code,
            "error_type": "UNKNOWN",
            "error_message": "",
            "stack_trace": [],
            "stderr": result.stderr,
            "stdout": result.stdout
        }
        
        # Extract error message and type from stderr
        if result.stderr:
            # Try to find the error message
            error_message = result.stderr.strip()
            
            # Try to determine the error type
            error_type = "UNKNOWN"
            for pattern, type_name in self.error_patterns:
                if re.search(pattern, error_message, re.IGNORECASE):
                    error_type = type_name
                    break
            
            # Extract stack trace if present
            stack_trace = self._extract_stack_trace(result.stderr)
            
            # Update crash info
            crash_info["error_message"] = error_message
            crash_info["error_type"] = error_type
            crash_info["stack_trace"] = stack_trace
        
        return crash_info
    
    def _extract_stack_trace(self, stderr: str) -> List[Dict[str, str]]:
        """Extract stack trace from stderr.
        
        Args:
            stderr: Standard error output
            
        Returns:
            Extracted stack trace
        """
        # This is a simplified implementation; a real implementation would parse the actual stack trace format
        stack_trace = []
        
        # Look for stack trace lines
        for line in stderr.split("\n"):
            # Skip empty lines
            if not line.strip():
                continue
            
            # Try to match a stack trace line
            match = re.search(r"at\s+([^\s]+)\s+\(([^:]+):(\d+):(\d+)\)", line)
            if match:
                function_name = match.group(1)
                file_name = match.group(2)
                line_number = match.group(3)
                column_number = match.group(4)
                
                stack_trace.append({
                    "function": function_name,
                    "file": file_name,
                    "line": line_number,
                    "column": column_number
                })
        
        return stack_trace
    
    def _generate_crash_signature(self, crash_info: Dict[str, Any]) -> str:
        """Generate a signature for a crash.
        
        Args:
            crash_info: Crash information
            
        Returns:
            Crash signature
        """
        # Create a signature based on error type and stack trace
        signature_parts = [crash_info["error_type"]]
        
        # Add stack trace functions if available
        if crash_info["stack_trace"]:
            for frame in crash_info["stack_trace"][:3]:  # Use top 3 frames
                signature_parts.append(frame.get("function", "unknown"))
        else:
            # No stack trace, use a hash of the error message
            error_hash = hashlib.md5(crash_info["error_message"].encode()).hexdigest()[:8]
            signature_parts.append(error_hash)
        
        # Join the parts
        return ":".join(signature_parts)
    
    def _determine_severity(self, crash_info: Dict[str, Any]) -> str:
        """Determine the severity of a crash.
        
        Args:
            crash_info: Crash information
            
        Returns:
            Severity level
        """
        # Determine severity based on error type
        error_type = crash_info["error_type"]
        
        if error_type in ["SEGMENTATION_FAULT", "ASSERTION_FAILURE"]:
            return "CRITICAL"
        elif error_type in ["RUNTIME_ERROR", "ABORT", "KILLED"]:
            return "HIGH"
        elif error_type in ["TYPE_ERROR", "REFERENCE_ERROR"]:
            return "MEDIUM"
        elif error_type in ["SYNTAX_ERROR"]:
            return "LOW"
        else:
            return "UNKNOWN"
    
    def get_crash_summary(self) -> Dict[str, Any]:
        """Get a summary of all crashes.
        
        Returns:
            Crash summary
        """
        # Count crashes by type
        crashes_by_type = {}
        for crash in self.crashes:
            error_type = crash["info"]["error_type"]
            crashes_by_type[error_type] = crashes_by_type.get(error_type, 0) + 1
        
        # Count crashes by severity
        crashes_by_severity = {}
        for crash in self.crashes:
            severity = self._determine_severity(crash["info"])
            crashes_by_severity[severity] = crashes_by_severity.get(severity, 0) + 1
        
        # Create summary
        return {
            "total_crashes": len(self.crashes),
            "unique_crashes": len(self.crash_signatures),
            "crashes_by_type": crashes_by_type,
            "crashes_by_severity": crashes_by_severity,
            "latest_crashes": self.crashes[-10:]  # Last 10 crashes
        }


def main():
    """Main entry point for testing the crash analyzer."""
    # Create a crash analyzer
    analyzer = CrashAnalyzer()
    
    # Create a mock test result with a crash
    class MockTestCase:
        def __init__(self, id):
            self.id = id
            self.content = "x ← 1 / 0"
            self.generator_type = "mock"
            self.metadata = {}
    
    test_result = TestResult(
        test_case=MockTestCase("test_case_1"),
        success=False,
        exit_code=1,
        stdout="",
        stderr="Runtime error: division by zero\nat evaluate (interpreter.rs:123:45)\nat execute (vm.rs:67:89)\nat main (main.rs:12:34)",
        execution_time_seconds=0.1,
        crash=True,
        timeout=False
    )
    
    # Analyze the crash
    analysis = analyzer.analyze(test_result)
    
    # Print the analysis
    print(f"Crash Analysis:")
    print(f"  Signature: {analysis['signature']}")
    print(f"  Error Type: {analysis['error_type']}")
    print(f"  Error Message: {analysis['error_message']}")
    print(f"  Severity: {analysis['severity']}")
    print(f"  Stack Trace: {analysis['stack_trace']}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
