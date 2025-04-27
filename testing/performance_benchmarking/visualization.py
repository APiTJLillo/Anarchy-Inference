"""
Enhanced Visualization for Anarchy Inference Benchmarks

This module provides advanced visualization capabilities for benchmark results,
including interactive charts, comparative visualizations, and trend analysis.
"""

import os
import sys
import json
import time
import datetime
import sqlite3
from typing import Dict, List, Any, Optional, Tuple, Union
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from matplotlib.ticker import FuncFormatter

# Add the parent directory to the path so we can import the performance_benchmarking module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from performance_benchmarking.performance_benchmarking import BenchmarkResult, BenchmarkSuite
from performance_benchmarking.ci_integration import BenchmarkDatabase

class BenchmarkVisualizer:
    """Creates visualizations for benchmark results."""
    
    def __init__(self, output_dir: str = None):
        """Initialize the benchmark visualizer.
        
        Args:
            output_dir: Directory to save visualizations
        """
        self.output_dir = output_dir or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "benchmark_reports",
            "visualizations"
        )
        
        # Create the output directory if it doesn't exist
        os.makedirs(self.output_dir, exist_ok=True)
        
        # Set up plotting style
        plt.style.use('seaborn-v0_8-whitegrid')
        sns.set_palette("viridis")
    
    def create_execution_time_chart(self, 
                                   results: Dict[str, BenchmarkResult],
                                   title: str = "Execution Time Comparison",
                                   filename: str = "execution_time_chart.png") -> str:
        """Create a chart comparing execution times.
        
        Args:
            results: Dictionary mapping benchmark names to results
            title: Chart title
            filename: Output filename
            
        Returns:
            Path to the generated chart
        """
        # Extract data
        names = list(results.keys())
        times = [result.avg_execution_time for result in results.values()]
        errors = [result.std_execution_time for result in results.values()]
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create bar chart
        bars = plt.bar(names, times, yerr=errors, capsize=10)
        
        # Add labels and title
        plt.xlabel('Benchmark')
        plt.ylabel('Execution Time (s)')
        plt.title(title)
        
        # Add value labels on top of bars
        for bar in bars:
            height = bar.get_height()
            plt.text(bar.get_x() + bar.get_width()/2., height + 0.001,
                    f'{height:.6f}s',
                    ha='center', va='bottom', rotation=0)
        
        # Adjust layout
        plt.tight_layout()
        plt.xticks(rotation=45, ha='right')
        
        # Save figure
        output_path = os.path.join(self.output_dir, filename)
        plt.savefig(output_path, dpi=300)
        plt.close()
        
        return output_path
    
    def create_memory_usage_chart(self, 
                                 results: Dict[str, BenchmarkResult],
                                 title: str = "Memory Usage Comparison",
                                 filename: str = "memory_usage_chart.png") -> str:
        """Create a chart comparing memory usage.
        
        Args:
            results: Dictionary mapping benchmark names to results
            title: Chart title
            filename: Output filename
            
        Returns:
            Path to the generated chart
        """
        # Extract data
        names = []
        memory_usage = []
        
        for name, result in results.items():
            if result.avg_memory_usage is not None:
                names.append(name)
                memory_usage.append(result.avg_memory_usage)
        
        if not names:
            return None
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create bar chart
        bars = plt.bar(names, memory_usage)
        
        # Add labels and title
        plt.xlabel('Benchmark')
        plt.ylabel('Memory Usage (MB)')
        plt.title(title)
        
        # Add value labels on top of bars
        for bar in bars:
            height = bar.get_height()
            plt.text(bar.get_x() + bar.get_width()/2., height + 0.1,
                    f'{height:.2f} MB',
                    ha='center', va='bottom', rotation=0)
        
        # Adjust layout
        plt.tight_layout()
        plt.xticks(rotation=45, ha='right')
        
        # Save figure
        output_path = os.path.join(self.output_dir, filename)
        plt.savefig(output_path, dpi=300)
        plt.close()
        
        return output_path
    
    def create_token_count_chart(self, 
                                results: Dict[str, BenchmarkResult],
                                title: str = "Token Count Comparison",
                                filename: str = "token_count_chart.png") -> str:
        """Create a chart comparing token counts.
        
        Args:
            results: Dictionary mapping benchmark names to results
            title: Chart title
            filename: Output filename
            
        Returns:
            Path to the generated chart
        """
        # Extract data
        names = []
        token_counts = []
        
        for name, result in results.items():
            if result.avg_token_count is not None:
                names.append(name)
                token_counts.append(result.avg_token_count)
        
        if not names:
            return None
        
        # Create figure
        plt.figure(figsize=(12, 8))
        
        # Create bar chart
        bars = plt.bar(names, token_counts)
        
        # Add labels and title
        plt.xlabel('Benchmark')
        plt.ylabel('Token Count')
        plt.title(title)
        
        # Add value labels on top of bars
        for bar in bars:
            height = bar.get_height()
            plt.text(bar.get_x() + bar.get_width()/2., height + 0.1,
                    f'{int(height)}',
                    ha='center', va='bottom', rotation=0)
        
        # Adjust layout
        plt.tight_layout()
        plt.xticks(rotation=45, ha='right')
        
        # Save figure
        output_path = os.path.join(self.output_dir, filename)
        plt.savefig(output_path, dpi=300)
        plt.close()
        
        return output_path
    
    def create_cross_language_comparison_chart(self, 
                                              results: Dict[str, Dict[str, BenchmarkResult]],
                                              metric: str = "execution_time",
                                              title: str = None,
                                              filename: str = None) -> str:
        """Create a chart comparing results across languages.
        
        Args:
            results: Dictionary mapping benchmark names to dictionaries mapping
                    language names to benchmark results
            metric: Metric to compare ("execution_time", "memory_usage", or "token_count")
            title: Chart title
            filename: Output filename
            
        Returns:
            Path to the generated chart
        """
        # Set default title and filename based on metric
        if title is None:
            if metric == "execution_time":
                title = "Execution Time Comparison Across Languages"
            elif metric == "memory_usage":
                title = "Memory Usage Comparison Across Languages"
            elif metric == "token_count":
                title = "Token Count Comparison Across Languages"
        
        if filename is None:
            filename = f"cross_language_{metric}_chart.png"
        
        # Extract data
        benchmarks = list(results.keys())
        languages = set()
        for benchmark_results in results.values():
            languages.update(benchmark_results.keys())
        languages = sorted(languages)
        
        # Create data structure for grouped bar chart
        data = []
        for benchmark in benchmarks:
            benchmark_data = {"benchmark": benchmark}
            for language in languages:
                if language in results[benchmark]:
                    result = results[benchmark][language]
                    if metric == "execution_time":
                        benchmark_data[language] = result.avg_execution_time
                    elif metric == "memory_usage":
                        benchmark_data[language] = result.avg_memory_usage
                    elif metric == "token_count":
                        benchmark_data[language] = result.avg_token_count
            data.append(benchmark_data)
        
        # Convert to DataFrame
        df = pd.DataFrame(data)
        df = df.set_index("benchmark")
        
        # Create figure
        plt.figure(figsize=(14, 10))
        
        # Create grouped bar chart
        ax = df.plot(kind="bar", figsize=(14, 10))
        
        # Add labels and title
        if metric == "execution_time":
            plt.ylabel("Execution Time (s)")
        elif metric == "memory_usage":
            plt.ylabel("Memory Usage (MB)")
        elif metric == "token_count":
            plt.ylabel("Token Count")
        
        plt.title(title)
        
        # Add legend
        plt.legend(title="Language")
        
        # Adjust layout
        plt.tight_layout()
        plt.xticks(rotation=45, ha='right')
        
        # Save figure
        output_path = os.path.join(self.output_dir, filename)
        plt.savefig(output_path, dpi=300)
        plt.close()
        
        return output_path
    
    def create_historical_trend_chart(self, 
                                     database: BenchmarkDatabase,
                                     benchmark_name: str,
                                     metric: str = "execution_time",
                                     limit: int = 10,
                                     title: str = None,
                                     filename: str = None) -> str:
        """Create a chart showing historical trends for a benchmark.
        
        Args:
            database: Benchmark database
            benchmark_name: Name of the benchmark
            metric: Metric to track ("execution_time", "memory_usage", or "token_count")
            limit: Maximum number of historical results to include
            title: Chart title
            filename: Output filename
            
        Returns:
            Path to the generated chart
        """
        # Set default title and filename based on metric
        if title is None:
            if metric == "execution_time":
                title = f"Execution Time Trend for {benchmark_name}"
            elif metric == "memory_usage":
                title = f"Memory Usage Trend for {benchmark_name}"
            elif metric == "token_count":
                title = f"Token Count Trend for {benchmark_name}"
        
        if filename is None:
            filename = f"historical_{benchmark_name}_{metric}_chart.png"
        
        # Get historical data
        history = database.get_benchmark_history(benchmark_name, limit)
        
        if not history:
            return None
        
        # Extract data
        timestamps = []
        values = []
        commits = []
        
        for entry in history:
            timestamps.append(datetime.datetime.fromisoformat(entry["timestamp"]))
            
            if metric == "execution_time":
                values.append(entry["avg_execution_time"])
            elif metric == "memory_usage":
                values.append(entry["avg_memory_usage"])
            elif metric == "token_count":
                values.append(entry["avg_token_count"])
            
            commits.append(entry["git_commit"][:7] if entry["git_commit"] else "N/A")
        
        # Create figure
        plt.figure(figsize=(14, 8))
        
        # Create line chart
        plt.plot(timestamps, values, marker='o', linestyle='-', linewidth=2, markersize=8)
        
        # Add labels and title
        plt.xlabel('Date')
        if metric == "execution_time":
            plt.ylabel("Execution Time (s)")
        elif metric == "memory_usage":
            plt.ylabel("Memory Usage (MB)")
        elif metric == "token_count":
            plt.ylabel("Token Count")
        
        plt.title(title)
        
        # Add commit labels
        for i, (timestamp, value, commit) in enumerate(zip(timestamps, values, commits)):
            plt.annotate(commit, 
                        (timestamp, value),
                        textcoords="offset points",
                        xytext=(0, 10),
                        ha='center')
        
        # Format x-axis as dates
        plt.gcf().autofmt_xdate()
        
        # Add grid
        plt.grid(True, linestyle='--', alpha=0.7)
        
        # Adjust layout
        plt.tight_layout()
        
        # Save figure
        output_path = os.path.join(self.output_dir, filename)
        plt.savefig(output_path, dpi=300)
        plt.close()
        
        return output_path
    
    def create_regression_heatmap(self, 
                                 alerts: List[Dict[str, Any]],
                                 title: str = "Performance Regression Heatmap",
                                 filename: str = "regression_heatmap.png") -> str:
        """Create a heatmap visualizing performance regressions.
        
        Args:
            alerts: List of regression alerts
            title: Chart title
            filename: Output filename
            
        Returns:
            Path to the generated chart
        """
        if not alerts:
            return None
        
        # Extract data
        benchmarks = set()
        metrics = set()
        data = {}
        
        for alert in alerts:
            benchmark = alert["benchmark_name"]
            metric = alert["metric"]
            percent_change = alert["percent_change"]
            
            benchmarks.add(benchmark)
            metrics.add(metric)
            
            data[(benchmark, metric)] = percent_change
        
        benchmarks = sorted(benchmarks)
        metrics = sorted(metrics)
        
        # Create matrix for heatmap
        matrix = np.zeros((len(benchmarks), len(metrics)))
        
        for i, benchmark in enumerate(benchmarks):
            for j, metric in enumerate(metrics):
                if (benchmark, metric) in data:
                    matrix[i, j] = data[(benchmark, metric)]
        
        # Create figure
        plt.figure(figsize=(12, 10))
        
        # Create heatmap
        ax = sns.heatmap(matrix, 
                        annot=True, 
                        fmt=".2f", 
                        cmap="YlOrRd", 
                        xticklabels=[m.replace("_", " ").title() for m in metrics],
                        yticklabels=benchmarks)
        
        # Add labels and title
        plt.xlabel('Metric')
        plt.ylabel('Benchmark')
        plt.title(title)
        
        # Add colorbar label
        cbar = ax.collections[0].colorbar
        cbar.set_label('Percent Change (%)')
        
        # Adjust layout
        plt.tight_layout()
        
        # Save figure
        output_path = os.path.join(self.output_dir, filename)
        plt.savefig(output_path, dpi=300)
        plt.close()
        
        return output_path
    
    def create_interactive_dashboard(self, 
                                    suite: BenchmarkSuite,
                                    database: BenchmarkDatabase = None,
                                    output_path: str = None) -> str:
        """Create an interactive HTML dashboard for benchmark results.
        
        Args:
            suite: Benchmark suite with results
            database: Benchmark database for historical data
            output_path: Path to save the dashboard
            
        Returns:
            Path to the generated dashboard
        """
        if output_path is None:
            timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
            output_path = os.path.join(self.output_dir, f"dashboard_{timestamp}.html")
        
        # Create HTML content
        html = """<!DOCTYPE html>
<html>
<head>
    <title>Anarchy Inference Benchmark Dashboard</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-plugin-datalabels"></script>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
            color: #333;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
        }
        h1, h2, h3 {
            color: #2c3e50;
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 1px solid #eee;
        }
        .chart-container {
            margin-bottom: 30px;
            padding: 15px;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 1px 5px rgba(0, 0, 0, 0.05);
        }
        .chart-row {
            display: flex;
            flex-wrap: wrap;
            margin: 0 -10px;
        }
        .chart-col {
            flex: 1;
            min-width: 300px;
            padding: 0 10px;
            margin-bottom: 20px;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 20px;
        }
        th, td {
            padding: 12px 15px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        th {
            background-color: #f8f9fa;
            font-weight: 600;
        }
        tr:hover {
            background-color: #f9f9f9;
        }
        .summary-cards {
            display: flex;
            flex-wrap: wrap;
            margin: 0 -10px 20px;
        }
        .summary-card {
            flex: 1;
            min-width: 200px;
            background-color: white;
            border-radius: 8px;
            padding: 15px;
            margin: 0 10px 20px;
            box-shadow: 0 1px 5px rgba(0, 0, 0, 0.05);
        }
        .summary-card h3 {
            margin-top: 0;
            font-size: 16px;
            color: #666;
        }
        .summary-card .value {
            font-size: 24px;
            font-weight: bold;
            color: #2c3e50;
        }
        .tabs {
            display: flex;
            margin-bottom: 20px;
            border-bottom: 1px solid #ddd;
        }
        .tab {
            padding: 10px 20px;
            cursor: pointer;
            border-bottom: 2px solid transparent;
        }
        .tab.active {
            border-bottom: 2px solid #2c3e50;
            font-weight: bold;
        }
        .tab-content {
            display: none;
        }
        .tab-content.active {
            display: block;
        }
        .footer {
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            text-align: center;
            color: #666;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Anarchy Inference Benchmark Dashboard</h1>
            <div>
                <span id="timestamp"></span>
            </div>
        </div>
        
        <div class="summary-cards">
            <div class="summary-card">
                <h3>Total Benchmarks</h3>
                <div class="value" id="total-benchmarks"></div>
            </div>
            <div class="summary-card">
                <h3>Avg Execution Time</h3>
                <div class="value" id="avg-execution-time"></div>
            </div>
            <div class="summary-card">
                <h3>Avg Memory Usage</h3>
                <div class="value" id="avg-memory-usage"></div>
            </div>
            <div class="summary-card">
                <h3>Avg Token Count</h3>
                <div class="value" id="avg-token-count"></div>
            </div>
        </div>
        
        <div class="tabs">
            <div class="tab active" data-tab="overview">Overview</div>
            <div class="tab" data-tab="execution-time">Execution Time</div>
            <div class="tab" data-tab="memory-usage">Memory Usage</div>
            <div class="tab" data-tab="token-count">Token Count</div>
            <div class="tab" data-tab="raw-data">Raw Data</div>
        </div>
        
        <div class="tab-content active" id="overview">
            <div class="chart-row">
                <div class="chart-col">
                    <div class="chart-container">
                        <h2>Execution Time Overview</h2>
                        <canvas id="execution-time-chart"></canvas>
                    </div>
                </div>
                <div class="chart-col">
                    <div class="chart-container">
                        <h2>Memory Usage Overview</h2>
                        <canvas id="memory-usage-chart"></canvas>
                    </div>
                </div>
            </div>
            <div class="chart-row">
                <div class="chart-col">
                    <div class="chart-container">
                        <h2>Token Count Overview</h2>
                        <canvas id="token-count-chart"></canvas>
                    </div>
                </div>
                <div class="chart-col">
                    <div class="chart-container">
                        <h2>Benchmark Distribution</h2>
                        <canvas id="benchmark-distribution-chart"></canvas>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="tab-content" id="execution-time">
            <div class="chart-container">
                <h2>Detailed Execution Time Analysis</h2>
                <canvas id="detailed-execution-time-chart"></canvas>
            </div>
            <div class="chart-row">
                <div class="chart-col">
                    <div class="chart-container">
                        <h2>Execution Time Distribution</h2>
                        <canvas id="execution-time-distribution-chart"></canvas>
                    </div>
                </div>
                <div class="chart-col">
                    <div class="chart-container">
                        <h2>Execution Time Variability</h2>
                        <canvas id="execution-time-variability-chart"></canvas>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="tab-content" id="memory-usage">
            <div class="chart-container">
                <h2>Detailed Memory Usage Analysis</h2>
                <canvas id="detailed-memory-usage-chart"></canvas>
            </div>
        </div>
        
        <div class="tab-content" id="token-count">
            <div class="chart-container">
                <h2>Detailed Token Count Analysis</h2>
                <canvas id="detailed-token-count-chart"></canvas>
            </div>
        </div>
        
        <div class="tab-content" id="raw-data">
            <h2>Raw Benchmark Data</h2>
            <table id="benchmark-table">
                <thead>
                    <tr>
                        <th>Benchmark</th>
                        <th>Avg Time (s)</th>
                        <th>Min Time (s)</th>
                        <th>Max Time (s)</th>
                        <th>Std Dev</th>
                        <th>Memory (MB)</th>
                        <th>Tokens</th>
                    </tr>
                </thead>
                <tbody>
                    <!-- Data will be inserted here -->
                </tbody>
            </table>
        </div>
        
        <div class="footer">
            <p>Generated by Anarchy Inference Benchmark Suite</p>
        </div>
    </div>
    
    <script>
        // Benchmark data
        const benchmarkData = BENCHMARK_DATA_PLACEHOLDER;
        
        // Update timestamp
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
        
        // Update summary cards
        document.getElementById('total-benchmarks').textContent = Object.keys(benchmarkData).length;
        
        let totalExecutionTime = 0;
        let totalMemoryUsage = 0;
        let totalTokenCount = 0;
        let memoryCount = 0;
        let tokenCount = 0;
        
        for (const name in benchmarkData) {
            const result = benchmarkData[name];
            totalExecutionTime += result.avg_execution_time;
            
            if (result.avg_memory_usage) {
                totalMemoryUsage += result.avg_memory_usage;
                memoryCount++;
            }
            
            if (result.avg_token_count) {
                totalTokenCount += result.avg_token_count;
                tokenCount++;
            }
        }
        
        const avgExecutionTime = totalExecutionTime / Object.keys(benchmarkData).length;
        document.getElementById('avg-execution-time').textContent = avgExecutionTime.toFixed(6) + ' s';
        
        if (memoryCount > 0) {
            const avgMemoryUsage = totalMemoryUsage / memoryCount;
            document.getElementById('avg-memory-usage').textContent = avgMemoryUsage.toFixed(2) + ' MB';
        } else {
            document.getElementById('avg-memory-usage').textContent = 'N/A';
        }
        
        if (tokenCount > 0) {
            const avgTokenCount = totalTokenCount / tokenCount;
            document.getElementById('avg-token-count').textContent = Math.round(avgTokenCount);
        } else {
            document.getElementById('avg-token-count').textContent = 'N/A';
        }
        
        // Fill the benchmark table
        const tableBody = document.querySelector('#benchmark-table tbody');
        for (const name in benchmarkData) {
            const result = benchmarkData[name];
            const row = document.createElement('tr');
            
            row.innerHTML = `
                <td>${name}</td>
                <td>${result.avg_execution_time.toFixed(6)}</td>
                <td>${result.min_execution_time.toFixed(6)}</td>
                <td>${result.max_execution_time.toFixed(6)}</td>
                <td>${result.std_execution_time.toFixed(6)}</td>
                <td>${result.avg_memory_usage ? result.avg_memory_usage.toFixed(2) : 'N/A'}</td>
                <td>${result.avg_token_count ? Math.round(result.avg_token_count) : 'N/A'}</td>
            `;
            
            tableBody.appendChild(row);
        }
        
        // Create charts
        const names = Object.keys(benchmarkData);
        const executionTimes = names.map(name => benchmarkData[name].avg_execution_time);
        const memoryUsages = names.map(name => benchmarkData[name].avg_memory_usage || 0);
        const tokenCounts = names.map(name => benchmarkData[name].avg_token_count || 0);
        
        // Execution time chart
        new Chart(document.getElementById('execution-time-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Execution Time (s)',
                    data: executionTimes,
                    backgroundColor: 'rgba(54, 162, 235, 0.5)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Execution Time (s)'
                        }
                    }
                }
            }
        });
        
        // Memory usage chart
        new Chart(document.getElementById('memory-usage-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Memory Usage (MB)',
                    data: memoryUsages,
                    backgroundColor: 'rgba(255, 99, 132, 0.5)',
                    borderColor: 'rgba(255, 99, 132, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Memory Usage (MB)'
                        }
                    }
                }
            }
        });
        
        // Token count chart
        new Chart(document.getElementById('token-count-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Token Count',
                    data: tokenCounts,
                    backgroundColor: 'rgba(75, 192, 192, 0.5)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Token Count'
                        }
                    }
                }
            }
        });
        
        // Benchmark distribution chart
        new Chart(document.getElementById('benchmark-distribution-chart'), {
            type: 'pie',
            data: {
                labels: names,
                datasets: [{
                    data: names.map(() => 1),
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.5)',
                        'rgba(54, 162, 235, 0.5)',
                        'rgba(255, 206, 86, 0.5)',
                        'rgba(75, 192, 192, 0.5)',
                        'rgba(153, 102, 255, 0.5)',
                        'rgba(255, 159, 64, 0.5)',
                        'rgba(199, 199, 199, 0.5)',
                        'rgba(83, 102, 255, 0.5)',
                        'rgba(40, 159, 64, 0.5)',
                        'rgba(210, 199, 199, 0.5)',
                    ]
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    legend: {
                        position: 'right',
                    }
                }
            }
        });
        
        // Detailed execution time chart
        new Chart(document.getElementById('detailed-execution-time-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Avg Execution Time (s)',
                    data: names.map(name => benchmarkData[name].avg_execution_time),
                    backgroundColor: 'rgba(54, 162, 235, 0.5)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }, {
                    label: 'Min Execution Time (s)',
                    data: names.map(name => benchmarkData[name].min_execution_time),
                    backgroundColor: 'rgba(75, 192, 192, 0.5)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }, {
                    label: 'Max Execution Time (s)',
                    data: names.map(name => benchmarkData[name].max_execution_time),
                    backgroundColor: 'rgba(255, 99, 132, 0.5)',
                    borderColor: 'rgba(255, 99, 132, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Execution Time (s)'
                        }
                    }
                }
            }
        });
        
        // Detailed memory usage chart
        new Chart(document.getElementById('detailed-memory-usage-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Memory Usage (MB)',
                    data: names.map(name => benchmarkData[name].avg_memory_usage || 0),
                    backgroundColor: 'rgba(255, 99, 132, 0.5)',
                    borderColor: 'rgba(255, 99, 132, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Memory Usage (MB)'
                        }
                    }
                }
            }
        });
        
        // Detailed token count chart
        new Chart(document.getElementById('detailed-token-count-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Token Count',
                    data: names.map(name => benchmarkData[name].avg_token_count || 0),
                    backgroundColor: 'rgba(75, 192, 192, 0.5)',
                    borderColor: 'rgba(75, 192, 192, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Token Count'
                        }
                    }
                }
            }
        });
        
        // Execution time distribution chart
        new Chart(document.getElementById('execution-time-distribution-chart'), {
            type: 'scatter',
            data: {
                datasets: [{
                    label: 'Execution Time (s)',
                    data: names.map((name, index) => ({
                        x: index,
                        y: benchmarkData[name].avg_execution_time
                    })),
                    backgroundColor: 'rgba(54, 162, 235, 0.5)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    x: {
                        type: 'linear',
                        position: 'bottom',
                        title: {
                            display: true,
                            text: 'Benchmark Index'
                        },
                        ticks: {
                            callback: function(value) {
                                return names[value] || '';
                            }
                        }
                    },
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Execution Time (s)'
                        }
                    }
                }
            }
        });
        
        // Execution time variability chart
        new Chart(document.getElementById('execution-time-variability-chart'), {
            type: 'bar',
            data: {
                labels: names,
                datasets: [{
                    label: 'Std Dev Execution Time (s)',
                    data: names.map(name => benchmarkData[name].std_execution_time),
                    backgroundColor: 'rgba(153, 102, 255, 0.5)',
                    borderColor: 'rgba(153, 102, 255, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Std Dev Execution Time (s)'
                        }
                    }
                }
            }
        });
        
        // Tab switching
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                // Remove active class from all tabs and tab contents
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
                
                // Add active class to clicked tab and corresponding content
                tab.classList.add('active');
                document.getElementById(tab.dataset.tab).classList.add('active');
            });
        });
    </script>
</body>
</html>
"""
        
        # Convert benchmark results to JSON
        benchmark_data = {}
        for name, result in suite.results.items():
            benchmark_data[name] = result.to_dict()
        
        # Replace placeholder with actual data
        html = html.replace('BENCHMARK_DATA_PLACEHOLDER', json.dumps(benchmark_data))
        
        # Write to file
        with open(output_path, 'w') as f:
            f.write(html)
        
        return output_path


# Example usage
if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Create visualizations for benchmark results")
    parser.add_argument("--suite", help="Path to benchmark suite file")
    parser.add_argument("--output-dir", help="Directory to save visualizations")
    
    args = parser.parse_args()
    
    if args.suite:
        from performance_benchmarking.performance_benchmarking import BenchmarkSuite
        
        # Load benchmark suite
        suite = BenchmarkSuite.load(args.suite)
        
        # Create visualizer
        visualizer = BenchmarkVisualizer(args.output_dir)
        
        # Create charts
        execution_time_chart = visualizer.create_execution_time_chart(suite.results)
        memory_usage_chart = visualizer.create_memory_usage_chart(suite.results)
        token_count_chart = visualizer.create_token_count_chart(suite.results)
        
        # Create dashboard
        dashboard_path = visualizer.create_interactive_dashboard(suite)
        
        print(f"Visualizations generated:")
        print(f"- Execution Time Chart: {execution_time_chart}")
        print(f"- Memory Usage Chart: {memory_usage_chart}")
        print(f"- Token Count Chart: {token_count_chart}")
        print(f"- Interactive Dashboard: {dashboard_path}")
    else:
        print("No benchmark suite specified")
        sys.exit(1)
