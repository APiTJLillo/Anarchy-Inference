#!/usr/bin/env python3
"""
Stress Testing Framework for Anarchy Inference

This module provides the core infrastructure for stress testing the Anarchy Inference language,
including test generation, resource monitoring, concurrency testing, long-running tests,
load testing, and fault injection.
"""

import os
import sys
import time
import json
import logging
import argparse
import threading
import multiprocessing
from typing import Dict, List, Any, Tuple, Optional, Callable, Union
from dataclasses import dataclass, field
from enum import Enum
import traceback
import random

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(os.path.join(os.path.dirname(__file__), "stress_testing.log")),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger("stress_testing")

class StressTestType(Enum):
    """Types of stress tests that can be performed."""
    MEMORY = "memory"
    COMPUTATIONAL = "computational"
    CONCURRENCY = "concurrency"
    IO = "io"
    LONG_RUNNING = "long_running"
    FAULT_INJECTION = "fault_injection"
    LOAD = "load"

class StressIntensity(Enum):
    """Intensity levels for stress tests."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    EXTREME = "extreme"

@dataclass
class StressTestConfig:
    """Configuration for stress tests."""
    test_types: List[StressTestType] = field(default_factory=lambda: list(StressTestType))
    intensity: StressIntensity = StressIntensity.MEDIUM
    duration_seconds: int = 60
    report_level: str = "standard"
    output_dir: str = field(default_factory=lambda: os.path.join(os.path.dirname(__file__), "results"))
    seed: Optional[int] = None
    parallel_tests: int = 1
    resource_limits: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Initialize derived fields and validate configuration."""
        # Create output directory if it doesn't exist
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
            
        # Set random seed if provided
        if self.seed is not None:
            random.seed(self.seed)
            
        # Set default resource limits if not provided
        if not self.resource_limits:
            self.resource_limits = {
                "memory_mb": 1024,  # 1GB
                "cpu_percent": 90,
                "file_handles": 100,
                "thread_count": 10
            }

