#!/usr/bin/env python3
"""
Automated Testing Framework for Anarchy Inference

This framework provides comprehensive testing capabilities for the Anarchy Inference language,
including unit tests, integration tests, token efficiency tests, and performance benchmarks.
"""

import os
import sys
import unittest
import subprocess
import json
import time
import re
from typing import List, Dict, Any, Tuple, Optional
import argparse

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

class AnarchyTestCase(unittest.TestCase):
    """Base class for Anarchy Inference test cases with additional assertion methods."""
    
    def setUp(self):
        """Set up test environment."""
        self.interpreter = anarchy.Interpreter()
    
    def assertTokenCount(self, code: str, max_tokens: int):
        """Assert that the Anarchy Inference code uses fewer than max_tokens."""
        token_count = self.interpreter.count_tokens(code)
        self.assertLessEqual(token_count, max_tokens, 
                            f"Token count {token_count} exceeds maximum {max_tokens}")
    
    def assertTokenEfficiency(self, anarchy_code: str, python_code: str, min_reduction: float = 0.2):
        """Assert that anarchy_code is at least min_reduction more token-efficient than python_code."""
        anarchy_tokens = self.interpreter.count_tokens(anarchy_code)
        python_tokens = self.interpreter.count_tokens(python_code)
        
        if python_tokens == 0:
            self.fail("Python code has zero tokens, cannot calculate reduction")
            
        reduction = 1 - (anarchy_tokens / python_tokens)
        self.assertGreaterEqual(reduction, min_reduction, 
                               f"Token reduction {reduction:.2f} is less than minimum {min_reduction:.2f}")
    
    def assertExecutionResult(self, code: str, expected_result: Any):
        """Assert that executing the code produces the expected result."""
        result = self.interpreter.execute(code)
        self.assertEqual(result, expected_result, 
                        f"Expected {expected_result}, but got {result}")
    
    def assertExecutionTime(self, code: str, max_time_ms: int):
        """Assert that executing the code takes less than max_time_ms milliseconds."""
        start_time = time.time()
        self.interpreter.execute(code)
        execution_time = (time.time() - start_time) * 1000
        
        self.assertLessEqual(execution_time, max_time_ms, 
                            f"Execution time {execution_time:.2f}ms exceeds maximum {max_time_ms}ms")


class TestRunner:
    """Manages test discovery, execution, and reporting."""
    
    def __init__(self, test_dir: str = None):
        """Initialize the test runner with the directory containing test files."""
        self.test_dir = test_dir or os.path.join(os.path.dirname(__file__), "test_cases")
        self.results = {}
        
    def discover_tests(self) -> List[str]:
        """Discover all test files in the test directory."""
        if not os.path.exists(self.test_dir):
            os.makedirs(self.test_dir)
            print(f"Created test directory: {self.test_dir}")
            
        test_files = []
        for root, _, files in os.walk(self.test_dir):
            for file in files:
                if file.startswith("test_") and file.endswith(".py"):
                    test_files.append(os.path.join(root, file))
                    
        return test_files
    
    def run_tests(self, pattern: str = None) -> Dict[str, Any]:
        """Run all tests, optionally filtering by pattern."""
        loader = unittest.TestLoader()
        suite = unittest.TestSuite()
        
        test_files = self.discover_tests()
        if not test_files:
            print("No test files found. Create test files in the test_cases directory.")
            return {}
            
        for test_file in test_files:
            # Skip files that don't match the pattern
            if pattern and pattern not in test_file:
                continue
                
            # Import the test module
            module_name = os.path.splitext(os.path.basename(test_file))[0]
            spec = importlib.util.spec_from_file_location(module_name, test_file)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            
            # Add tests from the module to the suite
            tests = loader.loadTestsFromModule(module)
            suite.addTests(tests)
        
        # Run the tests
        runner = unittest.TextTestRunner(verbosity=2)
        result = runner.run(suite)
        
        # Store the results
        self.results = {
            "total": result.testsRun,
            "failures": len(result.failures),
            "errors": len(result.errors),
            "skipped": len(result.skipped),
            "success": result.wasSuccessful()
        }
        
        return self.results
    
    def generate_report(self, output_file: str = None) -> str:
        """Generate a test report in markdown format."""
        if not self.results:
            return "No test results available. Run tests first."
            
        report = "# Anarchy Inference Test Report\n\n"
        report += f"## Summary\n\n"
        report += f"- **Total Tests**: {self.results['total']}\n"
        report += f"- **Passed**: {self.results['total'] - self.results['failures'] - self.results['errors']}\n"
        report += f"- **Failed**: {self.results['failures']}\n"
        report += f"- **Errors**: {self.results['errors']}\n"
        report += f"- **Skipped**: {self.results['skipped']}\n"
        report += f"- **Success**: {'Yes' if self.results['success'] else 'No'}\n\n"
        
        # Add timestamp
        report += f"Generated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n"
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report)
            print(f"Report written to {output_file}")
            
        return report


