"""
Integration tests for the Validation Tools.

This module provides tests that verify the integration between all validation tool components.
"""

import os
import sys
import unittest
import tempfile
from typing import List, Dict, Any

# Add the parent directory to the path so we can import the validation tools
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from validation_tools.static_analyzer import StaticAnalyzer, ValidationResult
from validation_tools.runtime_verifier import RuntimeVerifier
from validation_tools.security_scanner import SecurityScanner
from validation_tools.resource_analyzer import ResourceUsageAnalyzer

class ValidationManagerTests(unittest.TestCase):
    """Tests for the integration of all validation tools."""
    
    def setUp(self):
        """Set up test environment."""
        # Create temporary directory for test files
        self.temp_dir = tempfile.TemporaryDirectory()
        
        # Create test files
        self.create_test_files()
        
        # Initialize validation tools
        self.static_analyzer = StaticAnalyzer()
        self.runtime_verifier = RuntimeVerifier()
        self.security_scanner = SecurityScanner()
        self.resource_analyzer = ResourceUsageAnalyzer()
    
    def tearDown(self):
        """Clean up test environment."""
        self.temp_dir.cleanup()
    
    def create_test_files(self):
        """Create test files with various issues for validation."""
        # Test file with syntax issues
        with open(os.path.join(self.temp_dir.name, "syntax_issues.ai"), "w") as f:
            f.write("""
            ι main() {
                ι x = 5
                ι y = 10
                ι z = x + y
                print(z)
            }
            """)
        
        # Test file with security issues
        with open(os.path.join(self.temp_dir.name, "security_issues.ai"), "w") as f:
            f.write("""
            ι main() {
                ι user_input = 🎤("Enter your name: ")
                ι query = "SELECT * FROM users WHERE name = '" + user_input + "'"
                ⇓(query)
                
                ι password = "hardcoded_password123"
                ι api_key = "api_key_12345abcdef"
                
                ι cmd = "ls " + user_input
                exec(cmd)
            }
            """)
        
        # Test file with resource issues
        with open(os.path.join(self.temp_dir.name, "resource_issues.ai"), "w") as f:
            f.write("""
            ι main() {
                ι result = ""
                for (ι i = 0; i < 1000; i++) {
                    result = result + i
                }
                
                ι resource = new Resource()
                // No free(resource)
                
                ι deeply_nested_function() {
                    for (ι i = 0; i < 10; i++) {
                        for (ι j = 0; j < 10; j++) {
                            for (ι k = 0; k < 10; k++) {
                                // Deep nesting
                            }
                        }
                    }
                }
            }
            """)
    
    def test_static_analysis(self):
        """Test static analysis on test files."""
        # Analyze syntax issues file
        results = self.static_analyzer.analyze_file(
            os.path.join(self.temp_dir.name, "syntax_issues.ai")
        )
        
        # Verify that issues were found
        self.assertGreater(len(results), 0)
        
        # Verify that at least one missing semicolon issue was found
        self.assertTrue(any(r.rule_id == "SYNTAX001" for r in results))
    
    def test_security_scanning(self):
        """Test security scanning on test files."""
        # Scan security issues file
        results = self.security_scanner.scan_file(
            os.path.join(self.temp_dir.name, "security_issues.ai")
        )
        
        # Verify that issues were found
        self.assertGreater(len(results), 0)
        
        # Verify that SQL injection issue was found
        self.assertTrue(any(r.rule_id == "SEC001" for r in results))
        
        # Verify that secret detection issue was found
        self.assertTrue(any(r.rule_id == "SEC003" for r in results))
        
        # Verify that command injection issue was found
        self.assertTrue(any(r.rule_id == "SEC004" for r in results))
    
    def test_resource_analysis(self):
        """Test resource usage analysis on test files."""
        # Analyze resource issues file
        results = self.resource_analyzer.analyze_file(
            os.path.join(self.temp_dir.name, "resource_issues.ai")
        )
        
        # Verify that issues were found
        self.assertGreater(len(results), 0)
        
        # Verify that inefficient string concatenation issue was found
        self.assertTrue(any(r.rule_id == "RES003" for r in results))
        
        # Verify that memory leak issue was found
        self.assertTrue(any(r.rule_id == "RES002" for r in results))
        
        # Verify that complexity issue was found
        self.assertTrue(any(r.rule_id == "RES005" for r in results))
    
    def test_directory_analysis(self):
        """Test analyzing an entire directory."""
        # Analyze all files in the temp directory
        static_results = self.static_analyzer.analyze_directory(self.temp_dir.name)
        security_results = self.security_scanner.scan_directory(self.temp_dir.name)
        resource_results = self.resource_analyzer.analyze_directory(self.temp_dir.name)
        
        # Verify that issues were found by each tool
        self.assertGreater(len(static_results), 0)
        self.assertGreater(len(security_results), 0)
        self.assertGreater(len(resource_results), 0)
        
        # Verify that the total number of issues is the sum of issues from each tool
        all_results = static_results + security_results + resource_results
        self.assertEqual(
            len(all_results),
            len(static_results) + len(security_results) + len(resource_results)
        )
    
    def test_report_generation(self):
        """Test report generation from validation results."""
        # Get some results
        results = self.static_analyzer.analyze_file(
            os.path.join(self.temp_dir.name, "syntax_issues.ai")
        )
        
        # Generate reports in different formats
        text_report = self.static_analyzer.generate_report(results, "text")
        json_report = self.static_analyzer.generate_report(results, "json")
        html_report = self.static_analyzer.generate_report(results, "html")
        
        # Verify that reports were generated
        self.assertIsInstance(text_report, str)
        self.assertIsInstance(json_report, str)
        self.assertIsInstance(html_report, str)
        
        # Verify that reports contain the expected content
        self.assertIn("Static Analysis Report", text_report)
        self.assertIn("rule_id", json_report)
        self.assertIn("<html>", html_report)
    
    def test_combined_validation(self):
        """Test running all validation tools on the same files."""
        # Get results from all tools
        static_results = self.static_analyzer.analyze_file(
            os.path.join(self.temp_dir.name, "security_issues.ai")
        )
        security_results = self.security_scanner.scan_file(
            os.path.join(self.temp_dir.name, "security_issues.ai")
        )
        resource_results = self.resource_analyzer.analyze_file(
            os.path.join(self.temp_dir.name, "security_issues.ai")
        )
        
        # Combine results
        all_results = static_results + security_results + resource_results
        
        # Verify that we have results from each tool
        self.assertTrue(any(r.tool == "StaticAnalyzer" for r in all_results))
        self.assertTrue(any(r.tool == "SecurityScanner" for r in all_results))
        self.assertTrue(any(r.tool == "ResourceAnalyzer" for r in all_results))
        
        # Generate a combined report
        combined_report = self.static_analyzer.generate_report(all_results, "text")
        
        # Verify that the combined report contains results from all tools
        self.assertIn("StaticAnalyzer", combined_report)
        self.assertIn("SecurityScanner", combined_report)
        self.assertIn("ResourceAnalyzer", combined_report)


class ValidationManagerIntegrationTests(unittest.TestCase):
    """Integration tests for the validation tools working together."""
    
    def test_validation_manager(self):
        """Test the ValidationManager class that integrates all tools."""
        # This is a placeholder for testing the ValidationManager
        # which would be implemented to coordinate all validation tools
        pass


if __name__ == "__main__":
    unittest.main()
