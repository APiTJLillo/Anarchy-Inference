"""
Cross-Language Benchmarking for Anarchy Inference

This module provides functionality to compare Anarchy Inference performance
with other programming languages.
"""

import os
import sys
import json
import time
import subprocess
import tempfile
import statistics
from typing import Dict, List, Any, Optional, Tuple, Union
import matplotlib.pyplot as plt
import numpy as np

# Add the parent directory to the path so we can import the performance_benchmarking module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from performance_benchmarking.performance_benchmarking import BenchmarkResult, BenchmarkSuite, BenchmarkReporter

class LanguageAdapter:
    """Base class for language adapters that run benchmarks in different languages."""
    
    def __init__(self, name: str):
        """Initialize a language adapter.
        
        Args:
            name: Name of the language
        """
        self.name = name
    
    def run_benchmark(self, code: str, setup_code: str = "", teardown_code: str = "") -> BenchmarkResult:
        """Run a benchmark in this language.
        
        Args:
            code: Code to benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            
        Returns:
            Benchmark result
        """
        raise NotImplementedError("Subclasses must implement run_benchmark")
    
    def count_tokens(self, code: str) -> int:
        """Count the number of tokens in code.
        
        Args:
            code: Code to count tokens in
            
        Returns:
            Number of tokens
        """
        raise NotImplementedError("Subclasses must implement count_tokens")


class AnarchyAdapter(LanguageAdapter):
    """Adapter for running Anarchy Inference benchmarks."""
    
    def __init__(self, interpreter_path: str = None):
        """Initialize an Anarchy Inference adapter.
        
        Args:
            interpreter_path: Path to the Anarchy Inference interpreter
        """
        super().__init__("Anarchy Inference")
        self.interpreter_path = interpreter_path or "anarchy"
    
    def run_benchmark(self, code: str, setup_code: str = "", teardown_code: str = "") -> BenchmarkResult:
        """Run a benchmark in Anarchy Inference.
        
        Args:
            code: Code to benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            
        Returns:
            Benchmark result
        """
        # Create a temporary file for the benchmark code
        with tempfile.NamedTemporaryFile(suffix=".anarchy", delete=False) as f:
            # Write setup code
            if setup_code:
                f.write(setup_code.encode())
                f.write(b"\n")
            
            # Write benchmark code with timing
            f.write(b"ι start_time = Date.now();\n")
            f.write(code.encode())
            f.write(b"\nι end_time = Date.now();\n")
            f.write(b"⌽ (end_time - start_time);\n")
            
            # Write teardown code
            if teardown_code:
                f.write(teardown_code.encode())
                f.write(b"\n")
            
            temp_file = f.name
        
        try:
            # Run the benchmark multiple times
            execution_times = []
            for _ in range(5):  # Run 5 iterations
                # Run the benchmark and capture output
                result = subprocess.run(
                    [self.interpreter_path, temp_file],
                    capture_output=True,
                    text=True
                )
                
                # Parse the execution time from the output
                try:
                    execution_time = float(result.stdout.strip()) / 1000.0  # Convert ms to seconds
                    execution_times.append(execution_time)
                except ValueError:
                    print(f"Error parsing execution time: {result.stdout}")
            
            # Count tokens
            token_count = self.count_tokens(code)
            
            # Create and return the benchmark result
            return BenchmarkResult(
                name="anarchy_benchmark",
                execution_times=execution_times,
                token_counts=[token_count]
            )
        finally:
            # Clean up the temporary file
            os.unlink(temp_file)
    
    def count_tokens(self, code: str) -> int:
        """Count the number of tokens in Anarchy Inference code.
        
        Args:
            code: Code to count tokens in
            
        Returns:
            Number of tokens
        """
        # Create a temporary file for the code
        with tempfile.NamedTemporaryFile(suffix=".anarchy", delete=False) as f:
            f.write(code.encode())
            temp_file = f.name
        
        try:
            # Run the token counter
            result = subprocess.run(
                [self.interpreter_path, "--count-tokens", temp_file],
                capture_output=True,
                text=True
            )
            
            # Parse the token count from the output
            try:
                return int(result.stdout.strip())
            except ValueError:
                print(f"Error parsing token count: {result.stdout}")
                return 0
        finally:
            # Clean up the temporary file
            os.unlink(temp_file)