class TokenEfficiencyTester:
    """Tests token efficiency of Anarchy Inference code compared to other languages."""
    
    def __init__(self, samples_dir: str = None):
        """Initialize with directory containing code samples."""
        self.samples_dir = samples_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "code_samples"
        )
        self.interpreter = anarchy.Interpreter()
        
    def compare_token_efficiency(self, task_name: str) -> Dict[str, Any]:
        """Compare token efficiency between Anarchy Inference and other languages for a specific task."""
        results = {}
        
        # Find all relevant files for this task
        anarchy_file = os.path.join(self.samples_dir, f"{task_name}_anarchy_inference.ai")
        python_file = os.path.join(self.samples_dir, f"{task_name}_python.py")
        js_file = os.path.join(self.samples_dir, f"{task_name}_javascript.js")
        rust_file = os.path.join(self.samples_dir, f"{task_name}_rust.rs")
        
        # Check if optimized version exists
        anarchy_optimized_file = os.path.join(self.samples_dir, f"{task_name}_anarchy_inference_optimized.ai")
        
        # Read files and count tokens
        if os.path.exists(anarchy_file):
            with open(anarchy_file, 'r') as f:
                anarchy_code = f.read()
            results["anarchy"] = {
                "tokens": self.interpreter.count_tokens(anarchy_code),
                "code": anarchy_code
            }
            
        if os.path.exists(anarchy_optimized_file):
            with open(anarchy_optimized_file, 'r') as f:
                anarchy_optimized_code = f.read()
            results["anarchy_optimized"] = {
                "tokens": self.interpreter.count_tokens(anarchy_optimized_code),
                "code": anarchy_optimized_code
            }
            
        if os.path.exists(python_file):
            with open(python_file, 'r') as f:
                python_code = f.read()
            results["python"] = {
                "tokens": self.interpreter.count_tokens(python_code),
                "code": python_code
            }
            
        if os.path.exists(js_file):
            with open(js_file, 'r') as f:
                js_code = f.read()
            results["javascript"] = {
                "tokens": self.interpreter.count_tokens(js_code),
                "code": js_code
            }
            
        if os.path.exists(rust_file):
            with open(rust_file, 'r') as f:
                rust_code = f.read()
            results["rust"] = {
                "tokens": self.interpreter.count_tokens(rust_code),
                "code": rust_code
            }
        
        # Calculate efficiency metrics
        if "anarchy" in results and "python" in results:
            python_tokens = results["python"]["tokens"]
            anarchy_tokens = results["anarchy"]["tokens"]
            results["anarchy"]["reduction_vs_python"] = (python_tokens - anarchy_tokens) / python_tokens
            
        if "anarchy_optimized" in results and "python" in results:
            python_tokens = results["python"]["tokens"]
            anarchy_opt_tokens = results["anarchy_optimized"]["tokens"]
            results["anarchy_optimized"]["reduction_vs_python"] = (python_tokens - anarchy_opt_tokens) / python_tokens
            
        return results
    
    def run_all_comparisons(self) -> Dict[str, Any]:
        """Run token efficiency comparisons for all available tasks."""
        tasks = set()
        
        # Discover all tasks by looking at file names
        for file in os.listdir(self.samples_dir):
            match = re.match(r"(.+)_(?:anarchy_inference|python|javascript|rust)(?:_optimized)?\.(?:ai|py|js|rs)$", file)
            if match:
                tasks.add(match.group(1))
        
        results = {}
        for task in tasks:
            results[task] = self.compare_token_efficiency(task)
            
        return results
    
    def generate_report(self, results: Dict[str, Any], output_file: str = None) -> str:
        """Generate a token efficiency report in markdown format."""
        report = "# Anarchy Inference Token Efficiency Report\n\n"
        
        # Overall summary
        total_python_tokens = 0
        total_anarchy_tokens = 0
        total_anarchy_opt_tokens = 0
        
        for task, task_results in results.items():
            if "python" in task_results:
                total_python_tokens += task_results["python"]["tokens"]
            if "anarchy" in task_results:
                total_anarchy_tokens += task_results["anarchy"]["tokens"]
            if "anarchy_optimized" in task_results:
                total_anarchy_opt_tokens += task_results["anarchy_optimized"]["tokens"]
        
        if total_python_tokens > 0 and total_anarchy_tokens > 0:
            overall_reduction = (total_python_tokens - total_anarchy_tokens) / total_python_tokens
            report += f"## Overall Token Efficiency\n\n"
            report += f"- **Standard Anarchy Inference**: {overall_reduction:.2%} reduction vs Python\n"
            
        if total_python_tokens > 0 and total_anarchy_opt_tokens > 0:
            overall_opt_reduction = (total_python_tokens - total_anarchy_opt_tokens) / total_python_tokens
            report += f"- **Optimized Anarchy Inference**: {overall_opt_reduction:.2%} reduction vs Python\n\n"
        
        # Per-task details
        report += f"## Task-by-Task Comparison\n\n"
        
        for task, task_results in results.items():
            report += f"### {task.replace('_', ' ').title()}\n\n"
            
            # Create comparison table
            report += "| Language | Token Count | vs Python |\n"
            report += "|----------|-------------|----------|\n"
            
            python_tokens = task_results.get("python", {}).get("tokens", 0)
            
            if "python" in task_results:
                report += f"| Python | {python_tokens} | - |\n"
                
            if "javascript" in task_results:
                js_tokens = task_results["javascript"]["tokens"]
                js_vs_py = "-" if python_tokens == 0 else f"{(js_tokens - python_tokens) / python_tokens:+.2%}"
                report += f"| JavaScript | {js_tokens} | {js_vs_py} |\n"
                
            if "rust" in task_results:
                rust_tokens = task_results["rust"]["tokens"]
                rust_vs_py = "-" if python_tokens == 0 else f"{(rust_tokens - python_tokens) / python_tokens:+.2%}"
                report += f"| Rust | {rust_tokens} | {rust_vs_py} |\n"
                
            if "anarchy" in task_results:
                anarchy_tokens = task_results["anarchy"]["tokens"]
                anarchy_vs_py = "-" if python_tokens == 0 else f"{(anarchy_tokens - python_tokens) / python_tokens:+.2%}"
                report += f"| Anarchy Inference | {anarchy_tokens} | {anarchy_vs_py} |\n"
                
            if "anarchy_optimized" in task_results:
                anarchy_opt_tokens = task_results["anarchy_optimized"]["tokens"]
                anarchy_opt_vs_py = "-" if python_tokens == 0 else f"{(anarchy_opt_tokens - python_tokens) / python_tokens:+.2%}"
                report += f"| Anarchy Inference (Optimized) | {anarchy_opt_tokens} | {anarchy_opt_vs_py} |\n"
                
            report += "\n"
        
        # Add timestamp
        report += f"Generated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report)
            print(f"Report written to {output_file}")
            
        return report


