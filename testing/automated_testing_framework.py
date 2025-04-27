#!/usr/bin/env python3
"""
Automated Testing Framework for Anarchy Inference.

This script integrates all testing components including unit tests, integration tests,
benchmarks, stress tests, and fuzzing tests into a unified testing framework.
"""

import os
import sys
import time
import argparse
import logging
import json
import subprocess
from typing import Dict, List, Any, Optional, Tuple
from enum import Enum
from dataclasses import dataclass
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("automated_testing_framework")

class TestType(Enum):
    """Types of tests that can be run."""
    UNIT = "unit"                 # Unit tests
    INTEGRATION = "integration"   # Integration tests
    BENCHMARK = "benchmark"       # Performance benchmarks
    STRESS = "stress"             # Stress tests
    FUZZING = "fuzzing"           # Fuzzing tests
    ALL = "all"                   # All test types


@dataclass
class TestConfig:
    """Configuration for running tests."""
    test_types: List[TestType]
    output_dir: str
    parallel: int
    verbose: bool
    ci_mode: bool
    report_dir: str
    fuzzing_suite: Optional[str]
    fuzzing_duration: float


def parse_args() -> TestConfig:
    """Parse command line arguments.
    
    Returns:
        Test configuration
    """
    parser = argparse.ArgumentParser(description="Automated Testing Framework for Anarchy Inference")
    
    parser.add_argument(
        "--test-types",
        nargs="+",
        choices=[t.value for t in TestType],
        default=[TestType.UNIT.value],
        help="Types of tests to run"
    )
    
    parser.add_argument(
        "--output-dir",
        default="./test_output",
        help="Directory for output files"
    )
    
    parser.add_argument(
        "--parallel",
        type=int,
        default=os.cpu_count() or 1,
        help="Number of parallel jobs"
    )
    
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose output"
    )
    
    parser.add_argument(
        "--ci-mode",
        action="store_true",
        help="Run in CI mode (fail on any error)"
    )
    
    parser.add_argument(
        "--report-dir",
        default="./test_reports",
        help="Directory for report files"
    )
    
    parser.add_argument(
        "--fuzzing-suite",
        choices=["quick", "standard", "comprehensive", "nightly", "release"],
        default="quick",
        help="Fuzzing test suite to run (only used if fuzzing test type is selected)"
    )
    
    parser.add_argument(
        "--fuzzing-duration",
        type=float,
        default=0.1,
        help="Duration multiplier for fuzzing tests (only used if fuzzing test type is selected)"
    )
    
    args = parser.parse_args()
    
    # Convert string values to enum values
    test_types = []
    if TestType.ALL.value in args.test_types:
        test_types = [t for t in TestType if t != TestType.ALL]
    else:
        test_types = [TestType(t) for t in args.test_types]
    
    # Create and return the configuration
    return TestConfig(
        test_types=test_types,
        output_dir=args.output_dir,
        parallel=args.parallel,
        verbose=args.verbose,
        ci_mode=args.ci_mode,
        report_dir=args.report_dir,
        fuzzing_suite=args.fuzzing_suite,
        fuzzing_duration=args.fuzzing_duration
    )


def run_tests(config: TestConfig) -> Dict[str, Any]:
    """Run tests.
    
    Args:
        config: Test configuration
        
    Returns:
        Results of the tests
    """
    # Create output directory
    os.makedirs(config.output_dir, exist_ok=True)
    
    # Create report directory
    os.makedirs(config.report_dir, exist_ok=True)
    
    # Log configuration
    logger.info(f"Running tests: {', '.join(t.value for t in config.test_types)}")
    logger.info(f"Parallel jobs: {config.parallel}")
    logger.info(f"Verbose: {config.verbose}")
    logger.info(f"CI mode: {config.ci_mode}")
    
    # Run each test type
    results = {}
    
    for test_type in config.test_types:
        logger.info(f"Running {test_type.value} tests...")
        
        # Create test type output directory
        test_type_output_dir = os.path.join(config.output_dir, test_type.value)
        os.makedirs(test_type_output_dir, exist_ok=True)
        
        # Run the test type
        if test_type == TestType.UNIT:
            results[test_type.value] = run_unit_tests(
                test_type_output_dir,
                config
            )
        elif test_type == TestType.INTEGRATION:
            results[test_type.value] = run_integration_tests(
                test_type_output_dir,
                config
            )
        elif test_type == TestType.BENCHMARK:
            results[test_type.value] = run_benchmark_tests(
                test_type_output_dir,
                config
            )
        elif test_type == TestType.STRESS:
            results[test_type.value] = run_stress_tests(
                test_type_output_dir,
                config
            )
        elif test_type == TestType.FUZZING:
            results[test_type.value] = run_fuzzing_tests(
                test_type_output_dir,
                config
            )
    
    # Generate overall report
    generate_overall_report(results, config)
    
    return results


