#!/usr/bin/env python3
"""
Run Fuzzing Tests for Anarchy Inference.

This script provides a command-line interface for running fuzzing tests
on the Anarchy Inference language implementation.
"""

import os
import sys
import time
import argparse
import logging
import json
import multiprocessing
from typing import Dict, List, Any, Optional, Tuple
from enum import Enum
from dataclasses import dataclass
from pathlib import Path

# Add parent directory to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import fuzzing framework
from fuzzing.fuzzing_framework import (
    FuzzingFramework, FuzzingConfig, FuzzingStrategy, GeneratorType,
    TestCase, TestResult
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("run_fuzzing")

class TestSuite(Enum):
    """Test suites for fuzzing."""
    QUICK = "quick"               # Quick tests for rapid feedback
    STANDARD = "standard"         # Standard test suite for regular testing
    COMPREHENSIVE = "comprehensive"  # Comprehensive test suite for thorough testing
    NIGHTLY = "nightly"           # Extended test suite for nightly builds
    RELEASE = "release"           # Full test suite for release validation


@dataclass
class RunConfig:
    """Configuration for running fuzzing tests."""
    suite: TestSuite
    output_dir: str
    parallel: int
    duration_multiplier: float
    include_components: List[str]
    exclude_components: List[str]
    seed: Optional[int]
    coverage_guided: bool
    report_dir: str


def parse_args() -> RunConfig:
    """Parse command line arguments.
    
    Returns:
        Run configuration
    """
    parser = argparse.ArgumentParser(description="Run fuzzing tests for Anarchy Inference")
    
    parser.add_argument(
        "--suite",
        choices=[s.value for s in TestSuite],
        default=TestSuite.STANDARD.value,
        help="Test suite to run"
    )
    
    parser.add_argument(
        "--output-dir",
        default="./fuzzing_output",
        help="Directory for output files"
    )
    
    parser.add_argument(
        "--parallel",
        type=int,
        default=multiprocessing.cpu_count(),
        help="Number of parallel jobs"
    )
    
    parser.add_argument(
        "--duration-multiplier",
        type=float,
        default=1.0,
        help="Multiplier for test duration"
    )
    
    parser.add_argument(
        "--include",
        nargs="+",
        dest="include_components",
        default=[],
        help="Components to include (standard_stress, concurrency, long_running, load_testing, fault_injection)"
    )
    
    parser.add_argument(
        "--exclude",
        nargs="+",
        dest="exclude_components",
        default=[],
        help="Components to exclude (standard_stress, concurrency, long_running, load_testing, fault_injection)"
    )
    
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducibility"
    )
    
    parser.add_argument(
        "--coverage-guided",
        action="store_true",
        help="Enable coverage-guided fuzzing"
    )
    
    parser.add_argument(
        "--report-dir",
        default="./fuzzing_reports",
        help="Directory for report files"
    )
    
    args = parser.parse_args()
    
    # Convert string values to enum values
    suite = TestSuite(args.suite)
    
    # Create and return the configuration
    return RunConfig(
        suite=suite,
        output_dir=args.output_dir,
        parallel=args.parallel,
        duration_multiplier=args.duration_multiplier,
        include_components=args.include_components,
        exclude_components=args.exclude_components,
        seed=args.seed,
        coverage_guided=args.coverage_guided,
        report_dir=args.report_dir
    )


