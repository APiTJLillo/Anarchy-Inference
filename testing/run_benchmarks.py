#!/usr/bin/env python3
"""
Benchmark Runner for Anarchy Inference

This script runs the benchmark suite for Anarchy Inference, collecting and reporting
performance metrics across different scenarios.
"""

import os
import sys
import argparse
import subprocess
import time
import json
from typing import Dict, List, Any, Optional

# Add the parent directory to the path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import benchmark modules
from performance_benchmarking.performance_benchmarking import BenchmarkSuite, BenchmarkResult
from performance_benchmarking.benchmark_config import ConfigurationManager
from performance_benchmarking.cross_language_benchmarking import CrossLanguageBenchmark, AnarchyAdapter, PythonAdapter, JavaScriptAdapter
from performance_benchmarking.ci_integration import BenchmarkDatabase, RegressionDetector, ContinuousIntegrationRunner
from performance_benchmarking.visualization import BenchmarkVisualizer

def run_anarchy_benchmark(benchmark_file: str, iterations: int = 5) -> BenchmarkResult:
    """Run a benchmark file using the Anarchy Inference interpreter.
    
    Args:
        benchmark_file: Path to the benchmark file
        iterations: Number of iterations to run
        
    Returns:
        Benchmark result
    """
    print(f"Running benchmark: {os.path.basename(benchmark_file)}")
    
    # Create the adapter
    adapter = AnarchyAdapter()
    
    # Read the benchmark file
    with open(benchmark_file, 'r') as f:
        code = f.read()
    
    # Run the benchmark
    result = adapter.run_benchmark(code)
    
    # Update the result name
    result.name = os.path.basename(benchmark_file).replace('.anarchy', '')
    
    return result

def run_benchmark_category(category_dir: str, iterations: int = 5) -> Dict[str, BenchmarkResult]:
    """Run all benchmarks in a category.
    
    Args:
        category_dir: Path to the category directory
        iterations: Number of iterations to run
        
    Returns:
        Dictionary mapping benchmark names to results
    """
    results = {}
    
    # Find all benchmark files in the category directory
    for filename in os.listdir(category_dir):
        if filename.endswith('.anarchy'):
            benchmark_file = os.path.join(category_dir, filename)
            result = run_anarchy_benchmark(benchmark_file, iterations)
            results[result.name] = result
    
    return results

def run_all_benchmarks(benchmark_dir: str, iterations: int = 5) -> Dict[str, Dict[str, BenchmarkResult]]:
    """Run all benchmarks in all categories.
    
    Args:
        benchmark_dir: Path to the benchmark directory
        iterations: Number of iterations to run
        
    Returns:
        Dictionary mapping category names to dictionaries mapping benchmark names to results
    """
    results = {}
    
    # Find all category directories
    for category in os.listdir(benchmark_dir):
        category_dir = os.path.join(benchmark_dir, category)
        if os.path.isdir(category_dir):
            category_results = run_benchmark_category(category_dir, iterations)
            if category_results:
                results[category] = category_results
    
    return results

def run_cross_language_benchmarks(benchmark_dir: str) -> Dict[str, Dict[str, Dict[str, BenchmarkResult]]]:
    """Run cross-language benchmarks.
    
    Args:
        benchmark_dir: Path to the benchmark directory
        
    Returns:
        Dictionary mapping category names to dictionaries mapping benchmark names to
        dictionaries mapping language names to benchmark results
    """
    # Create language adapters
    anarchy = AnarchyAdapter()
    python = PythonAdapter()
    javascript = JavaScriptAdapter()
    
    # Create cross-language benchmark
    benchmark = CrossLanguageBenchmark([anarchy, python, javascript])
    
    # Define benchmark implementations
    # For simplicity, we'll just use the fibonacci benchmark as an example
    fibonacci_implementations = {
        "Anarchy Inference": {
            "code": """
ι fibonacci(n) ⟼ {
    if (n <= 1) {
        ⟼ n;
    }
    ⟼ fibonacci(n - 1) + fibonacci(n - 2);
}

⟼ fibonacci(20);
"""
        },
        "Python": {
            "code": """
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

result = fibonacci(20)
"""
        },
        "JavaScript": {
            "code": """
function fibonacci(n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const result = fibonacci(20);
"""
        }
    }
    
    # Run the benchmark
    results = {
        "algorithms": {
            "fibonacci": benchmark.run_benchmark("fibonacci", fibonacci_implementations)
        }
    }
    
    return results