def run_unit_tests(output_dir: str, config: TestConfig) -> Dict[str, Any]:
    """Run unit tests.
    
    Args:
        output_dir: Output directory
        config: Test configuration
        
    Returns:
        Results of the unit tests
    """
    logger.info("Running unit tests...")
    
    # Build command
    cmd = ["cargo", "test", "--lib", "--bins"]
    
    if config.verbose:
        cmd.append("--verbose")
    
    # Run command
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    end_time = time.time()
    
    # Parse output
    success = result.returncode == 0
    
    # Save output
    with open(os.path.join(output_dir, "unit_tests_stdout.txt"), "w") as f:
        f.write(result.stdout)
    
    with open(os.path.join(output_dir, "unit_tests_stderr.txt"), "w") as f:
        f.write(result.stderr)
    
    # Extract test counts
    total_tests = 0
    passed_tests = 0
    failed_tests = 0
    
    for line in result.stdout.split("\n"):
        if "test result:" in line:
            parts = line.split(":")
            if len(parts) > 1:
                result_parts = parts[1].strip().split(".")
                for part in result_parts:
                    if "passed" in part:
                        passed_tests = int(part.split()[0])
                    elif "failed" in part:
                        failed_tests = int(part.split()[0])
    
    total_tests = passed_tests + failed_tests
    
    # Return results
    return {
        "success": success,
        "total_tests": total_tests,
        "passed_tests": passed_tests,
        "failed_tests": failed_tests,
        "execution_time_seconds": end_time - start_time
    }


def run_integration_tests(output_dir: str, config: TestConfig) -> Dict[str, Any]:
    """Run integration tests.
    
    Args:
        output_dir: Output directory
        config: Test configuration
        
    Returns:
        Results of the integration tests
    """
    logger.info("Running integration tests...")
    
    # Build command
    cmd = ["python3", "testing/integration_tests.py"]
    
    if config.verbose:
        cmd.append("--verbose")
    
    if config.parallel > 1:
        cmd.extend(["--parallel", str(config.parallel)])
    
    cmd.extend(["--output-dir", output_dir])
    
    # Run command
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    end_time = time.time()
    
    # Parse output
    success = result.returncode == 0
    
    # Save output
    with open(os.path.join(output_dir, "integration_tests_stdout.txt"), "w") as f:
        f.write(result.stdout)
    
    with open(os.path.join(output_dir, "integration_tests_stderr.txt"), "w") as f:
        f.write(result.stderr)
    
    # Extract test counts
    total_tests = 0
    passed_tests = 0
    failed_tests = 0
    
    for line in result.stdout.split("\n"):
        if "Total tests:" in line:
            total_tests = int(line.split(":")[1].strip())
        elif "Passed tests:" in line:
            passed_tests = int(line.split(":")[1].strip())
        elif "Failed tests:" in line:
            failed_tests = int(line.split(":")[1].strip())
    
    # Return results
    return {
        "success": success,
        "total_tests": total_tests,
        "passed_tests": passed_tests,
        "failed_tests": failed_tests,
        "execution_time_seconds": end_time - start_time
    }


def run_benchmark_tests(output_dir: str, config: TestConfig) -> Dict[str, Any]:
    """Run benchmark tests.
    
    Args:
        output_dir: Output directory
        config: Test configuration
        
    Returns:
        Results of the benchmark tests
    """
    logger.info("Running benchmark tests...")
    
    # Build command
    cmd = ["python3", "testing/run_benchmarks.py"]
    
    if config.verbose:
        cmd.append("--verbose")
    
    if config.parallel > 1:
        cmd.extend(["--parallel", str(config.parallel)])
    
    cmd.extend(["--output-dir", output_dir])
    
    # Run command
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    end_time = time.time()
    
    # Parse output
    success = result.returncode == 0
    
    # Save output
    with open(os.path.join(output_dir, "benchmark_tests_stdout.txt"), "w") as f:
        f.write(result.stdout)
    
    with open(os.path.join(output_dir, "benchmark_tests_stderr.txt"), "w") as f:
        f.write(result.stderr)
    
    # Extract benchmark counts
    total_benchmarks = 0
    passed_benchmarks = 0
    failed_benchmarks = 0
    
    for line in result.stdout.split("\n"):
        if "Total benchmarks:" in line:
            total_benchmarks = int(line.split(":")[1].strip())
        elif "Passed benchmarks:" in line:
            passed_benchmarks = int(line.split(":")[1].strip())
        elif "Failed benchmarks:" in line:
            failed_benchmarks = int(line.split(":")[1].strip())
    
    # Return results
    return {
        "success": success,
        "total_benchmarks": total_benchmarks,
        "passed_benchmarks": passed_benchmarks,
        "failed_benchmarks": failed_benchmarks,
        "execution_time_seconds": end_time - start_time
    }