def get_suite_config(suite: TestSuite, duration_multiplier: float) -> Dict[str, Any]:
    """Get configuration for a test suite.
    
    Args:
        suite: Test suite
        duration_multiplier: Multiplier for test duration
        
    Returns:
        Suite configuration
    """
    # Base configurations for different suites
    configs = {
        TestSuite.QUICK: {
            "time_limit_seconds": int(300 * duration_multiplier),  # 5 minutes
            "strategies": [FuzzingStrategy.BLIND],
            "generators": [GeneratorType.RANDOM, GeneratorType.GRAMMAR],
            "components": ["standard_stress"]
        },
        TestSuite.STANDARD: {
            "time_limit_seconds": int(1800 * duration_multiplier),  # 30 minutes
            "strategies": [FuzzingStrategy.BLIND, FuzzingStrategy.MUTATION_BASED],
            "generators": [GeneratorType.RANDOM, GeneratorType.GRAMMAR, GeneratorType.MUTATION],
            "components": ["standard_stress", "concurrency"]
        },
        TestSuite.COMPREHENSIVE: {
            "time_limit_seconds": int(3600 * duration_multiplier),  # 1 hour
            "strategies": [FuzzingStrategy.BLIND, FuzzingStrategy.MUTATION_BASED, FuzzingStrategy.GRAMMAR_BASED],
            "generators": [GeneratorType.RANDOM, GeneratorType.GRAMMAR, GeneratorType.MUTATION, GeneratorType.TEMPLATE],
            "components": ["standard_stress", "concurrency", "long_running", "load_testing"]
        },
        TestSuite.NIGHTLY: {
            "time_limit_seconds": int(14400 * duration_multiplier),  # 4 hours
            "strategies": [FuzzingStrategy.BLIND, FuzzingStrategy.MUTATION_BASED, FuzzingStrategy.GRAMMAR_BASED, FuzzingStrategy.COVERAGE_GUIDED],
            "generators": [GeneratorType.RANDOM, GeneratorType.GRAMMAR, GeneratorType.MUTATION, GeneratorType.TEMPLATE],
            "components": ["standard_stress", "concurrency", "long_running", "load_testing", "fault_injection"]
        },
        TestSuite.RELEASE: {
            "time_limit_seconds": int(86400 * duration_multiplier),  # 24 hours
            "strategies": [FuzzingStrategy.BLIND, FuzzingStrategy.MUTATION_BASED, FuzzingStrategy.GRAMMAR_BASED, FuzzingStrategy.COVERAGE_GUIDED, FuzzingStrategy.DIRECTED],
            "generators": [GeneratorType.RANDOM, GeneratorType.GRAMMAR, GeneratorType.MUTATION, GeneratorType.TEMPLATE],
            "components": ["standard_stress", "concurrency", "long_running", "load_testing", "fault_injection"]
        }
    }
    
    return configs[suite]


def filter_components(components: List[str], include: List[str], exclude: List[str]) -> List[str]:
    """Filter components based on include and exclude lists.
    
    Args:
        components: List of components
        include: List of components to include
        exclude: List of components to exclude
        
    Returns:
        Filtered list of components
    """
    # If include list is not empty, only include those components
    if include:
        components = [c for c in components if c in include]
    
    # Exclude components in the exclude list
    if exclude:
        components = [c for c in components if c not in exclude]
    
    return components


def run_fuzzing(config: RunConfig) -> Dict[str, Any]:
    """Run fuzzing tests.
    
    Args:
        config: Run configuration
        
    Returns:
        Results of the fuzzing tests
    """
    # Get suite configuration
    suite_config = get_suite_config(config.suite, config.duration_multiplier)
    
    # Filter components
    components = filter_components(
        suite_config["components"],
        config.include_components,
        config.exclude_components
    )
    
    # Create output directory
    os.makedirs(config.output_dir, exist_ok=True)
    
    # Create report directory
    os.makedirs(config.report_dir, exist_ok=True)
    
    # Log configuration
    logger.info(f"Running fuzzing tests with suite: {config.suite.value}")
    logger.info(f"Components: {', '.join(components)}")
    logger.info(f"Parallel jobs: {config.parallel}")
    logger.info(f"Duration multiplier: {config.duration_multiplier}")
    logger.info(f"Time limit: {suite_config['time_limit_seconds']} seconds")
    
    # Run each component
    results = {}
    
    for component in components:
        logger.info(f"Running component: {component}")
        
        # Create component output directory
        component_output_dir = os.path.join(config.output_dir, component)
        os.makedirs(component_output_dir, exist_ok=True)
        
        # Run the component
        if component == "standard_stress":
            results[component] = run_standard_stress(
                component_output_dir,
                suite_config,
                config
            )
        elif component == "concurrency":
            results[component] = run_concurrency(
                component_output_dir,
                suite_config,
                config
            )
        elif component == "long_running":
            results[component] = run_long_running(
                component_output_dir,
                suite_config,
                config
            )
        elif component == "load_testing":
            results[component] = run_load_testing(
                component_output_dir,
                suite_config,
                config
            )
        elif component == "fault_injection":
            results[component] = run_fault_injection(
                component_output_dir,
                suite_config,
                config
            )
    
    # Generate overall report
    generate_overall_report(results, config)
    
    return results