@dataclass
class StressTestResult:
    """Results from a stress test."""
    test_name: str
    test_type: StressTestType
    success: bool
    duration_seconds: float
    error_message: Optional[str] = None
    resource_usage: Dict[str, Any] = field(default_factory=dict)
    performance_metrics: Dict[str, Any] = field(default_factory=dict)
    stability_metrics: Dict[str, Any] = field(default_factory=dict)
    issues_detected: List[str] = field(default_factory=list)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the result to a dictionary."""
        return {
            "test_name": self.test_name,
            "test_type": self.test_type.value,
            "success": self.success,
            "duration_seconds": self.duration_seconds,
            "error_message": self.error_message,
            "resource_usage": self.resource_usage,
            "performance_metrics": self.performance_metrics,
            "stability_metrics": self.stability_metrics,
            "issues_detected": self.issues_detected
        }

class StressTestRunner:
    """Runs stress tests and collects results."""
    
    def __init__(self, config: StressTestConfig):
        """Initialize the stress test runner with the given configuration."""
        self.config = config
        self.interpreter = anarchy.Interpreter()
        self.results: List[StressTestResult] = []
        
    def run_all_tests(self) -> List[StressTestResult]:
        """Run all stress tests based on the configuration."""
        logger.info(f"Starting stress test run with intensity: {self.config.intensity.value}")
        
        # Discover and load all test cases
        test_cases = self._discover_test_cases()
        
        # Filter test cases based on configuration
        filtered_tests = self._filter_test_cases(test_cases)
        
        if not filtered_tests:
            logger.warning("No test cases match the current configuration.")
            return []
        
        logger.info(f"Running {len(filtered_tests)} stress tests")
        
        # Run the tests
        if self.config.parallel_tests > 1:
            self._run_tests_parallel(filtered_tests)
        else:
            self._run_tests_sequential(filtered_tests)
        
        # Generate reports
        self._generate_reports()
        
        return self.results
    
    def _discover_test_cases(self) -> List[Dict[str, Any]]:
        """Discover all available test cases."""
        test_cases = []
        
        # Look for test case modules in the stress_test_cases directory
        test_cases_dir = os.path.join(os.path.dirname(__file__), "..", "stress_test_cases")
        
        if not os.path.exists(test_cases_dir):
            logger.warning(f"Test cases directory not found: {test_cases_dir}")
            return []
        
        # Walk through the directory structure to find test case files
        for root, _, files in os.walk(test_cases_dir):
            for file in files:
                if file.endswith("_tests.py"):
                    # Extract test type from directory name
                    dir_name = os.path.basename(root)
                    try:
                        test_type = StressTestType(dir_name)
                    except ValueError:
                        logger.warning(f"Unknown test type directory: {dir_name}")
                        continue
                    
                    # Load the test case file
                    test_file_path = os.path.join(root, file)
                    test_cases.extend(self._load_test_cases_from_file(test_file_path, test_type))
        
        return test_cases
    
    def _load_test_cases_from_file(self, file_path: str, test_type: StressTestType) -> List[Dict[str, Any]]:
        """Load test cases from a file."""
        try:
            # Import the module
            import importlib.util
            spec = importlib.util.spec_from_file_location("test_module", file_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            
            # Look for test case functions
            test_cases = []
            for name in dir(module):
                if name.startswith("stress_test_"):
                    func = getattr(module, name)
                    if callable(func):
                        # Extract metadata from function docstring
                        description = func.__doc__ or "No description"
                        
                        test_cases.append({
                            "name": name,
                            "function": func,
                            "type": test_type,
                            "description": description,
                            "file_path": file_path
                        })
            
            return test_cases
        except Exception as e:
            logger.error(f"Error loading test cases from {file_path}: {e}")
            return []
    
    def _filter_test_cases(self, test_cases: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Filter test cases based on configuration."""
        filtered_tests = []
        
        for test_case in test_cases:
            # Filter by test type
            if test_case["type"] in self.config.test_types:
                filtered_tests.append(test_case)
        
        return filtered_tests
    
    def _run_tests_sequential(self, test_cases: List[Dict[str, Any]]):
        """Run tests sequentially."""
        for test_case in test_cases:
            result = self._run_single_test(test_case)
            self.results.append(result)
    
    def _run_tests_parallel(self, test_cases: List[Dict[str, Any]]):
        """Run tests in parallel."""
        with multiprocessing.Pool(processes=self.config.parallel_tests) as pool:
            self.results = pool.map(self._run_single_test, test_cases)
    
    def _run_single_test(self, test_case: Dict[str, Any]) -> StressTestResult:
        """Run a single stress test."""
        test_name = test_case["name"]
        test_type = test_case["type"]
        test_func = test_case["function"]
        
        logger.info(f"Running stress test: {test_name} ({test_type.value})")
        
        start_time = time.time()
        success = True
        error_message = None
        resource_usage = {}
        performance_metrics = {}
        stability_metrics = {}
        issues_detected = []
        
        try:
            # Set up resource monitoring
            resource_monitor = self._create_resource_monitor()
            resource_monitor.start()
            
            # Run the test function
            test_result = test_func(
                self.interpreter,
                intensity=self.config.intensity,
                duration=self.config.duration_seconds
            )
            
            # Process test result
            if isinstance(test_result, dict):
                if "issues" in test_result:
                    issues_detected.extend(test_result["issues"])
                if "performance" in test_result:
                    performance_metrics.update(test_result["performance"])
                if "stability" in test_result:
                    stability_metrics.update(test_result["stability"])
            
            # Stop resource monitoring
            resource_usage = resource_monitor.stop()
            
        except Exception as e:
            success = False
            error_message = f"{type(e).__name__}: {str(e)}\n{traceback.format_exc()}"
            logger.error(f"Error in stress test {test_name}: {error_message}")
        
        duration = time.time() - start_time
        
        # Create and return the result
        return StressTestResult(
            test_name=test_name,
            test_type=test_type,
            success=success,
            duration_seconds=duration,
            error_message=error_message,
            resource_usage=resource_usage,
            performance_metrics=performance_metrics,
            stability_metrics=stability_metrics,
            issues_detected=issues_detected
        )
    
    def _create_resource_monitor(self):
        """Create a resource monitor for tracking resource usage during tests."""
        # This is a placeholder - the actual implementation will be in the ResourceMonitor class
        from .monitors.resource_monitor import ResourceMonitor
        return ResourceMonitor()
    
    def _generate_reports(self):
        """Generate reports based on test results."""
        # Generate summary report
        summary_report = self._generate_summary_report()
        summary_path = os.path.join(self.config.output_dir, "summary_report.md")
        with open(summary_path, 'w') as f:
            f.write(summary_report)
        
        # Generate detailed report if requested
        if self.config.report_level == "detailed":
            detailed_report = self._generate_detailed_report()
            detailed_path = os.path.join(self.config.output_dir, "detailed_report.md")
            with open(detailed_path, 'w') as f:
                f.write(detailed_report)
        
        # Save raw results as JSON
        results_json = [result.to_dict() for result in self.results]
        json_path = os.path.join(self.config.output_dir, "stress_test_results.json")
        with open(json_path, 'w') as f:
            json.dump(results_json, f, indent=2)
        
        logger.info(f"Reports generated in {self.config.output_dir}")
    
    def _generate_summary_report(self) -> str:
        """Generate a summary report of the stress test results."""
        report = "# Anarchy Inference Stress Test Summary Report\n\n"
        
        # Add configuration information
        report += "## Test Configuration\n\n"
        report += f"- **Intensity**: {self.config.intensity.value}\n"
        report += f"- **Duration**: {self.config.duration_seconds} seconds\n"
        report += f"- **Test Types**: {', '.join(t.value for t in self.config.test_types)}\n"
        report += f"- **Parallel Tests**: {self.config.parallel_tests}\n\n"
        
        # Add overall statistics
        total_tests = len(self.results)
        successful_tests = sum(1 for r in self.results if r.success)
        failed_tests = total_tests - successful_tests
        
        report += "## Overall Results\n\n"
        report += f"- **Total Tests**: {total_tests}\n"
        report += f"- **Successful Tests**: {successful_tests}\n"
        report += f"- **Failed Tests**: {failed_tests}\n"
        
        if total_tests > 0:
            success_rate = (successful_tests / total_tests) * 100
            report += f"- **Success Rate**: {success_rate:.2f}%\n\n"
        
        # Add summary by test type
        report += "## Results by Test Type\n\n"
        
        for test_type in StressTestType:
            type_results = [r for r in self.results if r.test_type == test_type]
            if type_results:
                type_total = len(type_results)
                type_successful = sum(1 for r in type_results if r.success)
                type_failed = type_total - type_successful
                
                report += f"### {test_type.value.replace('_', ' ').title()} Tests\n\n"
                report += f"- **Total**: {type_total}\n"
                report += f"- **Successful**: {type_successful}\n"
                report += f"- **Failed**: {type_failed}\n"
                
                if type_total > 0:
                    type_success_rate = (type_successful / type_total) * 100
                    report += f"- **Success Rate**: {type_success_rate:.2f}%\n\n"
        
        # Add summary of issues detected
        all_issues = [issue for result in self.results for issue in result.issues_detected]
        if all_issues:
            report += "## Issues Detected\n\n"
            for issue in all_issues:
                report += f"- {issue}\n"
            report += "\n"
        
        # Add timestamp
        report += f"\nGenerated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        return report
    
    def _generate_detailed_report(self) -> str:
        """Generate a detailed report of the stress test results."""
        report = "# Anarchy Inference Stress Test Detailed Report\n\n"
        
        # Add configuration information (same as summary report)
        report += "## Test Configuration\n\n"
        report += f"- **Intensity**: {self.config.intensity.value}\n"
        report += f"- **Duration**: {self.config.duration_seconds} seconds\n"
        report += f"- **Test Types**: {', '.join(t.value for t in self.config.test_types)}\n"
        report += f"- **Parallel Tests**: {self.config.parallel_tests}\n\n"
        
        # Add detailed results for each test
        report += "## Detailed Test Results\n\n"
        
        for result in self.results:
            report += f"### {result.test_name}\n\n"
            report += f"- **Type**: {result.test_type.value}\n"
            report += f"- **Success**: {'Yes' if result.success else 'No'}\n"
            report += f"- **Duration**: {result.duration_seconds:.2f} seconds\n"
            
            if result.error_message:
                report += f"- **Error**: {result.error_message}\n"
            
            if result.resource_usage:
                report += "- **Resource Usage**:\n"
                for resource, value in result.resource_usage.items():
                    report += f"  - {resource}: {value}\n"
            
            if result.performance_metrics:
                report += "- **Performance Metrics**:\n"
                for metric, value in result.performance_metrics.items():
                    report += f"  - {metric}: {value}\n"
            
            if result.stability_metrics:
                report += "- **Stability Metrics**:\n"
                for metric, value in result.stability_metrics.items():
                    report += f"  - {metric}: {value}\n"
            
            if result.issues_detected:
                report += "- **Issues Detected**:\n"
                for issue in result.issues_detected:
                    report += f"  - {issue}\n"
            
            report += "\n"
        
        # Add timestamp
        report += f"\nGenerated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        return report

