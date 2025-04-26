"""
Performance Benchmarking for Anarchy Inference

This module provides functionality to benchmark the performance of Anarchy Inference programs,
helping identify bottlenecks and optimize code.
"""

import os
import sys
import json
import time
import statistics
import datetime
import platform
import psutil
import matplotlib.pyplot as plt
import numpy as np
from typing import Dict, List, Any, Optional, Tuple, Set, Callable, Union
from collections import defaultdict
import csv
import tempfile
import subprocess
import gc
import tracemalloc
from functools import wraps

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

class BenchmarkResult:
    """Represents the result of a benchmark run."""
    
    def __init__(self, 
                 name: str, 
                 execution_times: List[float],
                 memory_usage: List[float] = None,
                 token_counts: List[int] = None):
        """Initialize a benchmark result.
        
        Args:
            name: Name of the benchmark
            execution_times: List of execution times in seconds
            memory_usage: List of memory usage measurements in MB
            token_counts: List of token counts
        """
        self.name = name
        self.execution_times = execution_times
        self.memory_usage = memory_usage or []
        self.token_counts = token_counts or []
        self.timestamp = time.time()
    
    @property
    def avg_execution_time(self) -> float:
        """Get the average execution time."""
        if not self.execution_times:
            return 0.0
        return statistics.mean(self.execution_times)
    
    @property
    def min_execution_time(self) -> float:
        """Get the minimum execution time."""
        if not self.execution_times:
            return 0.0
        return min(self.execution_times)
    
    @property
    def max_execution_time(self) -> float:
        """Get the maximum execution time."""
        if not self.execution_times:
            return 0.0
        return max(self.execution_times)
    
    @property
    def std_execution_time(self) -> float:
        """Get the standard deviation of execution times."""
        if len(self.execution_times) < 2:
            return 0.0
        return statistics.stdev(self.execution_times)
    
    @property
    def avg_memory_usage(self) -> float:
        """Get the average memory usage."""
        if not self.memory_usage:
            return 0.0
        return statistics.mean(self.memory_usage)
    
    @property
    def avg_token_count(self) -> float:
        """Get the average token count."""
        if not self.token_counts:
            return 0.0
        return statistics.mean(self.token_counts)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the benchmark result to a dictionary."""
        return {
            "name": self.name,
            "execution_times": self.execution_times,
            "memory_usage": self.memory_usage,
            "token_counts": self.token_counts,
            "avg_execution_time": self.avg_execution_time,
            "min_execution_time": self.min_execution_time,
            "max_execution_time": self.max_execution_time,
            "std_execution_time": self.std_execution_time,
            "avg_memory_usage": self.avg_memory_usage,
            "avg_token_count": self.avg_token_count,
            "timestamp": self.timestamp
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'BenchmarkResult':
        """Create a benchmark result from a dictionary."""
        result = cls(
            name=data["name"],
            execution_times=data["execution_times"],
            memory_usage=data.get("memory_usage", []),
            token_counts=data.get("token_counts", [])
        )
        result.timestamp = data.get("timestamp", time.time())
        return result


class BenchmarkSuite:
    """A collection of benchmarks to run together."""
    
    def __init__(self, name: str, description: str = ""):
        """Initialize a benchmark suite.
        
        Args:
            name: Name of the suite
            description: Description of the suite
        """
        self.name = name
        self.description = description
        self.benchmarks: Dict[str, Dict[str, Any]] = {}
        self.results: Dict[str, BenchmarkResult] = {}
    
    def add_benchmark(self, 
                      name: str, 
                      code: str, 
                      setup_code: str = "", 
                      teardown_code: str = "",
                      description: str = "",
                      tags: List[str] = None):
        """Add a benchmark to the suite.
        
        Args:
            name: Name of the benchmark
            code: Code to benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            description: Description of the benchmark
            tags: Tags for categorizing the benchmark
        """
        self.benchmarks[name] = {
            "code": code,
            "setup_code": setup_code,
            "teardown_code": teardown_code,
            "description": description,
            "tags": tags or []
        }
    
    def add_benchmark_from_file(self, 
                               name: str, 
                               file_path: str, 
                               setup_file: str = None, 
                               teardown_file: str = None,
                               description: str = "",
                               tags: List[str] = None):
        """Add a benchmark from a file.
        
        Args:
            name: Name of the benchmark
            file_path: Path to the file containing the code to benchmark
            setup_file: Path to the file containing setup code
            teardown_file: Path to the file containing teardown code
            description: Description of the benchmark
            tags: Tags for categorizing the benchmark
        """
        # Read the benchmark code
        with open(file_path, 'r') as f:
            code = f.read()
        
        # Read the setup code if provided
        setup_code = ""
        if setup_file:
            with open(setup_file, 'r') as f:
                setup_code = f.read()
        
        # Read the teardown code if provided
        teardown_code = ""
        if teardown_file:
            with open(teardown_file, 'r') as f:
                teardown_code = f.read()
        
        # Add the benchmark
        self.add_benchmark(
            name=name,
            code=code,
            setup_code=setup_code,
            teardown_code=teardown_code,
            description=description,
            tags=tags
        )
    
    def get_benchmark_names(self) -> List[str]:
        """Get the names of all benchmarks in the suite.
        
        Returns:
            List of benchmark names
        """
        return list(self.benchmarks.keys())
    
    def get_benchmarks_by_tag(self, tag: str) -> List[str]:
        """Get the names of benchmarks with a specific tag.
        
        Args:
            tag: Tag to filter by
            
        Returns:
            List of benchmark names
        """
        return [name for name, benchmark in self.benchmarks.items() 
                if tag in benchmark["tags"]]
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the benchmark suite to a dictionary.
        
        Returns:
            Dictionary representation of the suite
        """
        return {
            "name": self.name,
            "description": self.description,
            "benchmarks": self.benchmarks,
            "results": {name: result.to_dict() for name, result in self.results.items()}
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'BenchmarkSuite':
        """Create a benchmark suite from a dictionary.
        
        Args:
            data: Dictionary representation of a suite
            
        Returns:
            Benchmark suite
        """
        suite = cls(
            name=data["name"],
            description=data.get("description", "")
        )
        
        suite.benchmarks = data.get("benchmarks", {})
        
        # Load results
        for name, result_data in data.get("results", {}).items():
            suite.results[name] = BenchmarkResult.from_dict(result_data)
        
        return suite
    
    def save(self, file_path: str):
        """Save the benchmark suite to a file.
        
        Args:
            file_path: Path to save the suite to
        """
        with open(file_path, 'w') as f:
            json.dump(self.to_dict(), f, indent=2)
    
    @classmethod
    def load(cls, file_path: str) -> 'BenchmarkSuite':
        """Load a benchmark suite from a file.
        
        Args:
            file_path: Path to load the suite from
            
        Returns:
            Benchmark suite
        """
        with open(file_path, 'r') as f:
            data = json.load(f)
        
        return cls.from_dict(data)


class TokenCounter:
    """Counts tokens in Anarchy Inference code."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the token counter.
        
        Args:
            interpreter: The Anarchy Inference interpreter
        """
        self.interpreter = interpreter
    
    def count_tokens(self, code: str) -> int:
        """Count the number of tokens in code.
        
        Args:
            code: Code to count tokens in
            
        Returns:
            Number of tokens
        """
        # This is a simplified implementation
        # In a real implementation, this would use the interpreter's lexer
        
        # For demonstration purposes, we'll use a simple approximation
        # In a real implementation, this would be more accurate
        tokens = self.interpreter.tokenize(code)
        return len(tokens)


class MemoryProfiler:
    """Profiles memory usage of Anarchy Inference code."""
    
    def __init__(self):
        """Initialize the memory profiler."""
        self.enabled = False
    
    def start(self):
        """Start profiling memory usage."""
        tracemalloc.start()
        self.enabled = True
    
    def stop(self) -> float:
        """Stop profiling memory usage and return the peak.
        
        Returns:
            Peak memory usage in MB
        """
        if not self.enabled:
            return 0.0
        
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        self.enabled = False
        
        # Convert to MB
        return peak / (1024 * 1024)
    
    def measure(self, func: Callable) -> Tuple[Any, float]:
        """Measure the memory usage of a function.
        
        Args:
            func: Function to measure
            
        Returns:
            Tuple of (function result, peak memory usage in MB)
        """
        self.start()
        result = func()
        memory_usage = self.stop()
        return result, memory_usage


class BenchmarkRunner:
    """Runs benchmarks and collects results."""
    
    def __init__(self, 
                 interpreter: 'anarchy.Interpreter',
                 iterations: int = 5,
                 warmup_iterations: int = 2,
                 gc_between_runs: bool = True):
        """Initialize the benchmark runner.
        
        Args:
            interpreter: The Anarchy Inference interpreter
            iterations: Number of iterations to run each benchmark
            warmup_iterations: Number of warmup iterations
            gc_between_runs: Whether to run garbage collection between runs
        """
        self.interpreter = interpreter
        self.iterations = iterations
        self.warmup_iterations = warmup_iterations
        self.gc_between_runs = gc_between_runs
        self.token_counter = TokenCounter(interpreter)
        self.memory_profiler = MemoryProfiler()
        self.system_info = self._get_system_info()
    
    def _get_system_info(self) -> Dict[str, str]:
        """Get information about the system.
        
        Returns:
            Dictionary with system information
        """
        return {
            "platform": platform.platform(),
            "processor": platform.processor(),
            "python_version": platform.python_version(),
            "memory": f"{psutil.virtual_memory().total / (1024**3):.2f} GB",
            "cpu_count": str(psutil.cpu_count(logical=False)),
            "logical_cpu_count": str(psutil.cpu_count(logical=True))
        }
    
    def run_benchmark(self, 
                     name: str, 
                     code: str, 
                     setup_code: str = "", 
                     teardown_code: str = "") -> BenchmarkResult:
        """Run a single benchmark.
        
        Args:
            name: Name of the benchmark
            code: Code to benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            
        Returns:
            Benchmark result
        """
        execution_times = []
        memory_usage = []
        token_counts = []
        
        # Count tokens
        token_count = self.token_counter.count_tokens(code)
        token_counts.append(token_count)
        
        # Run warmup iterations
        for _ in range(self.warmup_iterations):
            # Run setup code
            if setup_code:
                self.interpreter.execute(setup_code)
            
            # Run the benchmark code (but don't measure it)
            self.interpreter.execute(code)
            
            # Run teardown code
            if teardown_code:
                self.interpreter.execute(teardown_code)
            
            # Run garbage collection if enabled
            if self.gc_between_runs:
                gc.collect()
        
        # Run measured iterations
        for _ in range(self.iterations):
            # Run setup code
            if setup_code:
                self.interpreter.execute(setup_code)
            
            # Measure execution time
            start_time = time.time()
            
            # Measure memory usage
            def run_code():
                return self.interpreter.execute(code)
            
            _, peak_memory = self.memory_profiler.measure(run_code)
            
            end_time = time.time()
            execution_time = end_time - start_time
            
            # Record measurements
            execution_times.append(execution_time)
            memory_usage.append(peak_memory)
            
            # Run teardown code
            if teardown_code:
                self.interpreter.execute(teardown_code)
            
            # Run garbage collection if enabled
            if self.gc_between_runs:
                gc.collect()
        
        # Create and return the result
        return BenchmarkResult(
            name=name,
            execution_times=execution_times,
            memory_usage=memory_usage,
            token_counts=token_counts
        )
    
    def run_suite(self, suite: BenchmarkSuite) -> Dict[str, BenchmarkResult]:
        """Run all benchmarks in a suite.
        
        Args:
            suite: Benchmark suite to run
            
        Returns:
            Dictionary mapping benchmark names to results
        """
        results = {}
        
        for name, benchmark in suite.benchmarks.items():
            print(f"Running benchmark: {name}")
            
            result = self.run_benchmark(
                name=name,
                code=benchmark["code"],
                setup_code=benchmark["setup_code"],
                teardown_code=benchmark["teardown_code"]
            )
            
            results[name] = result
            suite.results[name] = result
            
            print(f"  Avg time: {result.avg_execution_time:.6f} seconds")
            print(f"  Avg memory: {result.avg_memory_usage:.2f} MB")
            print(f"  Token count: {result.avg_token_count:.0f}")
        
        return results


class BenchmarkComparison:
    """Compares benchmark results."""
    
    def __init__(self, 
                 baseline_results: Dict[str, BenchmarkResult],
                 current_results: Dict[str, BenchmarkResult]):
        """Initialize the benchmark comparison.
        
        Args:
            baseline_results: Baseline benchmark results
            current_results: Current benchmark results
        """
        self.baseline_results = baseline_results
        self.current_results = current_results
    
    def compare(self) -> Dict[str, Dict[str, Any]]:
        """Compare the current results to the baseline.
        
        Returns:
            Dictionary with comparison results
        """
        comparison = {}
        
        # Find benchmarks in both sets
        common_benchmarks = set(self.baseline_results.keys()) & set(self.current_results.keys())
        
        for name in common_benchmarks:
            baseline = self.baseline_results[name]
            current = self.current_results[name]
            
            # Calculate time difference
            time_diff = current.avg_execution_time - baseline.avg_execution_time
            time_diff_pct = (time_diff / baseline.avg_execution_time) * 100 if baseline.avg_execution_time > 0 else 0
            
            # Calculate memory difference
            memory_diff = current.avg_memory_usage - baseline.avg_memory_usage
            memory_diff_pct = (memory_diff / baseline.avg_memory_usage) * 100 if baseline.avg_memory_usage > 0 else 0
            
            # Calculate token difference
            token_diff = current.avg_token_count - baseline.avg_token_count
            token_diff_pct = (token_diff / baseline.avg_token_count) * 100 if baseline.avg_token_count > 0 else 0
            
            comparison[name] = {
                "time_diff": time_diff,
                "time_diff_pct": time_diff_pct,
                "memory_diff": memory_diff,
                "memory_diff_pct": memory_diff_pct,
                "token_diff": token_diff,
                "token_diff_pct": token_diff_pct,
                "baseline": baseline.to_dict(),
                "current": current.to_dict()
            }
        
        return comparison
    
    def summary(self) -> Dict[str, Any]:
        """Generate a summary of the comparison.
        
        Returns:
            Dictionary with comparison summary
        """
        comparison = self.compare()
        
        # Calculate averages
        time_diffs = [data["time_diff_pct"] for data in comparison.values()]
        memory_diffs = [data["memory_diff_pct"] for data in comparison.values()]
        token_diffs = [data["token_diff_pct"] for data in comparison.values()]
        
        avg_time_diff = statistics.mean(time_diffs) if time_diffs else 0
        avg_memory_diff = statistics.mean(memory_diffs) if memory_diffs else 0
        avg_token_diff = statistics.mean(token_diffs) if token_diffs else 0
        
        # Find best and worst
        if time_diffs:
            best_time = min(time_diffs)
            worst_time = max(time_diffs)
            best_time_benchmark = next(name for name, data in comparison.items() 
                                      if data["time_diff_pct"] == best_time)
            worst_time_benchmark = next(name for name, data in comparison.items() 
                                       if data["time_diff_pct"] == worst_time)
        else:
            best_time = 0
            worst_time = 0
            best_time_benchmark = ""
            worst_time_benchmark = ""
        
        if memory_diffs:
            best_memory = min(memory_diffs)
            worst_memory = max(memory_diffs)
            best_memory_benchmark = next(name for name, data in comparison.items() 
                                        if data["memory_diff_pct"] == best_memory)
            worst_memory_benchmark = next(name for name, data in comparison.items() 
                                         if data["memory_diff_pct"] == worst_memory)
        else:
            best_memory = 0
            worst_memory = 0
            best_memory_benchmark = ""
            worst_memory_benchmark = ""
        
        if token_diffs:
            best_token = min(token_diffs)
            worst_token = max(token_diffs)
            best_token_benchmark = next(name for name, data in comparison.items() 
                                       if data["token_diff_pct"] == best_token)
            worst_token_benchmark = next(name for name, data in comparison.items() 
                                        if data["token_diff_pct"] == worst_token)
        else:
            best_token = 0
            worst_token = 0
            best_token_benchmark = ""
            worst_token_benchmark = ""
        
        return {
            "benchmark_count": len(comparison),
            "avg_time_diff_pct": avg_time_diff,
            "avg_memory_diff_pct": avg_memory_diff,
            "avg_token_diff_pct": avg_token_diff,
            "best_time": {
                "benchmark": best_time_benchmark,
                "diff_pct": best_time
            },
            "worst_time": {
                "benchmark": worst_time_benchmark,
                "diff_pct": worst_time
            },
            "best_memory": {
                "benchmark": best_memory_benchmark,
                "diff_pct": best_memory
            },
            "worst_memory": {
                "benchmark": worst_memory_benchmark,
                "diff_pct": worst_memory
            },
            "best_token": {
                "benchmark": best_token_benchmark,
                "diff_pct": best_token
            },
            "worst_token": {
                "benchmark": worst_token_benchmark,
                "diff_pct": worst_token
            }
        }


class BenchmarkReporter:
    """Generates reports from benchmark results."""
    
    def __init__(self, output_dir: str = None):
        """Initialize the benchmark reporter.
        
        Args:
            output_dir: Directory to save reports
        """
        self.output_dir = output_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "benchmark_reports"
        )
        
        # Create the output directory if it doesn't exist
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
    
    def generate_text_report(self, 
                            suite: BenchmarkSuite, 
                            comparison: BenchmarkComparison = None) -> str:
        """Generate a text report of benchmark results.
        
        Args:
            suite: Benchmark suite with results
            comparison: Optional comparison to include
            
        Returns:
            Path to the generated report
        """
        # Create report content
        content = f"Benchmark Report: {suite.name}\n"
        content += f"{'=' * len(suite.name)}\n\n"
        
        if suite.description:
            content += f"{suite.description}\n\n"
        
        content += f"Generated: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n"
        
        # Add system information
        content += "System Information:\n"
        content += "-----------------\n"
        
        # We don't have system info in the suite, so we'll create some dummy info
        system_info = {
            "platform": platform.platform(),
            "processor": platform.processor(),
            "python_version": platform.python_version(),
            "memory": f"{psutil.virtual_memory().total / (1024**3):.2f} GB",
            "cpu_count": str(psutil.cpu_count(logical=False)),
            "logical_cpu_count": str(psutil.cpu_count(logical=True))
        }
        
        for key, value in system_info.items():
            content += f"{key}: {value}\n"
        
        content += "\n"
        
        # Add benchmark results
        content += "Benchmark Results:\n"
        content += "-----------------\n"
        
        for name, result in suite.results.items():
            content += f"\n{name}:\n"
            content += f"  Average execution time: {result.avg_execution_time:.6f} seconds\n"
            content += f"  Min execution time: {result.min_execution_time:.6f} seconds\n"
            content += f"  Max execution time: {result.max_execution_time:.6f} seconds\n"
            content += f"  Std dev execution time: {result.std_execution_time:.6f} seconds\n"
            
            if result.memory_usage:
                content += f"  Average memory usage: {result.avg_memory_usage:.2f} MB\n"
            
            if result.token_counts:
                content += f"  Average token count: {result.avg_token_count:.0f}\n"
        
        # Add comparison if provided
        if comparison:
            content += "\nComparison to Baseline:\n"
            content += "----------------------\n"
            
            summary = comparison.summary()
            
            content += f"\nOverall:\n"
            content += f"  Average time difference: {summary['avg_time_diff_pct']:.2f}%\n"
            content += f"  Average memory difference: {summary['avg_memory_diff_pct']:.2f}%\n"
            content += f"  Average token difference: {summary['avg_token_diff_pct']:.2f}%\n"
            
            content += f"\nBest improvements:\n"
            content += f"  Time: {summary['best_time']['benchmark']} ({summary['best_time']['diff_pct']:.2f}%)\n"
            content += f"  Memory: {summary['best_memory']['benchmark']} ({summary['best_memory']['diff_pct']:.2f}%)\n"
            content += f"  Tokens: {summary['best_token']['benchmark']} ({summary['best_token']['diff_pct']:.2f}%)\n"
            
            content += f"\nWorst regressions:\n"
            content += f"  Time: {summary['worst_time']['benchmark']} ({summary['worst_time']['diff_pct']:.2f}%)\n"
            content += f"  Memory: {summary['worst_memory']['benchmark']} ({summary['worst_memory']['diff_pct']:.2f}%)\n"
            content += f"  Tokens: {summary['worst_token']['benchmark']} ({summary['worst_token']['diff_pct']:.2f}%)\n"
            
            content += f"\nDetailed comparison:\n"
            
            comparison_data = comparison.compare()
            for name, data in comparison_data.items():
                content += f"\n{name}:\n"
                content += f"  Time: {data['time_diff_pct']:.2f}% ({data['baseline']['avg_execution_time']:.6f}s -> {data['current']['avg_execution_time']:.6f}s)\n"
                content += f"  Memory: {data['memory_diff_pct']:.2f}% ({data['baseline']['avg_memory_usage']:.2f}MB -> {data['current']['avg_memory_usage']:.2f}MB)\n"
                content += f"  Tokens: {data['token_diff_pct']:.2f}% ({data['baseline']['avg_token_count']:.0f} -> {data['current']['avg_token_count']:.0f})\n"
        
        # Write to file
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(self.output_dir, f"benchmark_report_{timestamp}.txt")
        
        with open(output_path, 'w') as f:
            f.write(content)
        
        return output_path
    
    def generate_csv_report(self, suite: BenchmarkSuite) -> str:
        """Generate a CSV report of benchmark results.
        
        Args:
            suite: Benchmark suite with results
            
        Returns:
            Path to the generated report
        """
        # Create CSV content
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(self.output_dir, f"benchmark_report_{timestamp}.csv")
        
        with open(output_path, 'w', newline='') as f:
            writer = csv.writer(f)
            
            # Write header
            writer.writerow([
                "Benchmark", 
                "Avg Time (s)", 
                "Min Time (s)", 
                "Max Time (s)", 
                "Std Dev Time (s)",
                "Avg Memory (MB)",
                "Avg Tokens"
            ])
            
            # Write data
            for name, result in suite.results.items():
                writer.writerow([
                    name,
                    result.avg_execution_time,
                    result.min_execution_time,
                    result.max_execution_time,
                    result.std_execution_time,
                    result.avg_memory_usage,
                    result.avg_token_count
                ])
        
        return output_path
    
    def generate_html_report(self, 
                            suite: BenchmarkSuite, 
                            comparison: BenchmarkComparison = None) -> str:
        """Generate an HTML report of benchmark results.
        
        Args:
            suite: Benchmark suite with results
            comparison: Optional comparison to include
            
        Returns:
            Path to the generated report
        """
        # Create HTML content
        html = f"""<!DOCTYPE html>
<html>
<head>
    <title>Benchmark Report: {suite.name}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            color: #333;
        }}
        h1, h2, h3 {{
            color: #2c3e50;
        }}
        .summary {{
            background-color: #f8f9fa;
            border-radius: 5px;
            padding: 15px;
            margin-bottom: 20px;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin-bottom: 20px;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background-color: #f2f2f2;
        }}
        tr:nth-child(even) {{
            background-color: #f9f9f9;
        }}
        .positive {{
            color: green;
        }}
        .negative {{
            color: red;
        }}
        .chart-container {{
            width: 100%;
            height: 400px;
            margin-bottom: 20px;
        }}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <h1>Benchmark Report: {suite.name}</h1>
    
    <div class="summary">
        <p>{suite.description}</p>
        <p>Generated: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
    </div>
    
    <h2>System Information</h2>
    <table>
        <tr>
            <th>Property</th>
            <th>Value</th>
        </tr>
"""
        
        # Add system information
        system_info = {
            "platform": platform.platform(),
            "processor": platform.processor(),
            "python_version": platform.python_version(),
            "memory": f"{psutil.virtual_memory().total / (1024**3):.2f} GB",
            "cpu_count": str(psutil.cpu_count(logical=False)),
            "logical_cpu_count": str(psutil.cpu_count(logical=True))
        }
        
        for key, value in system_info.items():
            html += f"""
        <tr>
            <td>{key}</td>
            <td>{value}</td>
        </tr>"""
        
        html += """
    </table>
    
    <h2>Benchmark Results</h2>
    <table>
        <tr>
            <th>Benchmark</th>
            <th>Avg Time (s)</th>
            <th>Min Time (s)</th>
            <th>Max Time (s)</th>
            <th>Std Dev Time (s)</th>
            <th>Avg Memory (MB)</th>
            <th>Avg Tokens</th>
        </tr>
"""
        
        # Add benchmark results
        for name, result in suite.results.items():
            html += f"""
        <tr>
            <td>{name}</td>
            <td>{result.avg_execution_time:.6f}</td>
            <td>{result.min_execution_time:.6f}</td>
            <td>{result.max_execution_time:.6f}</td>
            <td>{result.std_execution_time:.6f}</td>
            <td>{result.avg_memory_usage:.2f}</td>
            <td>{result.avg_token_count:.0f}</td>
        </tr>"""
        
        html += """
    </table>
    
    <div class="chart-container">
        <canvas id="timeChart"></canvas>
    </div>
    
    <div class="chart-container">
        <canvas id="memoryChart"></canvas>
    </div>
    
    <div class="chart-container">
        <canvas id="tokenChart"></canvas>
    </div>
"""
        
        # Add comparison if provided
        if comparison:
            html += """
    <h2>Comparison to Baseline</h2>
    <table>
        <tr>
            <th>Benchmark</th>
            <th>Time Diff (%)</th>
            <th>Memory Diff (%)</th>
            <th>Token Diff (%)</th>
            <th>Baseline Time (s)</th>
            <th>Current Time (s)</th>
            <th>Baseline Memory (MB)</th>
            <th>Current Memory (MB)</th>
            <th>Baseline Tokens</th>
            <th>Current Tokens</th>
        </tr>
"""
            
            comparison_data = comparison.compare()
            for name, data in comparison_data.items():
                time_class = "positive" if data["time_diff_pct"] <= 0 else "negative"
                memory_class = "positive" if data["memory_diff_pct"] <= 0 else "negative"
                token_class = "positive" if data["token_diff_pct"] <= 0 else "negative"
                
                html += f"""
        <tr>
            <td>{name}</td>
            <td class="{time_class}">{data["time_diff_pct"]:.2f}%</td>
            <td class="{memory_class}">{data["memory_diff_pct"]:.2f}%</td>
            <td class="{token_class}">{data["token_diff_pct"]:.2f}%</td>
            <td>{data["baseline"]["avg_execution_time"]:.6f}</td>
            <td>{data["current"]["avg_execution_time"]:.6f}</td>
            <td>{data["baseline"]["avg_memory_usage"]:.2f}</td>
            <td>{data["current"]["avg_memory_usage"]:.2f}</td>
            <td>{data["baseline"]["avg_token_count"]:.0f}</td>
            <td>{data["current"]["avg_token_count"]:.0f}</td>
        </tr>"""
            
            html += """
    </table>
    
    <div class="chart-container">
        <canvas id="comparisonChart"></canvas>
    </div>
"""
        
        # Add JavaScript for charts
        html += """
    <script>
        // Time chart
        const timeCtx = document.getElementById('timeChart').getContext('2d');
        new Chart(timeCtx, {
            type: 'bar',
            data: {
                labels: ["""
        
        # Add benchmark names for time chart
        html += ", ".join([f"'{name}'" for name in suite.results.keys()])
        
        html += """],
                datasets: [{
                    label: 'Execution Time (s)',
                    data: ["""
        
        # Add execution times for time chart
        html += ", ".join([f"{result.avg_execution_time}" for result in suite.results.values()])
        
        html += """],
                    backgroundColor: 'rgba(54, 162, 235, 0.5)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Average Execution Time'
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Time (seconds)'
                        }
                    }
                }
            }
        });
        
        // Memory chart
        const memoryCtx = document.getElementById('memoryChart').getContext('2d');
        new Chart(memoryCtx, {
            type: 'bar',
            data: {
                labels: ["""
        
        # Add benchmark names for memory chart
        html += ", ".join([f"'{name}'" for name in suite.results.keys()])
        
        html += """],
                datasets: [{
                    label: 'Memory Usage (MB)',
                    data: ["""
        
        # Add memory usage for memory chart
        html += ", ".join([f"{result.avg_memory_usage}" for result in suite.results.values()])
        
        html += """],
                    backgroundColor: 'rgba(75, 192, 192, 0.5)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Average Memory Usage'
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Memory (MB)'
                        }
                    }
                }
            }
        });
        
        // Token chart
        const tokenCtx = document.getElementById('tokenChart').getContext('2d');
        new Chart(tokenCtx, {
            type: 'bar',
            data: {
                labels: ["""
        
        # Add benchmark names for token chart
        html += ", ".join([f"'{name}'" for name in suite.results.keys()])
        
        html += """],
                datasets: [{
                    label: 'Token Count',
                    data: ["""
        
        # Add token counts for token chart
        html += ", ".join([f"{result.avg_token_count}" for result in suite.results.values()])
        
        html += """],
                    backgroundColor: 'rgba(255, 159, 64, 0.5)',
                    borderColor: 'rgba(255, 159, 64, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Average Token Count'
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Tokens'
                        }
                    }
                }
            }
        });
"""
        
        # Add comparison chart if provided
        if comparison:
            html += """
        // Comparison chart
        const comparisonCtx = document.getElementById('comparisonChart').getContext('2d');
        new Chart(comparisonCtx, {
            type: 'bar',
            data: {
                labels: ["""
            
            # Add benchmark names for comparison chart
            comparison_data = comparison.compare()
            html += ", ".join([f"'{name}'" for name in comparison_data.keys()])
            
            html += """],
                datasets: [{
                    label: 'Time Difference (%)',
                    data: ["""
            
            # Add time differences for comparison chart
            html += ", ".join([f"{data['time_diff_pct']}" for data in comparison_data.values()])
            
            html += """],
                    backgroundColor: 'rgba(54, 162, 235, 0.5)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }, {
                    label: 'Memory Difference (%)',
                    data: ["""
            
            # Add memory differences for comparison chart
            html += ", ".join([f"{data['memory_diff_pct']}" for data in comparison_data.values()])
            
            html += """],
                    backgroundColor: 'rgba(75, 192, 192, 0.5)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }, {
                    label: 'Token Difference (%)',
                    data: ["""
            
            # Add token differences for comparison chart
            html += ", ".join([f"{data['token_diff_pct']}" for data in comparison_data.values()])
            
            html += """],
                    backgroundColor: 'rgba(255, 159, 64, 0.5)',
                    borderColor: 'rgba(255, 159, 64, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Comparison to Baseline (%)'
                    }
                },
                scales: {
                    y: {
                        title: {
                            display: true,
                            text: 'Difference (%)'
                        }
                    }
                }
            }
        });
"""
        
        html += """
    </script>
</body>
</html>
"""
        
        # Write to file
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(self.output_dir, f"benchmark_report_{timestamp}.html")
        
        with open(output_path, 'w') as f:
            f.write(html)
        
        return output_path
    
    def generate_json_report(self, 
                            suite: BenchmarkSuite, 
                            comparison: BenchmarkComparison = None) -> str:
        """Generate a JSON report of benchmark results.
        
        Args:
            suite: Benchmark suite with results
            comparison: Optional comparison to include
            
        Returns:
            Path to the generated report
        """
        # Create report data
        data = {
            "suite": suite.to_dict(),
            "system_info": {
                "platform": platform.platform(),
                "processor": platform.processor(),
                "python_version": platform.python_version(),
                "memory": f"{psutil.virtual_memory().total / (1024**3):.2f} GB",
                "cpu_count": str(psutil.cpu_count(logical=False)),
                "logical_cpu_count": str(psutil.cpu_count(logical=True))
            },
            "timestamp": time.time()
        }
        
        if comparison:
            data["comparison"] = {
                "details": comparison.compare(),
                "summary": comparison.summary()
            }
        
        # Write to file
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(self.output_dir, f"benchmark_report_{timestamp}.json")
        
        with open(output_path, 'w') as f:
            json.dump(data, f, indent=2)
        
        return output_path