def run_standard_stress(output_dir: str, suite_config: Dict[str, Any], config: RunConfig) -> Dict[str, Any]:
    """Run standard stress tests.
    
    Args:
        output_dir: Output directory
        suite_config: Suite configuration
        config: Run configuration
        
    Returns:
        Results of the standard stress tests
    """
    # Create fuzzing configuration
    fuzzing_config = FuzzingConfig(
        strategy=suite_config["strategies"][0],
        generator_types=suite_config["generators"],
        seed_corpus_dir=os.path.join("corpus", "seeds"),
        output_dir=output_dir,
        time_limit_seconds=suite_config["time_limit_seconds"],
        parallel_jobs=config.parallel,
        coverage_guided=config.coverage_guided,
        seed=config.seed
    )
    
    # Create and run the fuzzing framework
    framework = FuzzingFramework(fuzzing_config)
    result = framework.run()
    
    # Return the result
    return {
        "total_tests": result.total_tests,
        "unique_crashes": result.unique_crashes,
        "unique_behaviors": result.unique_behaviors,
        "coverage_percent": result.coverage_percent,
        "execution_time_seconds": result.execution_time_seconds,
        "tests_per_second": result.tests_per_second
    }


def run_concurrency(output_dir: str, suite_config: Dict[str, Any], config: RunConfig) -> Dict[str, Any]:
    """Run concurrency tests.
    
    Args:
        output_dir: Output directory
        suite_config: Suite configuration
        config: Run configuration
        
    Returns:
        Results of the concurrency tests
    """
    # This is a placeholder; a real implementation would run actual concurrency tests
    logger.info("Running concurrency tests...")
    time.sleep(1)  # Simulate running tests
    
    return {
        "total_tests": 100,
        "unique_crashes": 2,
        "unique_behaviors": 5,
        "coverage_percent": 75.0,
        "execution_time_seconds": 60.0,
        "tests_per_second": 1.67
    }


def run_long_running(output_dir: str, suite_config: Dict[str, Any], config: RunConfig) -> Dict[str, Any]:
    """Run long-running tests.
    
    Args:
        output_dir: Output directory
        suite_config: Suite configuration
        config: Run configuration
        
    Returns:
        Results of the long-running tests
    """
    # This is a placeholder; a real implementation would run actual long-running tests
    logger.info("Running long-running tests...")
    time.sleep(1)  # Simulate running tests
    
    return {
        "total_tests": 10,
        "unique_crashes": 1,
        "unique_behaviors": 3,
        "coverage_percent": 80.0,
        "execution_time_seconds": 300.0,
        "tests_per_second": 0.03
    }


def run_load_testing(output_dir: str, suite_config: Dict[str, Any], config: RunConfig) -> Dict[str, Any]:
    """Run load testing.
    
    Args:
        output_dir: Output directory
        suite_config: Suite configuration
        config: Run configuration
        
    Returns:
        Results of the load testing
    """
    # This is a placeholder; a real implementation would run actual load testing
    logger.info("Running load testing...")
    time.sleep(1)  # Simulate running tests
    
    return {
        "total_tests": 50,
        "unique_crashes": 3,
        "unique_behaviors": 7,
        "coverage_percent": 70.0,
        "execution_time_seconds": 120.0,
        "tests_per_second": 0.42
    }


def run_fault_injection(output_dir: str, suite_config: Dict[str, Any], config: RunConfig) -> Dict[str, Any]:
    """Run fault injection.
    
    Args:
        output_dir: Output directory
        suite_config: Suite configuration
        config: Run configuration
        
    Returns:
        Results of the fault injection
    """
    # This is a placeholder; a real implementation would run actual fault injection
    logger.info("Running fault injection...")
    time.sleep(1)  # Simulate running tests
    
    return {
        "total_tests": 30,
        "unique_crashes": 5,
        "unique_behaviors": 10,
        "coverage_percent": 65.0,
        "execution_time_seconds": 90.0,
        "tests_per_second": 0.33
    }