class PythonAdapter(LanguageAdapter):
    """Adapter for running Python benchmarks."""
    
    def __init__(self, python_path: str = None):
        """Initialize a Python adapter.
        
        Args:
            python_path: Path to the Python interpreter
        """
        super().__init__("Python")
        self.python_path = python_path or "python3"
    
    def run_benchmark(self, code: str, setup_code: str = "", teardown_code: str = "") -> BenchmarkResult:
        """Run a benchmark in Python.
        
        Args:
            code: Code to benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            
        Returns:
            Benchmark result
        """
        # Create a temporary file for the benchmark code
        with tempfile.NamedTemporaryFile(suffix=".py", delete=False) as f:
            # Write imports
            f.write(b"import time\n")
            
            # Write setup code
            if setup_code:
                f.write(setup_code.encode())
                f.write(b"\n")
            
            # Write benchmark code with timing
            f.write(b"start_time = time.time()\n")
            f.write(code.encode())
            f.write(b"\nend_time = time.time()\n")
            f.write(b"print(end_time - start_time)\n")
            
            # Write teardown code
            if teardown_code:
                f.write(teardown_code.encode())
                f.write(b"\n")
            
            temp_file = f.name
        
        try:
            # Run the benchmark multiple times
            execution_times = []
            for _ in range(5):  # Run 5 iterations
                # Run the benchmark and capture output
                result = subprocess.run(
                    [self.python_path, temp_file],
                    capture_output=True,
                    text=True
                )
                
                # Parse the execution time from the output
                try:
                    execution_time = float(result.stdout.strip())
                    execution_times.append(execution_time)
                except ValueError:
                    print(f"Error parsing execution time: {result.stdout}")
            
            # Count tokens
            token_count = self.count_tokens(code)
            
            # Create and return the benchmark result
            return BenchmarkResult(
                name="python_benchmark",
                execution_times=execution_times,
                token_counts=[token_count]
            )
        finally:
            # Clean up the temporary file
            os.unlink(temp_file)
    
    def count_tokens(self, code: str) -> int:
        """Count the number of tokens in Python code.
        
        Args:
            code: Code to count tokens in
            
        Returns:
            Number of tokens
        """
        # Create a temporary file for the code
        with tempfile.NamedTemporaryFile(suffix=".py", delete=False) as f:
            f.write(b"import tokenize\n")
            f.write(b"import io\n")
            f.write(b"import sys\n\n")
            f.write(b"def count_tokens(code):\n")
            f.write(b"    token_count = 0\n")
            f.write(b"    for token in tokenize.tokenize(io.BytesIO(code.encode('utf-8')).readline):\n")
            f.write(b"        token_count += 1\n")
            f.write(b"    return token_count\n\n")
            f.write(b"code = '''\n")
            f.write(code.encode())
            f.write(b"'''\n\n")
            f.write(b"print(count_tokens(code))\n")
            
            temp_file = f.name
        
        try:
            # Run the token counter
            result = subprocess.run(
                [self.python_path, temp_file],
                capture_output=True,
                text=True
            )
            
            # Parse the token count from the output
            try:
                return int(result.stdout.strip())
            except ValueError:
                print(f"Error parsing token count: {result.stdout}")
                return 0
        finally:
            # Clean up the temporary file
            os.unlink(temp_file)


