#!/usr/bin/env python3
"""
Stress Test Runner for Anarchy Inference.

This module integrates all stress testing components and provides a unified interface
for running comprehensive stress tests.
"""

import os
import sys
import time
import logging
import argparse
import json
from typing import Dict, List, Any, Optional, Tuple
from enum import Enum
from dataclasses import dataclass
import concurrent.futures

# Add parent directory to path to import stress testing modules
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import stress testing components
from stress_testing.stress_testing import (
    StressTestConfig, StressTestType, StressIntensity, StressTestRunner
)
from stress_testing.generators.stress_test_generator import StressTestGenerator, TestSequencer
from stress_testing.monitors.resource_monitor import ResourceMonitor, MemoryMonitor, CPUMonitor
from stress_testing.concurrency.concurrency_tester import ConcurrencyTester
from stress_testing.long_running.long_running_test_manager import LongRunningTestManager
from stress_testing.load_testing.load_test_controller import LoadTestController, LoadPattern, LoadTestConfig
from stress_testing.fault_injection.fault_injector import FaultInjector, FaultType, FaultInjectionConfig

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("stress_test_runner")

class StressTestSuite(Enum):
    """Predefined stress test suites."""
    QUICK = "quick"               # Quick tests for rapid feedback
    STANDARD = "standard"         # Standard test suite for regular testing
    COMPREHENSIVE = "comprehensive"  # Comprehensive test suite for thorough testing
    NIGHTLY = "nightly"           # Extended test suite for nightly builds
    RELEASE = "release"           # Full test suite for release validation


@dataclass
class IntegratedStressTestConfig:
    """Configuration for integrated stress testing."""
    suite: StressTestSuite
    output_dir: str
    parallel_tests: int = 1
    include_components: List[str] = None  # Components to include (all if None)
    exclude_components: List[str] = None  # Components to exclude (none if None)
    seed: Optional[int] = None
    duration_multiplier: float = 1.0  # Multiplier for test durations


@dataclass
class IntegratedStressTestResult:
    """Results from integrated stress testing."""
    config: IntegratedStressTestConfig
    component_results: Dict[str, Any]
    overall_success: bool
    start_time: float
    end_time: float
    issues_detected: List[str]