def run_stress_tests(output_dir: str, config: TestConfig) -> Dict[str, Any]:
    """Run stress tests.
    
    Args:
        output_dir: Output directory
        config: Test configuration
        
    Returns:
        Results of the stress tests
    """
    logger.info("Running stress tests...")
    
    # Build command
    cmd = ["python3", "testing/run_stress_tests.py"]
    
    if config.verbose:
        cmd.append("--verbose")
    
    if config.parallel > 1:
        cmd.extend(["--parallel", str(config.parallel)])
    
    cmd.extend(["--output-dir", output_dir])
    
    # Run command
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    end_time = time.time()
    
    # Parse output
    success = result.returncode == 0
    
    # Save output
    with open(os.path.join(output_dir, "stress_tests_stdout.txt"), "w") as f:
        f.write(result.stdout)
    
    with open(os.path.join(output_dir, "stress_tests_stderr.txt"), "w") as f:
        f.write(result.stderr)
    
    # Extract test counts
    total_tests = 0
    passed_tests = 0
    failed_tests = 0
    
    for line in result.stdout.split("\n"):
        if "Total tests:" in line:
            total_tests = int(line.split(":")[1].strip())
        elif "Passed tests:" in line:
            passed_tests = int(line.split(":")[1].strip())
        elif "Failed tests:" in line:
            failed_tests = int(line.split(":")[1].strip())
    
    # Return results
    return {
        "success": success,
        "total_tests": total_tests,
        "passed_tests": passed_tests,
        "failed_tests": failed_tests,
        "execution_time_seconds": end_time - start_time
    }


def run_fuzzing_tests(output_dir: str, config: TestConfig) -> Dict[str, Any]:
    """Run fuzzing tests.
    
    Args:
        output_dir: Output directory
        config: Test configuration
        
    Returns:
        Results of the fuzzing tests
    """
    logger.info("Running fuzzing tests...")
    
    # Build command
    cmd = ["python3", "testing/fuzzing/run_fuzzing.py"]
    
    # Add suite
    if config.fuzzing_suite:
        cmd.extend(["--suite", config.fuzzing_suite])
    
    # Add duration multiplier
    if config.fuzzing_duration:
        cmd.extend(["--duration-multiplier", str(config.fuzzing_duration)])
    
    if config.parallel > 1:
        cmd.extend(["--parallel", str(config.parallel)])
    
    cmd.extend(["--output-dir", output_dir])
    cmd.extend(["--report-dir", os.path.join(config.report_dir, "fuzzing")])
    
    # Run command
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True)
    end_time = time.time()
    
    # Parse output
    success = result.returncode == 0
    
    # Save output
    with open(os.path.join(output_dir, "fuzzing_tests_stdout.txt"), "w") as f:
        f.write(result.stdout)
    
    with open(os.path.join(output_dir, "fuzzing_tests_stderr.txt"), "w") as f:
        f.write(result.stderr)
    
    # Extract test counts
    total_tests = 0
    unique_crashes = 0
    unique_behaviors = 0
    
    for line in result.stdout.split("\n"):
        if "Total Tests:" in line:
            total_tests = int(line.split(":")[1].strip())
        elif "Unique Crashes:" in line:
            unique_crashes = int(line.split(":")[1].strip())
        elif "Unique Interesting Behaviors:" in line:
            unique_behaviors = int(line.split(":")[1].strip())
    
    # Return results
    return {
        "success": success,
        "total_tests": total_tests,
        "unique_crashes": unique_crashes,
        "unique_behaviors": unique_behaviors,
        "execution_time_seconds": end_time - start_time
    }