class JavaScriptAdapter(LanguageAdapter):
    """Adapter for running JavaScript benchmarks."""
    
    def __init__(self, node_path: str = None):
        """Initialize a JavaScript adapter.
        
        Args:
            node_path: Path to the Node.js interpreter
        """
        super().__init__("JavaScript")
        self.node_path = node_path or "node"
    
    def run_benchmark(self, code: str, setup_code: str = "", teardown_code: str = "") -> BenchmarkResult:
        """Run a benchmark in JavaScript.
        
        Args:
            code: Code to benchmark
            setup_code: Code to run before the benchmark
            teardown_code: Code to run after the benchmark
            
        Returns:
            Benchmark result
        """
        # Create a temporary file for the benchmark code
        with tempfile.NamedTemporaryFile(suffix=".js", delete=False) as f:
            # Write setup code
            if setup_code:
                f.write(setup_code.encode())
                f.write(b"\n")
            
            # Write benchmark code with timing
            f.write(b"const startTime = Date.now();\n")
            f.write(code.encode())
            f.write(b"\nconst endTime = Date.now();\n")
            f.write(b"console.log((endTime - startTime) / 1000);\n")  # Convert ms to seconds
            
            # Write teardown code
            if teardown_code:
                f.write(teardown_code.encode())
                f.write(b"\n")
            
            temp_file = f.name
        
        try:
            # Run the benchmark multiple times
            execution_times = []
            for _ in range(5):  # Run 5 iterations
                # Run the benchmark and capture output
                result = subprocess.run(
                    [self.node_path, temp_file],
                    capture_output=True,
                    text=True
                )
                
                # Parse the execution time from the output
                try:
                    execution_time = float(result.stdout.strip())
                    execution_times.append(execution_time)
                except ValueError:
                    print(f"Error parsing execution time: {result.stdout}")
            
            # Count tokens
            token_count = self.count_tokens(code)
            
            # Create and return the benchmark result
            return BenchmarkResult(
                name="javascript_benchmark",
                execution_times=execution_times,
                token_counts=[token_count]
            )
        finally:
            # Clean up the temporary file
            os.unlink(temp_file)
    
    def count_tokens(self, code: str) -> int:
        """Count the number of tokens in JavaScript code.
        
        Args:
            code: Code to count tokens in
            
        Returns:
            Number of tokens
        """
        # Create a temporary file for the code
        with tempfile.NamedTemporaryFile(suffix=".js", delete=False) as f:
            f.write(b"const acorn = require('acorn');\n\n")
            f.write(b"function countTokens(code) {\n")
            f.write(b"  const tokens = [];\n")
            f.write(b"  acorn.tokenizer(code, { ecmaVersion: 2020 })\n")
            f.write(b"    .forEach(token => tokens.push(token));\n")
            f.write(b"  return tokens.length;\n")
            f.write(b"}\n\n")
            f.write(b"const code = `\n")
            f.write(code.encode())
            f.write(b"`;\n\n")
            f.write(b"console.log(countTokens(code));\n")
            
            temp_file = f.name
        
        try:
            # Install acorn if not already installed
            subprocess.run(
                ["npm", "install", "acorn", "--no-save"],
                capture_output=True,
                text=True
            )
            
            # Run the token counter
            result = subprocess.run(
                [self.node_path, temp_file],
                capture_output=True,
                text=True
            )
            
            # Parse the token count from the output
            try:
                return int(result.stdout.strip())
            except ValueError:
                print(f"Error parsing token count: {result.stdout}")
                return 0
        finally:
            # Clean up the temporary file
            os.unlink(temp_file)