def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Anarchy Inference Stress Testing Framework")
    
    parser.add_argument(
        "--type",
        choices=[t.value for t in StressTestType],
        action="append",
        dest="test_types",
        help="Types of stress tests to run (can specify multiple)"
    )
    
    parser.add_argument(
        "--intensity",
        choices=[i.value for i in StressIntensity],
        default=StressIntensity.MEDIUM.value,
        help="Intensity level of stress tests"
    )
    
    parser.add_argument(
        "--duration",
        type=int,
        default=60,
        help="Duration of stress tests in seconds"
    )
    
    parser.add_argument(
        "--report-level",
        choices=["standard", "detailed"],
        default="standard",
        help="Level of detail in reports"
    )
    
    parser.add_argument(
        "--output-dir",
        default=os.path.join(os.path.dirname(__file__), "results"),
        help="Directory for output files"
    )
    
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducible tests"
    )
    
    parser.add_argument(
        "--parallel",
        type=int,
        default=1,
        help="Number of tests to run in parallel"
    )
    
    args = parser.parse_args()
    
    # Convert string values to enum values
    intensity = StressIntensity(args.intensity)
    
    test_types = []
    if args.test_types:
        test_types = [StressTestType(t) for t in args.test_types]
    else:
        # Default to all test types if none specified
        test_types = list(StressTestType)
    
    # Create and return the configuration
    return StressTestConfig(
        test_types=test_types,
        intensity=intensity,
        duration_seconds=args.duration,
        report_level=args.report_level,
        output_dir=args.output_dir,
        seed=args.seed,
        parallel_tests=args.parallel
    )

def main():
    """Main entry point for the stress testing framework."""
    try:
        # Parse command line arguments
        config = parse_args()
        
        # Create and run the stress test runner
        runner = StressTestRunner(config)
        results = runner.run_all_tests()
        
        # Print summary
        total_tests = len(results)
        successful_tests = sum(1 for r in results if r.success)
        failed_tests = total_tests - successful_tests
        
        print(f"\nStress Test Summary:")
        print(f"- Total Tests: {total_tests}")
        print(f"- Successful Tests: {successful_tests}")
        print(f"- Failed Tests: {failed_tests}")
        
        if total_tests > 0:
            success_rate = (successful_tests / total_tests) * 100
            print(f"- Success Rate: {success_rate:.2f}%")
        
        print(f"\nDetailed reports available in: {config.output_dir}")
        
        # Return success if all tests passed
        return 0 if failed_tests == 0 else 1
    
    except Exception as e:
        logger.error(f"Error in stress testing framework: {e}")
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())