def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="Run benchmarks for Anarchy Inference")
    parser.add_argument("--profile", help="Benchmark profile to use")
    parser.add_argument("--output-dir", help="Directory to save reports")
    parser.add_argument("--cross-language", action="store_true", help="Run cross-language benchmarks")
    parser.add_argument("--ci", action="store_true", help="Run in CI mode")
    parser.add_argument("--visualize", action="store_true", help="Generate visualizations")
    
    args = parser.parse_args()
    
    # Set up paths
    script_dir = os.path.dirname(os.path.abspath(__file__))
    benchmark_dir = os.path.join(script_dir, "benchmarks")
    output_dir = args.output_dir or os.path.join(script_dir, "benchmark_reports")
    
    # Create output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    
    # Load configuration
    config_manager = ConfigurationManager()
    if not args.profile:
        # Create default profiles if they don't exist
        config_manager.create_default_profiles()
    
    profile = config_manager.get_profile(args.profile)
    
    if not profile:
        print(f"Profile '{args.profile}' not found, using default")
        profile = config_manager.get_profile()
    
    print(f"Using profile: {profile.name}")
    print(f"Description: {profile.description}")
    print(f"Iterations: {profile.iterations}")
    print(f"Categories: {', '.join(profile.categories.keys())}")
    
    # Run benchmarks
    all_results = {}
    
    for category_name, category in profile.categories.items():
        if not category.enabled:
            continue
        
        category_dir = os.path.join(benchmark_dir, category_name)
        if os.path.isdir(category_dir):
            print(f"\nRunning {category_name} benchmarks...")
            category_results = run_benchmark_category(category_dir, profile.iterations)
            if category_results:
                all_results[category_name] = category_results
    
    # Create benchmark suite
    suite = BenchmarkSuite()
    
    for category_results in all_results.values():
        for name, result in category_results.items():
            suite.add_result(result)
    
    # Run cross-language benchmarks if requested
    cross_language_results = None
    if args.cross_language:
        print("\nRunning cross-language benchmarks...")
        cross_language_results = run_cross_language_benchmarks(benchmark_dir)
        
        # Compare results
        for category_name, category_results in cross_language_results.items():
            for benchmark_name, language_results in category_results.items():
                comparison = benchmark.compare_results({benchmark_name: language_results})
                print(f"\nComparison for {benchmark_name}:")
                for language_name, comp in comparison[benchmark_name].items():
                    print(f"  {language_name}:")
                    print(f"    Time diff: {comp['time_diff_pct']:.2f}%")
                    print(f"    Token diff: {comp['token_diff_pct']:.2f}%")
    
    # Save results
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    results_file = os.path.join(output_dir, f"benchmark_results_{timestamp}.json")
    
    with open(results_file, 'w') as f:
        json.dump(suite.to_dict(), f, indent=2)
    
    print(f"\nResults saved to {results_file}")
    
    # Run in CI mode if requested
    if args.ci:
        print("\nRunning in CI mode...")
        
        # Create database and CI runner
        db = BenchmarkDatabase()
        detector = RegressionDetector(db)
        runner = ContinuousIntegrationRunner(db, detector)
        
        # Run benchmarks in CI mode
        run_id, alerts = runner.run_ci_benchmarks(suite)
        
        # Generate CI report
        report_path = runner.generate_ci_report(run_id, alerts, output_dir)
        
        print(f"CI report generated: {report_path}")
        
        if alerts:
            print("\nRegression alerts:")
            for alert in alerts:
                print(f"  {alert['benchmark_name']} - {alert['metric']}: +{alert['percent_change']:.2f}% ({alert['severity'].upper()})")
    
    # Generate visualizations if requested
    if args.visualize:
        print("\nGenerating visualizations...")
        
        # Create visualizer
        visualizer = BenchmarkVisualizer(output_dir)
        
        # Create charts
        execution_time_chart = visualizer.create_execution_time_chart(suite.results)
        memory_usage_chart = visualizer.create_memory_usage_chart(suite.results)
        token_count_chart = visualizer.create_token_count_chart(suite.results)
        
        print(f"Execution time chart: {execution_time_chart}")
        print(f"Memory usage chart: {memory_usage_chart}")
        print(f"Token count chart: {token_count_chart}")
        
        # Create dashboard
        dashboard_path = visualizer.create_interactive_dashboard(suite)
        
        print(f"Interactive dashboard: {dashboard_path}")
        
        # Create cross-language comparison charts if available
        if cross_language_results:
            for category_name, category_results in cross_language_results.items():
                for benchmark_name, language_results in category_results.items():
                    chart_path = visualizer.create_cross_language_comparison_chart(
                        {benchmark_name: language_results},
                        "execution_time",
                        f"Execution Time Comparison for {benchmark_name}",
                        f"cross_language_{benchmark_name}_time.png"
                    )
                    print(f"Cross-language comparison chart for {benchmark_name}: {chart_path}")
    
    print("\nBenchmark run completed successfully!")

if __name__ == "__main__":
    main()
