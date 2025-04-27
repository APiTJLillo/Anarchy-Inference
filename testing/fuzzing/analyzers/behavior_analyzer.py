#!/usr/bin/env python3
"""
Behavior Analyzer for Anarchy Inference Fuzzing.

This module provides functionality for analyzing unexpected behaviors that don't result in crashes
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
logger = logging.getLogger("behavior_analyzer")

class BehaviorAnalyzer:
    """Analyzes unexpected behaviors that don't result in crashes."""
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the behavior analyzer.
        
        Args:
            config: Optional configuration for the analyzer
        """
        self.config = config or {}
        
        # Default configuration values
        self.min_similarity_threshold = self.config.get("min_similarity_threshold", 0.8)
        self.max_behaviors_to_track = self.config.get("max_behaviors_to_track", 100)
        
        # Behavior tracking
        self.behaviors: List[Dict[str, Any]] = []
        self.behavior_signatures: Set[str] = set()
        
        # Common warning patterns
        self.warning_patterns = [
            # Performance warnings
            (r"performance warning", "PERFORMANCE_WARNING"),
            (r"slow operation", "PERFORMANCE_WARNING"),
            (r"inefficient", "PERFORMANCE_WARNING"),
            
            # Deprecation warnings
            (r"deprecated", "DEPRECATION_WARNING"),
            (r"will be removed", "DEPRECATION_WARNING"),
            (r"use .* instead", "DEPRECATION_WARNING"),
            
            # Memory warnings
            (r"memory usage", "MEMORY_WARNING"),
            (r"high memory", "MEMORY_WARNING"),
            (r"memory pressure", "MEMORY_WARNING"),
            
            # Resource warnings
            (r"resource leak", "RESOURCE_WARNING"),
            (r"unclosed", "RESOURCE_WARNING"),
            (r"not released", "RESOURCE_WARNING"),
            
            # Potential bugs
            (r"potential bug", "POTENTIAL_BUG"),
            (r"suspicious", "POTENTIAL_BUG"),
            (r"unexpected", "POTENTIAL_BUG"),
            
            # Overflow warnings
            (r"overflow", "OVERFLOW_WARNING"),
            (r"underflow", "OVERFLOW_WARNING"),
            (r"precision loss", "OVERFLOW_WARNING"),
            
            # Other warnings
            (r"warning", "GENERAL_WARNING"),
            (r"note:", "GENERAL_WARNING")
        ]
        
        # Interesting output patterns
        self.interesting_output_patterns = [
            # Unusual output
            (r"NaN", "UNUSUAL_OUTPUT"),
            (r"Infinity", "UNUSUAL_OUTPUT"),
            (r"-Infinity", "UNUSUAL_OUTPUT"),
            (r"undefined", "UNUSUAL_OUTPUT"),
            
            # Excessive output
            (r".{1000,}", "EXCESSIVE_OUTPUT"),
            
            # Empty output
            (r"^\s*$", "EMPTY_OUTPUT"),
            
            # Binary output
            (r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\xFF]", "BINARY_OUTPUT")
        ]
    
    def analyze(self, result: TestResult) -> Dict[str, Any]:
        """Analyze an unexpected behavior.
        
        Args:
            result: Test result with an unexpected behavior
            
        Returns:
            Analysis results
        """
        # Skip crashes (those are handled by the crash analyzer)
        if result.crash:
            return {"is_interesting": False}
        
        # Extract behavior information
        behavior_info = self._extract_behavior_info(result)
        
        # Check if it's an interesting behavior
        if not behavior_info["is_interesting"]:
            return {"is_interesting": False}
        
        # Generate a behavior signature
        signature = self._generate_behavior_signature(behavior_info)
        
        # Check if we've seen this behavior before
        is_duplicate = signature in self.behavior_signatures
        
        # If it's not a duplicate, add it to our tracking
        if not is_duplicate:
            self.behavior_signatures.add(signature)
            
            # Add to behaviors list, limiting the size
            self.behaviors.append({
                "test_case_id": result.test_case.id,
                "signature": signature,
                "info": behavior_info,
                "timestamp": time.time()
            })
            
            # Trim the behaviors list if it's too long
            if len(self.behaviors) > self.max_behaviors_to_track:
                self.behaviors.pop(0)
        
        # Determine severity
        severity = self._determine_severity(behavior_info)
        
        # Create analysis results
        analysis = {
            "is_interesting": True,
            "signature": signature,
            "is_duplicate": is_duplicate,
            "behavior_type": behavior_info["behavior_type"],
            "behavior_message": behavior_info["behavior_message"],
            "severity": severity,
            "timestamp": time.time()
        }
        
        # Log the analysis
        logger.info(f"Behavior analysis for test case {result.test_case.id}: {analysis['behavior_type']} (severity: {severity})")
        
        return analysis
    
    def _extract_behavior_info(self, result: TestResult) -> Dict[str, Any]:
        """Extract behavior information from a test result.
        
        Args:
            result: Test result
            
        Returns:
            Extracted behavior information
        """
        # Initialize behavior info
        behavior_info = {
            "exit_code": result.exit_code,
            "is_interesting": False,
            "behavior_type": "UNKNOWN",
            "behavior_message": "",
            "stderr": result.stderr,
            "stdout": result.stdout,
            "execution_time_seconds": result.execution_time_seconds
        }
        
        # Check for warnings in stderr
        if result.stderr:
            for pattern, type_name in self.warning_patterns:
                match = re.search(pattern, result.stderr, re.IGNORECASE)
                if match:
                    behavior_info["is_interesting"] = True
                    behavior_info["behavior_type"] = type_name
                    behavior_info["behavior_message"] = result.stderr.strip()
                    break
        
        # Check for interesting output in stdout
        if result.stdout and not behavior_info["is_interesting"]:
            for pattern, type_name in self.interesting_output_patterns:
                match = re.search(pattern, result.stdout, re.IGNORECASE | re.DOTALL)
                if match:
                    behavior_info["is_interesting"] = True
                    behavior_info["behavior_type"] = type_name
                    behavior_info["behavior_message"] = f"Interesting output: {type_name}"
                    break
        
        # Check for slow execution
        if result.execution_time_seconds > 1.0 and not behavior_info["is_interesting"]:
            behavior_info["is_interesting"] = True
            behavior_info["behavior_type"] = "SLOW_EXECUTION"
            behavior_info["behavior_message"] = f"Slow execution: {result.execution_time_seconds:.2f} seconds"
        
        # Check for successful execution with non-zero exit code
        if result.success and result.exit_code != 0 and not behavior_info["is_interesting"]:
            behavior_info["is_interesting"] = True
            behavior_info["behavior_type"] = "SUCCESS_WITH_NON_ZERO_EXIT"
            behavior_info["behavior_message"] = f"Success with non-zero exit code: {result.exit_code}"
        
        return behavior_info
    
    def _generate_behavior_signature(self, behavior_info: Dict[str, Any]) -> str:
        """Generate a signature for a behavior.
        
        Args:
            behavior_info: Behavior information
            
        Returns:
            Behavior signature
        """
        # Create a signature based on behavior type and message
        signature_parts = [behavior_info["behavior_type"]]
        
        # Add a hash of the behavior message
        message_hash = hashlib.md5(behavior_info["behavior_message"].encode()).hexdigest()[:8]
        signature_parts.append(message_hash)
        
        # Join the parts
        return ":".join(signature_parts)
    
    def _determine_severity(self, behavior_info: Dict[str, Any]) -> str:
        """Determine the severity of a behavior.
        
        Args:
            behavior_info: Behavior information
            
        Returns:
            Severity level
        """
        # Determine severity based on behavior type
        behavior_type = behavior_info["behavior_type"]
        
        if behavior_type in ["POTENTIAL_BUG", "RESOURCE_WARNING"]:
            return "HIGH"
        elif behavior_type in ["MEMORY_WARNING", "OVERFLOW_WARNING"]:
            return "MEDIUM"
        elif behavior_type in ["PERFORMANCE_WARNING", "DEPRECATION_WARNING"]:
            return "LOW"
        elif behavior_type in ["UNUSUAL_OUTPUT", "EXCESSIVE_OUTPUT", "BINARY_OUTPUT"]:
            return "INFO"
        else:
            return "UNKNOWN"
    
    def get_behavior_summary(self) -> Dict[str, Any]:
        """Get a summary of all interesting behaviors.
        
        Returns:
            Behavior summary
        """
        # Count behaviors by type
        behaviors_by_type = {}
        for behavior in self.behaviors:
            behavior_type = behavior["info"]["behavior_type"]
            behaviors_by_type[behavior_type] = behaviors_by_type.get(behavior_type, 0) + 1
        
        # Count behaviors by severity
        behaviors_by_severity = {}
        for behavior in self.behaviors:
            severity = self._determine_severity(behavior["info"])
            behaviors_by_severity[severity] = behaviors_by_severity.get(severity, 0) + 1
        
        # Create summary
        return {
            "total_behaviors": len(self.behaviors),
            "unique_behaviors": len(self.behavior_signatures),
            "behaviors_by_type": behaviors_by_type,
            "behaviors_by_severity": behaviors_by_severity,
            "latest_behaviors": self.behaviors[-10:]  # Last 10 behaviors
        }


def main():
    """Main entry point for testing the behavior analyzer."""
    # Create a behavior analyzer
    analyzer = BehaviorAnalyzer()
    
    # Create a mock test result with an interesting behavior
    class MockTestCase:
        def __init__(self, id):
            self.id = id
            self.content = "x ← 1000000 * 1000000"
            self.generator_type = "mock"
            self.metadata = {}
    
    test_result = TestResult(
        test_case=MockTestCase("test_case_1"),
        success=True,
        exit_code=0,
        stdout="1000000000000",
        stderr="Warning: Large number might cause precision loss",
        execution_time_seconds=0.1,
        crash=False,
        timeout=False
    )
    
    # Analyze the behavior
    analysis = analyzer.analyze(test_result)
    
    # Print the analysis
    print(f"Behavior Analysis:")
    print(f"  Is Interesting: {analysis['is_interesting']}")
    if analysis['is_interesting']:
        print(f"  Signature: {analysis['signature']}")
        print(f"  Behavior Type: {analysis['behavior_type']}")
        print(f"  Behavior Message: {analysis['behavior_message']}")
        print(f"  Severity: {analysis['severity']}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