class CrossLanguageBenchmark:
    """Runs benchmarks across multiple languages and compares results."""
    
    def __init__(self, languages: List[LanguageAdapter] = None):
        """Initialize a cross-language benchmark.
        
        Args:
            languages: List of language adapters to use
        """
        self.languages = languages or [
            AnarchyAdapter(),
            PythonAdapter(),
            JavaScriptAdapter()
        ]
    
    def run_benchmark(self, 
                     name: str, 
                     implementations: Dict[str, Dict[str, str]]) -> Dict[str, BenchmarkResult]:
        """Run a benchmark across multiple languages.
        
        Args:
            name: Name of the benchmark
            implementations: Dictionary mapping language names to dictionaries containing
                            'code', 'setup_code', and 'teardown_code' for each language
            
        Returns:
            Dictionary mapping language names to benchmark results
        """
        results = {}
        
        for language in self.languages:
            if language.name in implementations:
                print(f"Running {name} benchmark in {language.name}...")
                
                implementation = implementations[language.name]
                result = language.run_benchmark(
                    code=implementation.get("code", ""),
                    setup_code=implementation.get("setup_code", ""),
                    teardown_code=implementation.get("teardown_code", "")
                )
                
                # Update the result name
                result.name = f"{language.name}_{name}"
                
                results[language.name] = result
            else:
                print(f"No implementation for {name} benchmark in {language.name}")
        
        return results
    
    def compare_results(self, results: Dict[str, Dict[str, BenchmarkResult]]) -> Dict[str, Any]:
        """Compare benchmark results across languages.
        
        Args:
            results: Dictionary mapping benchmark names to dictionaries mapping
                    language names to benchmark results
            
        Returns:
            Dictionary with comparison metrics
        """
        comparison = {}
        
        for benchmark_name, language_results in results.items():
            comparison[benchmark_name] = {}
            
            # Find the baseline language (Anarchy Inference)
            baseline_language = "Anarchy Inference"
            if baseline_language not in language_results:
                # If Anarchy Inference is not available, use the first language as baseline
                baseline_language = next(iter(language_results.keys()))
            
            baseline_result = language_results[baseline_language]
            
            # Compare each language to the baseline
            for language_name, result in language_results.items():
                if language_name == baseline_language:
                    continue
                
                # Calculate time difference
                baseline_time = baseline_result.avg_execution_time
                current_time = result.avg_execution_time
                time_diff_pct = ((current_time - baseline_time) / baseline_time) * 100
                
                # Calculate token difference
                baseline_tokens = baseline_result.avg_token_count
                current_tokens = result.avg_token_count
                token_diff_pct = ((current_tokens - baseline_tokens) / baseline_tokens) * 100
                
                comparison[benchmark_name][language_name] = {
                    "time_diff_pct": time_diff_pct,
                    "token_diff_pct": token_diff_pct,
                    "baseline_time": baseline_time,
                    "current_time": current_time,
                    "baseline_tokens": baseline_tokens,
                    "current_tokens": current_tokens
                }
        
        return comparison
    
    def generate_report(self, 
                       results: Dict[str, Dict[str, BenchmarkResult]],
                       comparison: Dict[str, Any],
                       output_dir: str = None) -> str:
        """Generate a report of cross-language benchmark results.
        
        Args:
            results: Dictionary mapping benchmark names to dictionaries mapping
                    language names to benchmark results
            comparison: Dictionary with comparison metrics
            output_dir: Directory to save the report
            
        Returns:
            Path to the generated report
        """
        # Create a benchmark reporter
        reporter = BenchmarkReporter(output_dir)
        
        # Create HTML content
        html = """<!DOCTYPE html>
<html>
<head>
    <title>Cross-Language Benchmark Report</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            color: #333;
        }
        h1, h2, h3 {
            color: #2c3e50;
        }
        .summary {
            background-color: #f8f9fa;
            border-radius: 5px;
            padding: 15px;
            margin-bottom: 20px;
        }
        table {
            border-collapse: collapse;
            width: 100%;
            margin-bottom: 20px;
        }
        th, td {
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }
        th {
            background-color: #f2f2f2;
        }
        tr:nth-child(even) {
            background-color: #f9f9f9;
        }
        .positive {
            color: green;
        }
        .negative {
            color: red;
        }
        .chart-container {
            width: 100%;
            height: 400px;
            margin-bottom: 20px;
        }
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <h1>Cross-Language Benchmark Report</h1>
    
    <div class="summary">
        <p>This report compares the performance of Anarchy Inference with other programming languages.</p>
    </div>
"""
        
        # Add benchmark results
        for benchmark_name, language_results in results.items():
            html += f"""
    <h2>{benchmark_name}</h2>
    <table>
        <tr>
            <th>Language</th>
            <th>Avg Time (s)</th>
            <th>Min Time (s)</th>
            <th>Max Time (s)</th>
            <th>Std Dev Time (s)</th>
            <th>Avg Tokens</th>
        </tr>
"""
            
            for language_name, result in language_results.items():
                html += f"""
        <tr>
            <td>{language_name}</td>
            <td>{result.avg_execution_time:.6f}</td>
            <td>{result.min_execution_time:.6f}</td>
            <td>{result.max_execution_time:.6f}</td>
            <td>{result.std_execution_time:.6f}</td>
            <td>{result.avg_token_count:.0f}</td>
        </tr>"""
            
            html += """
    </table>
    
    <div class="chart-container">
        <canvas id="timeChart_{benchmark_name}"></canvas>
    </div>
    
    <div class="chart-container">
        <canvas id="tokenChart_{benchmark_name}"></canvas>
    </div>
"""
        
        # Add comparison
        html += """
    <h2>Comparison to Anarchy Inference</h2>
    <table>
        <tr>
            <th>Benchmark</th>
            <th>Language</th>
            <th>Time Diff (%)</th>
            <th>Token Diff (%)</th>
            <th>Anarchy Time (s)</th>
            <th>Language Time (s)</th>
            <th>Anarchy Tokens</th>
            <th>Language Tokens</th>
        </tr>
"""
        
        for benchmark_name, language_comparisons in comparison.items():
            for language_name, comp in language_comparisons.items():
                time_class = "positive" if comp["time_diff_pct"] <= 0 else "negative"
                token_class = "positive" if comp["token_diff_pct"] <= 0 else "negative"
                
                html += f"""
        <tr>
            <td>{benchmark_name}</td>
            <td>{language_name}</td>
            <td class="{time_class}">{comp["time_diff_pct"]:.2f}%</td>
            <td class="{token_class}">{comp["token_diff_pct"]:.2f}%</td>
            <td>{comp["baseline_time"]:.6f}</td>
            <td>{comp["current_time"]:.6f}</td>
            <td>{comp["baseline_tokens"]:.0f}</td>
            <td>{comp["current_tokens"]:.0f}</td>
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
        // Time charts
"""
        
        for benchmark_name, language_results in results.items():
            html += f"""
        new Chart(document.getElementById('timeChart_{benchmark_name}'), {{
            type: 'bar',
            data: {{
                labels: {json.dumps(list(language_results.keys()))},
                datasets: [{{
                    label: 'Execution Time (s)',
                    data: {json.dumps([result.avg_execution_time for result in language_results.values()])},
                    backgroundColor: 'rgba(54, 162, 235, 0.5)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        title: {{
                            display: true,
                            text: 'Execution Time (s)'
                        }}
                    }}
                }}
            }}
        }});
        
        new Chart(document.getElementById('tokenChart_{benchmark_name}'), {{
            type: 'bar',
            data: {{
                labels: {json.dumps(list(language_results.keys()))},
                datasets: [{{
                    label: 'Token Count',
                    data: {json.dumps([result.avg_token_count for result in language_results.values()])},
                    backgroundColor: 'rgba(255, 99, 132, 0.5)',
                    borderColor: 'rgba(255, 99, 132, 1)',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        title: {{
                            display: true,
                            text: 'Token Count'
                        }}
                    }}
                }}
            }}
        }});
"""
        
        # Add comparison chart
        html += """
        // Comparison chart
        new Chart(document.getElementById('comparisonChart'), {
            type: 'bar',
            data: {
                labels: [],
                datasets: [
                    {
                        label: 'Time Difference (%)',
                        data: [],
                        backgroundColor: 'rgba(54, 162, 235, 0.5)',
                        borderColor: 'rgba(54, 162, 235, 1)',
                        borderWidth: 1
                    },
                    {
                        label: 'Token Difference (%)',
                        data: [],
                        backgroundColor: 'rgba(255, 99, 132, 0.5)',
                        borderColor: 'rgba(255, 99, 132, 1)',
                        borderWidth: 1
                    }
                ]
            },
            options: {
                responsive: true,
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
    </script>
</body>
</html>
"""
        
        # Write to file
        import datetime
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(output_dir or ".", f"cross_language_benchmark_report_{timestamp}.html")
        
        with open(output_path, 'w') as f:
            f.write(html)
        
        return output_path


# Example usage
if __name__ == "__main__":
    # Create language adapters
    anarchy = AnarchyAdapter()
    python = PythonAdapter()
    javascript = JavaScriptAdapter()
    
    # Create cross-language benchmark
    benchmark = CrossLanguageBenchmark([anarchy, python, javascript])
    
    # Define benchmark implementations
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
        "fibonacci": benchmark.run_benchmark("fibonacci", fibonacci_implementations)
    }
    
    # Compare results
    comparison = benchmark.compare_results(results)
    
    # Generate report
    report_path = benchmark.generate_report(results, comparison)
    print(f"Report generated: {report_path}")