class IntegratedStressTestRunner:
    """Runs integrated stress tests using all stress testing components."""
    
    def __init__(self, config: IntegratedStressTestConfig):
        """Initialize the integrated stress test runner.
        
        Args:
            config: Configuration for integrated stress testing
        """
        self.config = config
        self.component_results = {}
        self.issues_detected = []
        
        # Create output directory if it doesn't exist
        if not os.path.exists(config.output_dir):
            os.makedirs(config.output_dir)
        
        # Set random seed if specified
        if config.seed is not None:
            import random
            random.seed(config.seed)
    
    def run_all_tests(self) -> IntegratedStressTestResult:
        """Run all stress tests.
        
        Returns:
            Results of integrated stress testing
        """
        start_time = time.time()
        logger.info(f"Starting integrated stress tests with suite: {self.config.suite.value}")
        
        # Determine which components to run based on configuration
        components_to_run = self._get_components_to_run()
        
        # Run tests for each component
        if self.config.parallel_tests > 1 and len(components_to_run) > 1:
            # Run components in parallel
            with concurrent.futures.ThreadPoolExecutor(max_workers=self.config.parallel_tests) as executor:
                futures = {}
                for component in components_to_run:
                    future = executor.submit(self._run_component_tests, component)
                    futures[future] = component
                
                for future in concurrent.futures.as_completed(futures):
                    component = futures[future]
                    try:
                        result = future.result()
                        self.component_results[component] = result
                    except Exception as e:
                        logger.error(f"Error running {component} tests: {e}")
                        self.issues_detected.append(f"Error in {component}: {str(e)}")
        else:
            # Run components sequentially
            for component in components_to_run:
                try:
                    result = self._run_component_tests(component)
                    self.component_results[component] = result
                except Exception as e:
                    logger.error(f"Error running {component} tests: {e}")
                    self.issues_detected.append(f"Error in {component}: {str(e)}")
        
        # Determine overall success
        overall_success = all(
            result.get("success", False) 
            for result in self.component_results.values()
        )
        
        end_time = time.time()
        logger.info(f"Integrated stress tests completed in {end_time - start_time:.2f} seconds")
        
        return IntegratedStressTestResult(
            config=self.config,
            component_results=self.component_results,
            overall_success=overall_success,
            start_time=start_time,
            end_time=end_time,
            issues_detected=self.issues_detected
        )
    
    def _get_components_to_run(self) -> List[str]:
        """Get the list of components to run based on configuration.
        
        Returns:
            List of component names to run
        """
        # All available components
        all_components = [
            "standard_stress",
            "concurrency",
            "long_running",
            "load_testing",
            "fault_injection"
        ]
        
        # Filter based on include/exclude lists
        if self.config.include_components:
            components = [c for c in all_components if c in self.config.include_components]
        else:
            components = all_components.copy()
        
        if self.config.exclude_components:
            components = [c for c in components if c not in self.config.exclude_components]
        
        # Filter based on test suite
        if self.config.suite == StressTestSuite.QUICK:
            # Only run standard stress tests for quick suite
            components = [c for c in components if c == "standard_stress"]
        
        elif self.config.suite == StressTestSuite.STANDARD:
            # Exclude long-running tests for standard suite
            components = [c for c in components if c != "long_running"]
        
        # All components are included for COMPREHENSIVE, NIGHTLY, and RELEASE suites
        
        return components
    
    def _run_component_tests(self, component: str) -> Dict[str, Any]:
        """Run tests for a specific component.
        
        Args:
            component: Component to run tests for
            
        Returns:
            Test results for the component
        """
        logger.info(f"Running {component} tests")
        
        if component == "standard_stress":
            return self._run_standard_stress_tests()
        
        elif component == "concurrency":
            return self._run_concurrency_tests()
        
        elif component == "long_running":
            return self._run_long_running_tests()
        
        elif component == "load_testing":
            return self._run_load_tests()
        
        elif component == "fault_injection":
            return self._run_fault_injection_tests()
        
        else:
            raise ValueError(f"Unknown component: {component}")
    
    def _run_standard_stress_tests(self) -> Dict[str, Any]:
        """Run standard stress tests.
        
        Returns:
            Results of standard stress tests
        """
        # Determine test types and intensity based on suite
        if self.config.suite == StressTestSuite.QUICK:
            test_types = [StressTestType.MEMORY, StressTestType.COMPUTATIONAL]
            intensity = StressIntensity.LOW
            duration = 30
        
        elif self.config.suite == StressTestSuite.STANDARD:
            test_types = list(StressTestType)
            intensity = StressIntensity.MEDIUM
            duration = 60
        
        elif self.config.suite == StressTestSuite.COMPREHENSIVE:
            test_types = list(StressTestType)
            intensity = StressIntensity.HIGH
            duration = 120
        
        elif self.config.suite == StressTestSuite.NIGHTLY:
            test_types = list(StressTestType)
            intensity = StressIntensity.HIGH
            duration = 300
        
        elif self.config.suite == StressTestSuite.RELEASE:
            test_types = list(StressTestType)
            intensity = StressIntensity.EXTREME
            duration = 600
        
        # Apply duration multiplier
        duration = int(duration * self.config.duration_multiplier)
        
        # Create configuration
        config = StressTestConfig(
            test_types=test_types,
            intensity=intensity,
            duration_seconds=duration,
            report_level="detailed",
            output_dir=os.path.join(self.config.output_dir, "standard_stress"),
            seed=self.config.seed,
            parallel_tests=min(self.config.parallel_tests, 4)  # Limit parallelism for standard tests
        )
        
        # Run tests
        runner = StressTestRunner(config)
        results = runner.run_all_tests()
        
        # Process results
        success = all(result.success for result in results)
        
        return {
            "component": "standard_stress",
            "success": success,
            "results": results,
            "issues": [
                f"{result.test_name}: {result.error_message}"
                for result in results
                if not result.success and result.error_message
            ]
        }
    
    def _run_concurrency_tests(self) -> Dict[str, Any]:
        """Run concurrency tests.
        
        Returns:
            Results of concurrency tests
        """
        # Determine thread count and duration based on suite
        if self.config.suite == StressTestSuite.QUICK:
            max_threads = 4
            duration = 30
        
        elif self.config.suite == StressTestSuite.STANDARD:
            max_threads = 8
            duration = 60
        
        elif self.config.suite == StressTestSuite.COMPREHENSIVE:
            max_threads = 16
            duration = 120
        
        elif self.config.suite == StressTestSuite.NIGHTLY:
            max_threads = 32
            duration = 300
        
        elif self.config.suite == StressTestSuite.RELEASE:
            max_threads = 64
            duration = 600
        
        # Apply duration multiplier
        duration = int(duration * self.config.duration_multiplier)
        
        # Create concurrency tester
        tester = ConcurrencyTester(
            max_threads=max_threads,
            duration_seconds=duration,
            output_dir=os.path.join(self.config.output_dir, "concurrency")
        )
        
        # Run tests
        results = tester.run_tests()
        
        return {
            "component": "concurrency",
            "success": results.success,
            "results": results,
            "issues": results.issues_detected
        }
    
    def _run_long_running_tests(self) -> Dict[str, Any]:
        """Run long-running tests.
        
        Returns:
            Results of long-running tests
        """
        # Determine duration based on suite
        if self.config.suite == StressTestSuite.QUICK:
            duration = 60  # 1 minute
        
        elif self.config.suite == StressTestSuite.STANDARD:
            duration = 300  # 5 minutes
        
        elif self.config.suite == StressTestSuite.COMPREHENSIVE:
            duration = 1800  # 30 minutes
        
        elif self.config.suite == StressTestSuite.NIGHTLY:
            duration = 3600  # 1 hour
        
        elif self.config.suite == StressTestSuite.RELEASE:
            duration = 7200  # 2 hours
        
        # Apply duration multiplier
        duration = int(duration * self.config.duration_multiplier)
        
        # Create long-running test manager
        manager = LongRunningTestManager(
            duration_seconds=duration,
            output_dir=os.path.join(self.config.output_dir, "long_running")
        )
        
        # Run tests
        results = manager.run_tests()
        
        return {
            "component": "long_running",
            "success": results.success,
            "results": results,
            "issues": results.issues_detected
        }
    
    def _run_load_tests(self) -> Dict[str, Any]:
        """Run load tests.
        
        Returns:
            Results of load tests
        """
        # Determine load pattern and duration based on suite
        if self.config.suite == StressTestSuite.QUICK:
            pattern = LoadPattern.CONSTANT
            initial_load = 5
            max_load = 10
            duration = 30
        
        elif self.config.suite == StressTestSuite.STANDARD:
            pattern = LoadPattern.STEP
            initial_load = 10
            max_load = 50
            duration = 60
        
        elif self.config.suite == StressTestSuite.COMPREHENSIVE:
            pattern = LoadPattern.RAMP
            initial_load = 10
            max_load = 100
            duration = 120
        
        elif self.config.suite == StressTestSuite.NIGHTLY:
            pattern = LoadPattern.WAVE
            initial_load = 10
            max_load = 200
            duration = 300
        
        elif self.config.suite == StressTestSuite.RELEASE:
            pattern = LoadPattern.RANDOM
            initial_load = 10
            max_load = 500
            duration = 600
        
        # Apply duration multiplier
        duration = int(duration * self.config.duration_multiplier)
        
        # Create load test configuration
        load_config = LoadTestConfig(
            pattern=pattern,
            initial_load=initial_load,
            max_load=max_load,
            duration_seconds=duration,
            output_dir=os.path.join(self.config.output_dir, "load_testing")
        )
        
        # Run load tests
        controller = LoadTestController(load_config)
        results = controller.run_test()
        controller.save_results(results)
        
        return {
            "component": "load_testing",
            "success": results.success,
            "results": results,
            "issues": [results.error_message] if results.error_message else []
        }
    
    def _run_fault_injection_tests(self) -> Dict[str, Any]:
        """Run fault injection tests.
        
        Returns:
            Results of fault injection tests
        """
        # Determine test count and fault types based on suite
        if self.config.suite == StressTestSuite.QUICK:
            test_count = 10
            fault_types = [FaultType.INVALID_INPUT, FaultType.SYNTAX_ERROR]
            frequency = 0.5
        
        elif self.config.suite == StressTestSuite.STANDARD:
            test_count = 50
            fault_types = [
                FaultType.INVALID_INPUT,
                FaultType.SYNTAX_ERROR,
                FaultType.RUNTIME_ERROR
            ]
            frequency = 0.7
        
        elif self.config.suite == StressTestSuite.COMPREHENSIVE:
            test_count = 100
            fault_types = list(FaultType)
            frequency = 0.8
        
        elif self.config.suite == StressTestSuite.NIGHTLY:
            test_count = 200
            fault_types = list(FaultType)
            frequency = 0.9
        
        elif self.config.suite == StressTestSuite.RELEASE:
            test_count = 500
            fault_types = list(FaultType)
            frequency = 1.0
        
        # Apply duration multiplier (affects test count)
        test_count = int(test_count * self.config.duration_multiplier)
        
        # Create fault injection configuration
        fault_config = FaultInjectionConfig(
            fault_types=fault_types,
            frequency=frequency,
            target_components=["parser", "interpreter", "garbage_collector"],
            seed=self.config.seed,
            output_dir=os.path.join(self.config.output_dir, "fault_injection")
        )
        
        # Run fault injection tests
        injector = FaultInjector(fault_config)
        results = injector.run_tests(test_count)
        injector.save_results(results)
        
        return {
            "component": "fault_injection",
            "success": results.recovery_success_rate >= 0.7,  # Consider successful if recovery rate is at least 70%
            "results": results,
            "issues": results.issues_detected
        }
    
    def generate_report(self, result: IntegratedStressTestResult) -> str:
        """Generate a report of the integrated stress test results.
        
        Args:
            result: Integrated stress test results
            
        Returns:
            Markdown-formatted report
        """
        # Create report header
        report = f"# Integrated Stress Test Report\n\n"
        report += f"## Configuration\n\n"
        report += f"- **Suite**: {result.config.suite.value}\n"
        report += f"- **Parallel Tests**: {result.config.parallel_tests}\n"
        report += f"- **Duration Multiplier**: {result.config.duration_multiplier}\n"
        report += f"- **Components**: {', '.join(result.component_results.keys())}\n\n"
        
        # Add summary
        report += f"## Summary\n\n"
        report += f"- **Overall Success**: {'Yes' if result.overall_success else 'No'}\n"
        report += f"- **Duration**: {result.end_time - result.start_time:.2f} seconds\n"
        report += f"- **Components Tested**: {len(result.component_results)}\n"
        report += f"- **Components Succeeded**: {sum(1 for r in result.component_results.values() if r.get('success', False))}\n"
        report += f"- **Components Failed**: {sum(1 for r in result.component_results.values() if not r.get('success', False))}\n\n"
        
        # Add component results
        report += f"## Component Results\n\n"
        report += f"| Component | Success | Issues |\n"
        report += f"|-----------|---------|--------|\n"
        
        for component, component_result in result.component_results.items():
            success = component_result.get("success", False)
            issues = component_result.get("issues", [])
            issue_count = len(issues)
            
            report += f"| {component} | {'Yes' if success else 'No'} | {issue_count} |\n"
        
        # Add issues
        if result.issues_detected:
            report += f"\n## Issues Detected\n\n"
            for issue in result.issues_detected:
                report += f"- {issue}\n"
        
        # Add component-specific issues
        for component, component_result in result.component_results.items():
            issues = component_result.get("issues", [])
            if issues:
                report += f"\n### {component} Issues\n\n"
                for issue in issues[:10]:  # Show at most 10 issues per component
                    report += f"- {issue}\n"
                
                if len(issues) > 10:
                    report += f"- ... ({len(issues) - 10} more issues)\n"
        
        # Add timestamp
        report += f"\n\nGenerated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        return report
    
    def save_results(self, result: IntegratedStressTestResult):
        """Save integrated stress test results to files.
        
        Args:
            result: Integrated stress test results
        """
        # Create timestamp
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        
        # Save report
        report_path = os.path.join(
            self.config.output_dir,
            f"integrated_stress_test_report_{result.config.suite.value}_{timestamp}.md"
        )
        with open(report_path, "w") as f:
            f.write(self.generate_report(result))
        
        # Save raw data
        data_path = os.path.join(
            self.config.output_dir,
            f"integrated_stress_test_data_{result.config.suite.value}_{timestamp}.json"
        )
        with open(data_path, "w") as f:
            # Convert result to dictionary
            result_dict = {
                "config": {
                    "suite": result.config.suite.value,
                    "parallel_tests": result.config.parallel_tests,
                    "duration_multiplier": result.config.duration_multiplier
                },
                "overall_success": result.overall_success,
                "start_time": result.start_time,
                "end_time": result.end_time,
                "issues_detected": result.issues_detected,
                "component_results": {
                    component: {
                        "success": component_result.get("success", False),
                        "issues": component_result.get("issues", [])
                    }
                    for component, component_result in result.component_results.items()
                }
            }
            json.dump(result_dict, f, indent=2)
        
        logger.info(f"Results saved to {report_path} and {data_path}")