class PerformanceBenchmarker:
    """Benchmarks performance of Anarchy Inference code execution."""
    
    def __init__(self, benchmark_dir: str = None):
        """Initialize with directory containing benchmark code."""
        self.benchmark_dir = benchmark_dir or os.path.join(
            os.path.dirname(__file__), 
            "benchmarks"
        )
        self.interpreter = anarchy.Interpreter()
        
        # Create benchmark directory if it doesn't exist
        if not os.path.exists(self.benchmark_dir):
            os.makedirs(self.benchmark_dir)
            
    def run_benchmark(self, benchmark_file: str, iterations: int = 5) -> Dict[str, Any]:
        """Run a specific benchmark multiple times and collect performance metrics."""
        if not os.path.exists(benchmark_file):
            return {"error": f"Benchmark file {benchmark_file} not found"}
            
        with open(benchmark_file, 'r') as f:
            code = f.read()
            
        results = {
            "file": benchmark_file,
            "iterations": iterations,
            "execution_times_ms": [],
            "token_count": self.interpreter.count_tokens(code)
        }
        
        # Run the benchmark multiple times
        for _ in range(iterations):
            start_time = time.time()
            try:
                self.interpreter.execute(code)
                execution_time = (time.time() - start_time) * 1000  # Convert to ms
                results["execution_times_ms"].append(execution_time)
            except Exception as e:
                results["error"] = str(e)
                break
                
        # Calculate statistics
        if "error" not in results and results["execution_times_ms"]:
            times = results["execution_times_ms"]
            results["min_time_ms"] = min(times)
            results["max_time_ms"] = max(times)
            results["avg_time_ms"] = sum(times) / len(times)
            results["median_time_ms"] = sorted(times)[len(times) // 2]
            
        return results
    
    def run_all_benchmarks(self, iterations: int = 5) -> Dict[str, Any]:
        """Run all benchmarks in the benchmark directory."""
        results = {}
        
        # Create example benchmark if directory is empty
        if not os.listdir(self.benchmark_dir):
            self._create_example_benchmarks()
            
        for file in os.listdir(self.benchmark_dir):
            if file.endswith(".ai"):
                benchmark_file = os.path.join(self.benchmark_dir, file)
                benchmark_name = os.path.splitext(file)[0]
                results[benchmark_name] = self.run_benchmark(benchmark_file, iterations)
                
        return results
    
    def _create_example_benchmarks(self):
        """Create example benchmark files if none exist."""
        examples = {
            "fibonacci.ai": """
# Fibonacci sequence benchmark
fib(n)={
  ?(n<=1){return n}
  return fib(n-1)+fib(n-2)
}

# Calculate 20th Fibonacci number
result=fib(20)
print(result)
""",
            "sort_benchmark.ai": """
# Sorting benchmark
sort(arr)={
  n=len(arr)
  @(i=0;i<n;i++){
    @(j=0;j<n-i-1;j++){
      ?(arr[j]>arr[j+1]){
        temp=arr[j]
        arr[j]=arr[j+1]
        arr[j+1]=temp
      }
    }
  }
  return arr
}

# Create array with 100 random numbers
arr=[]
@(i=0;i<100;i++){
  arr.push(rand(0,1000))
}

# Sort the array
sorted=sort(arr)
print("Sorted first 5 elements:", sorted[0:5])
"""
        }
        
        for filename, content in examples.items():
            with open(os.path.join(self.benchmark_dir, filename), 'w') as f:
                f.write(content)
                
        print(f"Created example benchmarks in {self.benchmark_dir}")
    
    def generate_report(self, results: Dict[str, Any], output_file: str = None) -> str:
        """Generate a performance benchmark report in markdown format."""
        report = "# Anarchy Inference Performance Benchmark Report\n\n"
        
        # Overall summary
        total_benchmarks = len(results)
        successful_benchmarks = sum(1 for r in results.values() if "error" not in r)
        
        report += f"## Summary\n\n"
        report += f"- **Total Benchmarks**: {total_benchmarks}\n"
        report += f"- **Successful Benchmarks**: {successful_benchmarks}\n"
        report += f"- **Failed Benchmarks**: {total_benchmarks - successful_benchmarks}\n\n"
        
        # Performance table
        report += "## Performance Results\n\n"
        report += "| Benchmark | Token Count | Avg Time (ms) | Min Time (ms) | Max Time (ms) | Median Time (ms) |\n"
        report += "|-----------|-------------|---------------|---------------|---------------|------------------|\n"
        
        for name, result in results.items():
            if "error" in result:
                report += f"| {name} | - | Failed | - | - | - |\n"
            else:
                report += (f"| {name} | {result['token_count']} | {result['avg_time_ms']:.2f} | "
                          f"{result['min_time_ms']:.2f} | {result['max_time_ms']:.2f} | "
                          f"{result['median_time_ms']:.2f} |\n")
                
        report += "\n"
        
        # Individual benchmark details
        report += "## Detailed Results\n\n"
        
        for name, result in results.items():
            report += f"### {name}\n\n"
            
            if "error" in result:
                report += f"**Error**: {result['error']}\n\n"
            else:
                report += f"- **File**: {os.path.basename(result['file'])}\n"
                report += f"- **Token Count**: {result['token_count']}\n"
                report += f"- **Iterations**: {result['iterations']}\n"
                report += f"- **Average Time**: {result['avg_time_ms']:.2f} ms\n"
                report += f"- **Min Time**: {result['min_time_ms']:.2f} ms\n"
                report += f"- **Max Time**: {result['max_time_ms']:.2f} ms\n"
                report += f"- **Median Time**: {result['median_time_ms']:.2f} ms\n\n"
                
                # Add execution time graph (ASCII art)
                max_width = 40
                max_time = max(result['execution_times_ms'])
                
                report += "**Execution Times**:\n```\n"
                for i, time_ms in enumerate(result['execution_times_ms']):
                    bar_width = int((time_ms / max_time) * max_width) if max_time > 0 else 0
                    bar = "#" * bar_width
                    report += f"Run {i+1}: {bar} {time_ms:.2f} ms\n"
                report += "```\n\n"
        
        # Add timestamp
        report += f"Generated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report)
            print(f"Report written to {output_file}")
            
        return report


def main():
    """Main entry point for the testing framework."""
    parser = argparse.ArgumentParser(description="Anarchy Inference Testing Framework")
    subparsers = parser.add_subparsers(dest="command", help="Command to run")
    
    # Unit tests command
    test_parser = subparsers.add_parser("test", help="Run unit tests")
    test_parser.add_argument("--pattern", help="Filter tests by pattern")
    test_parser.add_argument("--report", help="Generate report file")
    
    # Token efficiency command
    token_parser = subparsers.add_parser("tokens", help="Run token efficiency tests")
    token_parser.add_argument("--task", help="Specific task to analyze")
    token_parser.add_argument("--report", help="Generate report file")
    
    # Performance benchmark command
    bench_parser = subparsers.add_parser("benchmark", help="Run performance benchmarks")
    bench_parser.add_argument("--iterations", type=int, default=5, help="Number of iterations per benchmark")
    bench_parser.add_argument("--report", help="Generate report file")
    
    # All-in-one command
    all_parser = subparsers.add_parser("all", help="Run all tests and generate reports")
    all_parser.add_argument("--output-dir", default="./reports", help="Directory for reports")
    
    args = parser.parse_args()
    
    if args.command == "test":
        runner = TestRunner()
        results = runner.run_tests(args.pattern)
        if args.report:
            runner.generate_report(args.report)
            
    elif args.command == "tokens":
        tester = TokenEfficiencyTester()
        if args.task:
            results = {args.task: tester.compare_token_efficiency(args.task)}
        else:
            results = tester.run_all_comparisons()
            
        if args.report:
            tester.generate_report(results, args.report)
        else:
            print(json.dumps(results, indent=2))
            
    elif args.command == "benchmark":
        benchmarker = PerformanceBenchmarker()
        results = benchmarker.run_all_benchmarks(args.iterations)
        
        if args.report:
            benchmarker.generate_report(results, args.report)
        else:
            print(json.dumps(results, indent=2))
            
    elif args.command == "all":
        # Create output directory
        os.makedirs(args.output_dir, exist_ok=True)
        
        # Run unit tests
        runner = TestRunner()
        test_results = runner.run_tests()
        test_report = runner.generate_report(os.path.join(args.output_dir, "test_report.md"))
        
        # Run token efficiency tests
        tester = TokenEfficiencyTester()
        token_results = tester.run_all_comparisons()
        token_report = tester.generate_report(token_results, os.path.join(args.output_dir, "token_efficiency_report.md"))
        
        # Run benchmarks
        benchmarker = PerformanceBenchmarker()
        bench_results = benchmarker.run_all_benchmarks()
        bench_report = benchmarker.generate_report(bench_results, os.path.join(args.output_dir, "benchmark_report.md"))
        
        print(f"All reports generated in {args.output_dir}")
        
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