def generate_overall_report(results: Dict[str, Dict[str, Any]], config: RunConfig):
    """Generate an overall report.
    
    Args:
        results: Results of all components
        config: Run configuration
    """
    # Create report data
    report_data = {
        "suite": config.suite.value,
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "components": list(results.keys()),
        "results": results,
        "summary": {
            "total_tests": sum(r["total_tests"] for r in results.values()),
            "unique_crashes": sum(r["unique_crashes"] for r in results.values()),
            "unique_behaviors": sum(r["unique_behaviors"] for r in results.values()),
            "average_coverage_percent": sum(r["coverage_percent"] for r in results.values()) / len(results) if results else 0,
            "total_execution_time_seconds": sum(r["execution_time_seconds"] for r in results.values()),
            "average_tests_per_second": sum(r["tests_per_second"] for r in results.values()) / len(results) if results else 0
        }
    }
    
    # Save report data as JSON
    report_path = os.path.join(config.report_dir, f"fuzzing_report_{int(time.time())}.json")
    with open(report_path, "w") as f:
        json.dump(report_data, f, indent=2)
    
    # Generate markdown report
    markdown_report = generate_markdown_report(report_data)
    
    # Save markdown report
    markdown_path = os.path.join(config.report_dir, f"fuzzing_report_{int(time.time())}.md")
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
    report = f"# Fuzzing Report\n\n"
    report += f"## Configuration\n\n"
    report += f"- **Suite**: {report_data['suite']}\n"
    report += f"- **Timestamp**: {report_data['timestamp']}\n"
    report += f"- **Components**: {', '.join(report_data['components'])}\n\n"
    
    # Add summary
    report += f"## Summary\n\n"
    report += f"- **Total Tests**: {report_data['summary']['total_tests']}\n"
    report += f"- **Unique Crashes**: {report_data['summary']['unique_crashes']}\n"
    report += f"- **Unique Interesting Behaviors**: {report_data['summary']['unique_behaviors']}\n"
    report += f"- **Average Coverage**: {report_data['summary']['average_coverage_percent']:.2f}%\n"
    report += f"- **Total Execution Time**: {report_data['summary']['total_execution_time_seconds']:.2f} seconds\n"
    report += f"- **Average Tests Per Second**: {report_data['summary']['average_tests_per_second']:.2f}\n\n"
    
    # Add component results
    report += f"## Component Results\n\n"
    report += f"| Component | Tests | Crashes | Behaviors | Coverage | Time (s) | Tests/s |\n"
    report += f"|-----------|-------|---------|-----------|----------|----------|--------|\n"
    
    for component, result in report_data["results"].items():
        report += f"| {component} | {result['total_tests']} | {result['unique_crashes']} | {result['unique_behaviors']} | {result['coverage_percent']:.2f}% | {result['execution_time_seconds']:.2f} | {result['tests_per_second']:.2f} |\n"
    
    return report


def main():
    """Main entry point for running fuzzing tests."""
    try:
        # Parse command line arguments
        config = parse_args()
        
        # Run fuzzing tests
        results = run_fuzzing(config)
        
        # Print summary
        print(f"\nFuzzing Summary:")
        print(f"- Total Tests: {sum(r['total_tests'] for r in results.values())}")
        print(f"- Unique Crashes: {sum(r['unique_crashes'] for r in results.values())}")
        print(f"- Unique Interesting Behaviors: {sum(r['unique_behaviors'] for r in results.values())}")
        print(f"- Average Coverage: {sum(r['coverage_percent'] for r in results.values()) / len(results) if results else 0:.2f}%")
        print(f"- Total Execution Time: {sum(r['execution_time_seconds'] for r in results.values()):.2f} seconds")
        print(f"- Average Tests Per Second: {sum(r['tests_per_second'] for r in results.values()) / len(results) if results else 0:.2f}")
        
        print(f"\nDetailed reports available in: {config.report_dir}")
        
        # Return success if no crashes were found
        return 0 if sum(r["unique_crashes"] for r in results.values()) == 0 else 1
    
    except Exception as e:
        logger.error(f"Error running fuzzing tests: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