class BenchmarkDatabase:
    """Stores historical benchmark results."""
    
    def __init__(self, db_file: str = None):
        """Initialize the benchmark database.
        
        Args:
            db_file: Path to the database file
        """
        self.db_file = db_file or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "benchmark_db.json"
        )
        self.history = {}
        
        # Load existing data if available
        if os.path.exists(self.db_file):
            self.load()
    
    def add_suite_results(self, suite: BenchmarkSuite, commit_id: str = None):
        """Add benchmark suite results to the database.
        
        Args:
            suite: Benchmark suite with results
            commit_id: Optional Git commit ID
        """
        timestamp = time.time()
        
        if suite.name not in self.history:
            self.history[suite.name] = []
        
        entry = {
            "timestamp": timestamp,
            "commit_id": commit_id,
            "results": {name: result.to_dict() for name, result in suite.results.items()}
        }
        
        self.history[suite.name].append(entry)
        self.save()
    
    def get_baseline_results(self, 
                            suite_name: str, 
                            commit_id: str = None) -> Dict[str, BenchmarkResult]:
        """Get baseline results for a suite.
        
        Args:
            suite_name: Name of the suite
            commit_id: Optional Git commit ID to use as baseline
            
        Returns:
            Dictionary mapping benchmark names to results
        """
        if suite_name not in self.history:
            return {}
        
        # If commit ID is provided, find that specific entry
        if commit_id:
            for entry in self.history[suite_name]:
                if entry.get("commit_id") == commit_id:
                    return {name: BenchmarkResult.from_dict(data) 
                           for name, data in entry["results"].items()}
        
        # Otherwise, use the most recent entry
        if self.history[suite_name]:
            latest_entry = max(self.history[suite_name], key=lambda e: e["timestamp"])
            return {name: BenchmarkResult.from_dict(data) 
                   for name, data in latest_entry["results"].items()}
        
        return {}
    
    def get_trend_data(self, 
                      suite_name: str, 
                      benchmark_name: str, 
                      days: int = 30) -> List[Dict[str, Any]]:
        """Get trend data for a specific benchmark.
        
        Args:
            suite_name: Name of the suite
            benchmark_name: Name of the benchmark
            days: Number of days to include
            
        Returns:
            List of data points
        """
        if suite_name not in self.history:
            return []
        
        cutoff_time = time.time() - (days * 24 * 60 * 60)
        
        trend_data = []
        for entry in self.history[suite_name]:
            if entry["timestamp"] >= cutoff_time and benchmark_name in entry["results"]:
                result = entry["results"][benchmark_name]
                trend_data.append({
                    "timestamp": entry["timestamp"],
                    "commit_id": entry.get("commit_id"),
                    "avg_execution_time": result["avg_execution_time"],
                    "avg_memory_usage": result.get("avg_memory_usage", 0),
                    "avg_token_count": result.get("avg_token_count", 0)
                })
        
        # Sort by timestamp
        trend_data.sort(key=lambda d: d["timestamp"])
        
        return trend_data
    
    def save(self):
        """Save the database to disk."""
        with open(self.db_file, 'w') as f:
            json.dump({"history": self.history}, f, indent=2)
    
    def load(self):
        """Load the database from disk."""
        try:
            with open(self.db_file, 'r') as f:
                data = json.load(f)
                self.history = data.get("history", {})
        except (json.JSONDecodeError, FileNotFoundError):
            self.history = {}