def generate_overall_report(results: Dict[str, Dict[str, Any]], config: TestConfig):
    """Generate an overall report.
    
    Args:
        results: Results of all test types
        config: Test configuration
    """
    # Create report data
    report_data = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "test_types": [t.value for t in config.test_types],
        "results": results,
        "summary": {
            "success": all(r.get("success", False) for r in results.values()),
            "total_execution_time_seconds": sum(r.get("execution_time_seconds", 0) for r in results.values())
        }
    }
    
    # Save report data as JSON
    report_path = os.path.join(config.report_dir, f"test_report_{int(time.time())}.json")
    with open(report_path, "w") as f:
        json.dump(report_data, f, indent=2)
    
    # Generate markdown report
    markdown_report = generate_markdown_report(report_data)
    
    # Save markdown report
    markdown_path = os.path.join(config.report_dir, f"test_report_{int(time.time())}.md")
    with open(markdown_path, "w") as f:
        f.write(markdown_report)
    
    logger.info(f"Overall report saved to {report_path} and {markdown_path}")


def generate_markdown_report(report_data: Dict[str, Any]) -> str:
    """Generate a markdown report.
    
    Args:
        report_data: Report data
        
    Returns:
        Markdown report
    """
    # Create report header
    report = f"# Anarchy Inference Test Report\n\n"
    report += f"## Configuration\n\n"
    report += f"- **Timestamp**: {report_data['timestamp']}\n"
    report += f"- **Test Types**: {', '.join(report_data['test_types'])}\n"
    report += f"- **Overall Success**: {'✅ Yes' if report_data['summary']['success'] else '❌ No'}\n"
    report += f"- **Total Execution Time**: {report_data['summary']['total_execution_time_seconds']:.2f} seconds\n\n"
    
    # Add test type results
    for test_type, result in report_data["results"].items():
        report += f"## {test_type.capitalize()} Tests\n\n"
        report += f"- **Success**: {'✅ Yes' if result.get('success', False) else '❌ No'}\n"
        report += f"- **Execution Time**: {result.get('execution_time_seconds', 0):.2f} seconds\n"
        
        if test_type == "unit" or test_type == "integration" or test_type == "stress":
            report += f"- **Total Tests**: {result.get('total_tests', 0)}\n"
            report += f"- **Passed Tests**: {result.get('passed_tests', 0)}\n"
            report += f"- **Failed Tests**: {result.get('failed_tests', 0)}\n"
        elif test_type == "benchmark":
            report += f"- **Total Benchmarks**: {result.get('total_benchmarks', 0)}\n"
            report += f"- **Passed Benchmarks**: {result.get('passed_benchmarks', 0)}\n"
            report += f"- **Failed Benchmarks**: {result.get('failed_benchmarks', 0)}\n"
        elif test_type == "fuzzing":
            report += f"- **Total Tests**: {result.get('total_tests', 0)}\n"
            report += f"- **Unique Crashes**: {result.get('unique_crashes', 0)}\n"
            report += f"- **Unique Interesting Behaviors**: {result.get('unique_behaviors', 0)}\n"
        
        report += f"\n"
    
    return report


def main():
    """Main entry point for running tests."""
    try:
        # Parse command line arguments
        config = parse_args()
        
        # Run tests
        results = run_tests(config)
        
        # Print summary
        print(f"\nTest Summary:")
        print(f"- Overall Success: {'✅ Yes' if all(r.get('success', False) for r in results.values()) else '❌ No'}")
        print(f"- Total Execution Time: {sum(r.get('execution_time_seconds', 0) for r in results.values()):.2f} seconds")
        
        for test_type, result in results.items():
            print(f"\n{test_type.capitalize()} Tests:")
            print(f"- Success: {'✅ Yes' if result.get('success', False) else '❌ No'}")
            print(f"- Execution Time: {result.get('execution_time_seconds', 0):.2f} seconds")
            
            if test_type == "unit" or test_type == "integration" or test_type == "stress":
                print(f"- Total Tests: {result.get('total_tests', 0)}")
                print(f"- Passed Tests: {result.get('passed_tests', 0)}")
                print(f"- Failed Tests: {result.get('failed_tests', 0)}")
            elif test_type == "benchmark":
                print(f"- Total Benchmarks: {result.get('total_benchmarks', 0)}")
                print(f"- Passed Benchmarks: {result.get('passed_benchmarks', 0)}")
                print(f"- Failed Benchmarks: {result.get('failed_benchmarks', 0)}")
            elif test_type == "fuzzing":
                print(f"- Total Tests: {result.get('total_tests', 0)}")
                print(f"- Unique Crashes: {result.get('unique_crashes', 0)}")
                print(f"- Unique Interesting Behaviors: {result.get('unique_behaviors', 0)}")
        
        print(f"\nDetailed reports available in: {config.report_dir}")
        
        # Return success if all tests passed
        if config.ci_mode:
            return 0 if all(r.get("success", False) for r in results.values()) else 1
        else:
            return 0
    
    except Exception as e:
        logger.error(f"Error running tests: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