def main():
    """Main entry point for the integrated stress test runner."""
    parser = argparse.ArgumentParser(description="Anarchy Inference Integrated Stress Testing")
    
    parser.add_argument(
        "--suite",
        choices=[s.value for s in StressTestSuite],
        default=StressTestSuite.STANDARD.value,
        help="Stress test suite to run"
    )
    
    parser.add_argument(
        "--output-dir",
        default="./stress_test_results",
        help="Directory for output files"
    )
    
    parser.add_argument(
        "--parallel",
        type=int,
        default=1,
        help="Number of tests to run in parallel"
    )
    
    parser.add_argument(
        "--include",
        nargs="+",
        help="Components to include (all if not specified)"
    )
    
    parser.add_argument(
        "--exclude",
        nargs="+",
        help="Components to exclude (none if not specified)"
    )
    
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducibility"
    )
    
    parser.add_argument(
        "--duration-multiplier",
        type=float,
        default=1.0,
        help="Multiplier for test durations"
    )
    
    args = parser.parse_args()
    
    # Create configuration
    config = IntegratedStressTestConfig(
        suite=StressTestSuite(args.suite),
        output_dir=args.output_dir,
        parallel_tests=args.parallel,
        include_components=args.include,
        exclude_components=args.exclude,
        seed=args.seed,
        duration_multiplier=args.duration_multiplier
    )
    
    # Run tests
    runner = IntegratedStressTestRunner(config)
    result = runner.run_all_tests()
    runner.save_results(result)
    
    # Print summary
    print(f"\nIntegrated Stress Test Summary:")
    print(f"- Suite: {config.suite.value}")
    print(f"- Overall Success: {'Yes' if result.overall_success else 'No'}")
    print(f"- Duration: {result.end_time - result.start_time:.2f} seconds")
    print(f"- Components Tested: {len(result.component_results)}")
    print(f"- Components Succeeded: {sum(1 for r in result.component_results.values() if r.get('success', False))}")
    print(f"- Components Failed: {sum(1 for r in result.component_results.values() if not r.get('success', False))}")
    
    if result.issues_detected:
        print(f"\nIssues Detected:")
        for issue in result.issues_detected[:5]:  # Show at most 5 issues
            print(f"- {issue}")
        
        if len(result.issues_detected) > 5:
            print(f"- ... ({len(result.issues_detected) - 5} more issues)")
    
    print(f"\nDetailed report available in: {config.output_dir}")
    
    return 0 if result.overall_success else 1


if __name__ == "__main__":
    sys.exit(main())