class PerformanceBenchmarker:
    """Main class for benchmarking Anarchy Inference code."""
    
    def __init__(self, 
                 interpreter: 'anarchy.Interpreter',
                 output_dir: str = None,
                 iterations: int = 5,
                 warmup_iterations: int = 2):
        """Initialize the performance benchmarker.
        
        Args:
            interpreter: The Anarchy Inference interpreter
            output_dir: Directory to save benchmark data and reports
            iterations: Number of iterations to run each benchmark
            warmup_iterations: Number of warmup iterations
        """
        self.interpreter = interpreter
        self.output_dir = output_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "benchmarks"
        )
        
        # Create the output directory if it doesn't exist
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
        
        # Initialize components
        self.runner = BenchmarkRunner(
            interpreter=interpreter,
            iterations=iterations,
            warmup_iterations=warmup_iterations
        )
        self.reporter = BenchmarkReporter(self.output_dir)
        self.database = BenchmarkDatabase(os.path.join(self.output_dir, "benchmark_db.json"))
    
    def create_suite(self, name: str, description: str = "") -> BenchmarkSuite:
        """Create a new benchmark suite.
        
        Args:
            name: Name of the suite
            description: Description of the suite
            
        Returns:
            New benchmark suite
        """
        return BenchmarkSuite(name=name, description=description)
    
    def load_suite(self, file_path: str) -> BenchmarkSuite:
        """Load a benchmark suite from a file.
        
        Args:
            file_path: Path to the suite file
            
        Returns:
            Loaded benchmark suite
        """
        return BenchmarkSuite.load(file_path)
    
    def run_suite(self, suite: BenchmarkSuite) -> Dict[str, BenchmarkResult]:
        """Run all benchmarks in a suite.
        
        Args:
            suite: Benchmark suite to run
            
        Returns:
            Dictionary mapping benchmark names to results
        """
        return self.runner.run_suite(suite)
    
    def compare_to_baseline(self, 
                           suite: BenchmarkSuite, 
                           baseline_commit: str = None) -> BenchmarkComparison:
        """Compare suite results to a baseline.
        
        Args:
            suite: Benchmark suite with results
            baseline_commit: Optional Git commit ID to use as baseline
            
        Returns:
            Benchmark comparison
        """
        baseline_results = self.database.get_baseline_results(suite.name, baseline_commit)
        
        if not baseline_results:
            print(f"No baseline results found for suite '{suite.name}'")
            return None
        
        return BenchmarkComparison(baseline_results, suite.results)
    
    def save_results(self, suite: BenchmarkSuite, commit_id: str = None):
        """Save benchmark results to the database.
        
        Args:
            suite: Benchmark suite with results
            commit_id: Optional Git commit ID
        """
        self.database.add_suite_results(suite, commit_id)
        
        # Also save the suite to a file
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        suite_path = os.path.join(self.output_dir, f"{suite.name}_{timestamp}.json")
        suite.save(suite_path)
    
    def generate_reports(self, 
                        suite: BenchmarkSuite, 
                        comparison: BenchmarkComparison = None) -> Dict[str, str]:
        """Generate reports for benchmark results.
        
        Args:
            suite: Benchmark suite with results
            comparison: Optional comparison to include
            
        Returns:
            Dictionary mapping report types to file paths
        """
        reports = {}
        
        # Generate text report
        text_report = self.reporter.generate_text_report(suite, comparison)
        reports["text"] = text_report
        
        # Generate CSV report
        csv_report = self.reporter.generate_csv_report(suite)
        reports["csv"] = csv_report
        
        # Generate HTML report
        html_report = self.reporter.generate_html_report(suite, comparison)
        reports["html"] = html_report
        
        # Generate JSON report
        json_report = self.reporter.generate_json_report(suite, comparison)
        reports["json"] = json_report
        
        return reports
    
    def benchmark_file(self, 
                      file_path: str, 
                      name: str = None, 
                      description: str = "") -> BenchmarkResult:
        """Benchmark a single file.
        
        Args:
            file_path: Path to the file to benchmark
            name: Name for the benchmark
            description: Description of the benchmark
            
        Returns:
            Benchmark result
        """
        # Use the file name as the benchmark name if not provided
        if name is None:
            name = os.path.basename(file_path)
        
        # Create a suite with a single benchmark
        suite = self.create_suite(name=f"Single_{name}", description=description)
        
        # Add the benchmark from the file
        suite.add_benchmark_from_file(name=name, file_path=file_path)
        
        # Run the suite
        self.run_suite(suite)
        
        # Return the result
        return suite.results[name]
    
    def benchmark_directory(self, 
                           dir_path: str, 
                           name: str = None, 
                           description: str = "",
                           pattern: str = "*.ai") -> BenchmarkSuite:
        """Benchmark all files in a directory.
        
        Args:
            dir_path: Path to the directory
            name: Name for the suite
            description: Description of the suite
            pattern: File pattern to match
            
        Returns:
            Benchmark suite with results
        """
        # Use the directory name as the suite name if not provided
        if name is None:
            name = os.path.basename(dir_path)
        
        # Create a suite
        suite = self.create_suite(name=name, description=description)
        
        # Find all matching files
        import glob
        file_paths = glob.glob(os.path.join(dir_path, pattern))
        
        # Add each file as a benchmark
        for file_path in file_paths:
            benchmark_name = os.path.basename(file_path)
            suite.add_benchmark_from_file(name=benchmark_name, file_path=file_path)
        
        # Run the suite
        self.run_suite(suite)
        
        return suite
    
    def benchmark_code(self, 
                      code: str, 
                      name: str, 
                      setup_code: str = "", 
                      teardown_code: str = "",
                      description: str = "") -> BenchmarkResult:
        """Benchmark a code snippet.
        
        Args:
            code: Code to benchmark
            name: Name for the benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            description: Description of the benchmark
            
        Returns:
            Benchmark result
        """
        # Create a suite with a single benchmark
        suite = self.create_suite(name=f"Snippet_{name}", description=description)
        
        # Add the benchmark
        suite.add_benchmark(
            name=name,
            code=code,
            setup_code=setup_code,
            teardown_code=teardown_code,
            description=description
        )
        
        # Run the suite
        self.run_suite(suite)
        
        # Return the result
        return suite.results[name]
    
    def benchmark_function(self, func: Callable, *args, **kwargs) -> Dict[str, float]:
        """Benchmark a Python function.
        
        Args:
            func: Function to benchmark
            *args: Arguments to pass to the function
            **kwargs: Keyword arguments to pass to the function
            
        Returns:
            Dictionary with benchmark metrics
        """
        # Create a wrapper function that calls the target function with the provided arguments
        def wrapper():
            return func(*args, **kwargs)
        
        # Measure execution time
        start_time = time.time()
        result = wrapper()
        execution_time = time.time() - start_time
        
        # Measure memory usage
        memory_profiler = MemoryProfiler()
        _, memory_usage = memory_profiler.measure(wrapper)
        
        return {
            "execution_time": execution_time,
            "memory_usage": memory_usage,
            "result": result
        }
    
    def create_standard_suite(self) -> BenchmarkSuite:
        """Create a standard benchmark suite with common operations.
        
        Returns:
            Benchmark suite
        """
        suite = self.create_suite(
            name="Standard_Benchmarks",
            description="Standard benchmarks for common Anarchy Inference operations"
        )
        
        # Add some standard benchmarks
        
        # Simple arithmetic
        suite.add_benchmark(
            name="Arithmetic",
            code="""
λ⟨ arithmetic ⟩
    x ← 1
    y ← 2
    z ← 0
    
    # Perform a series of arithmetic operations
    for i in range(1000) {
        z ← z + (x * i) / (y + 1)
        x ← x + 0.1
        y ← y * 1.01
    }
    
    return z
""",
            description="Basic arithmetic operations",
            tags=["math", "basic"]
        )
        
        # String manipulation
        suite.add_benchmark(
            name="String_Manipulation",
            code="""
λ⟨ string_manipulation ⟩
    text ← "Hello, world!"
    result ← ""
    
    # Perform a series of string operations
    for i in range(100) {
        result ← result + text.substring(0, i % text.length())
        result ← result.replace("l", "L")
        result ← result.replace("o", "O")
    }
    
    return result.length()
""",
            description="String manipulation operations",
            tags=["string", "basic"]
        )
        
        # Dictionary operations
        suite.add_benchmark(
            name="Dictionary_Operations",
            code="""
λ⟨ dictionary_operations ⟩
    dict ← {}
    
    # Perform a series of dictionary operations
    for i in range(1000) {
        key ← "key_" + i
        dict[key] ← i * i
    }
    
    sum ← 0
    for key in dict.keys() {
        sum ← sum + dict[key]
    }
    
    return sum
""",
            description="Dictionary operations",
            tags=["dictionary", "basic"]
        )
        
        # Recursive function
        suite.add_benchmark(
            name="Recursion",
            code="""
λ⟨ fibonacci ⟩(n)
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)

λ⟨ recursion ⟩
    result ← 0
    for i in range(20) {
        result ← result + fibonacci(i)
    }
    return result
""",
            description="Recursive function calls",
            tags=["recursion", "advanced"]
        )
        
        # Memory-intensive operations
        suite.add_benchmark(
            name="Memory_Intensive",
            code="""
λ⟨ memory_intensive ⟩
    arrays ← []
    
    # Create a lot of arrays
    for i in range(100) {
        arr ← []
        for j in range(1000) {
            arr.append(i * j)
        }
        arrays.append(arr)
    }
    
    # Process the arrays
    sum ← 0
    for arr in arrays {
        for val in arr {
            sum ← sum + val
        }
    }
    
    return sum
""",
            description="Memory-intensive operations",
            tags=["memory", "advanced"]
        )
        
        return suite
