"""
Continuous Integration Support for Anarchy Inference Benchmarks

This module provides functionality to integrate benchmarks with CI/CD pipelines,
track historical performance, and detect regressions.
"""

import os
import sys
import json
import time
import datetime
import sqlite3
import statistics
from typing import Dict, List, Any, Optional, Tuple, Union
import matplotlib.pyplot as plt
import numpy as np

# Add the parent directory to the path so we can import the performance_benchmarking module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from performance_benchmarking.performance_benchmarking import BenchmarkResult, BenchmarkSuite, BenchmarkComparison

class BenchmarkDatabase:
    """Manages storage and retrieval of benchmark results."""
    
    def __init__(self, db_path: str = None):
        """Initialize the benchmark database.
        
        Args:
            db_path: Path to the SQLite database file
        """
        self.db_path = db_path or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "benchmark_data",
            "benchmark_history.db"
        )
        
        # Create the directory if it doesn't exist
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        
        # Initialize the database
        self._init_db()
    
    def _init_db(self) -> None:
        """Initialize the database schema."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Create tables
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS benchmark_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            git_commit TEXT,
            git_branch TEXT,
            system_info TEXT
        )
        ''')
        
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS benchmark_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            avg_execution_time REAL NOT NULL,
            min_execution_time REAL NOT NULL,
            max_execution_time REAL NOT NULL,
            std_execution_time REAL NOT NULL,
            avg_memory_usage REAL,
            avg_token_count REAL,
            raw_data TEXT NOT NULL,
            FOREIGN KEY (run_id) REFERENCES benchmark_runs(id)
        )
        ''')
        
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS regression_alerts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            benchmark_name TEXT NOT NULL,
            previous_value REAL NOT NULL,
            current_value REAL NOT NULL,
            percent_change REAL NOT NULL,
            metric TEXT NOT NULL,
            severity TEXT NOT NULL,
            acknowledged INTEGER DEFAULT 0
        )
        ''')
        
        conn.commit()
        conn.close()
    
    def store_benchmark_run(self, 
                           suite: BenchmarkSuite, 
                           git_commit: str = None, 
                           git_branch: str = None,
                           system_info: Dict[str, str] = None) -> int:
        """Store a benchmark run in the database.
        
        Args:
            suite: Benchmark suite with results
            git_commit: Git commit hash
            git_branch: Git branch name
            system_info: System information
            
        Returns:
            ID of the benchmark run
        """
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Insert benchmark run
        cursor.execute(
            "INSERT INTO benchmark_runs (timestamp, git_commit, git_branch, system_info) VALUES (?, ?, ?, ?)",
            (
                datetime.datetime.now().isoformat(),
                git_commit,
                git_branch,
                json.dumps(system_info or {})
            )
        )
        
        run_id = cursor.lastrowid
        
        # Insert benchmark results
        for name, result in suite.results.items():
            cursor.execute(
                """
                INSERT INTO benchmark_results 
                (run_id, name, avg_execution_time, min_execution_time, max_execution_time, 
                std_execution_time, avg_memory_usage, avg_token_count, raw_data)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    run_id,
                    name,
                    result.avg_execution_time,
                    result.min_execution_time,
                    result.max_execution_time,
                    result.std_execution_time,
                    result.avg_memory_usage,
                    result.avg_token_count,
                    json.dumps(result.to_dict())
                )
            )
        
        conn.commit()
        conn.close()
        
        return run_id
    
    def get_benchmark_run(self, run_id: int) -> Tuple[Dict[str, Any], Dict[str, BenchmarkResult]]:
        """Get a benchmark run from the database.
        
        Args:
            run_id: ID of the benchmark run
            
        Returns:
            Tuple of (run metadata, dictionary mapping benchmark names to results)
        """
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # Get benchmark run
        cursor.execute("SELECT * FROM benchmark_runs WHERE id = ?", (run_id,))
        run_row = cursor.fetchone()
        
        if not run_row:
            conn.close()
            raise ValueError(f"Benchmark run with ID {run_id} not found")
        
        run_data = dict(run_row)
        run_data["system_info"] = json.loads(run_data["system_info"])
        
        # Get benchmark results
        cursor.execute("SELECT * FROM benchmark_results WHERE run_id = ?", (run_id,))
        result_rows = cursor.fetchall()
        
        results = {}
        for row in result_rows:
            raw_data = json.loads(row["raw_data"])
            results[row["name"]] = BenchmarkResult.from_dict(raw_data)
        
        conn.close()
        
        return run_data, results
    
    def get_latest_benchmark_run(self) -> Optional[int]:
        """Get the ID of the latest benchmark run.
        
        Returns:
            ID of the latest benchmark run, or None if no runs exist
        """
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute("SELECT MAX(id) FROM benchmark_runs")
        result = cursor.fetchone()[0]
        
        conn.close()
        
        return result
    
    def get_benchmark_history(self, 
                             benchmark_name: str, 
                             limit: int = 10) -> List[Dict[str, Any]]:
        """Get the history of a benchmark.
        
        Args:
            benchmark_name: Name of the benchmark
            limit: Maximum number of results to return
            
        Returns:
            List of benchmark results, ordered by timestamp (newest first)
        """
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute(
            """
            SELECT r.*, b.timestamp, b.git_commit, b.git_branch
            FROM benchmark_results r
            JOIN benchmark_runs b ON r.run_id = b.id
            WHERE r.name = ?
            ORDER BY b.timestamp DESC
            LIMIT ?
            """,
            (benchmark_name, limit)
        )
        
        rows = cursor.fetchall()
        
        history = []
        for row in dict(row) for row in rows:
            history.append(row)
        
        conn.close()
        
        return history
    
    def store_regression_alert(self, 
                              benchmark_name: str, 
                              previous_value: float, 
                              current_value: float,
                              percent_change: float,
                              metric: str,
                              severity: str) -> int:
        """Store a regression alert in the database.
        
        Args:
            benchmark_name: Name of the benchmark
            previous_value: Previous value of the metric
            current_value: Current value of the metric
            percent_change: Percent change between previous and current values
            metric: Name of the metric (e.g., "execution_time", "memory_usage")
            severity: Severity of the regression (e.g., "low", "medium", "high")
            
        Returns:
            ID of the regression alert
        """
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute(
            """
            INSERT INTO regression_alerts 
            (timestamp, benchmark_name, previous_value, current_value, percent_change, metric, severity)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (
                datetime.datetime.now().isoformat(),
                benchmark_name,
                previous_value,
                current_value,
                percent_change,
                metric,
                severity
            )
        )
        
        alert_id = cursor.lastrowid
        
        conn.commit()
        conn.close()
        
        return alert_id
    
    def get_unacknowledged_alerts(self) -> List[Dict[str, Any]]:
        """Get all unacknowledged regression alerts.
        
        Returns:
            List of unacknowledged regression alerts
        """
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute("SELECT * FROM regression_alerts WHERE acknowledged = 0 ORDER BY timestamp DESC")
        
        rows = cursor.fetchall()
        
        alerts = []
        for row in rows:
            alerts.append(dict(row))
        
        conn.close()
        
        return alerts
    
    def acknowledge_alert(self, alert_id: int) -> None:
        """Acknowledge a regression alert.
        
        Args:
            alert_id: ID of the regression alert
        """
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute("UPDATE regression_alerts SET acknowledged = 1 WHERE id = ?", (alert_id,))
        
        conn.commit()
        conn.close()


class RegressionDetector:
    """Detects performance regressions in benchmark results."""
    
    def __init__(self, 
                database: BenchmarkDatabase,
                thresholds: Dict[str, Dict[str, float]] = None):
        """Initialize the regression detector.
        
        Args:
            database: Benchmark database
            thresholds: Dictionary mapping metrics to dictionaries mapping
                       severity levels to thresholds
        """
        self.database = database
        self.thresholds = thresholds or {
            "execution_time": {
                "low": 5.0,      # 5% increase
                "medium": 10.0,  # 10% increase
                "high": 20.0     # 20% increase
            },
            "memory_usage": {
                "low": 5.0,
                "medium": 10.0,
                "high": 20.0
            },
            "token_count": {
                "low": 2.0,
                "medium": 5.0,
                "high": 10.0
            }
        }
    
    def detect_regressions(self, 
                          current_suite: BenchmarkSuite, 
                          baseline_run_id: int = None) -> List[Dict[str, Any]]:
        """Detect regressions between current results and a baseline.
        
        Args:
            current_suite: Current benchmark suite with results
            baseline_run_id: ID of the baseline benchmark run, or None to use the latest
            
        Returns:
            List of regression alerts
        """
        # Get baseline run
        if baseline_run_id is None:
            baseline_run_id = self.database.get_latest_benchmark_run()
        
        if baseline_run_id is None:
            # No baseline available
            return []
        
        # Get baseline results
        _, baseline_results = self.database.get_benchmark_run(baseline_run_id)
        
        # Check for regressions
        alerts = []
        
        for name, current_result in current_suite.results.items():
            if name not in baseline_results:
                continue
            
            baseline_result = baseline_results[name]
            
            # Check execution time
            if current_result.avg_execution_time > baseline_result.avg_execution_time:
                percent_change = ((current_result.avg_execution_time - baseline_result.avg_execution_time) / 
                                 baseline_result.avg_execution_time * 100.0)
                
                severity = self._get_severity("execution_time", percent_change)
                
                if severity:
                    alert_id = self.database.store_regression_alert(
                        benchmark_name=name,
                        previous_value=baseline_result.avg_execution_time,
                        current_value=current_result.avg_execution_time,
                        percent_change=percent_change,
                        metric="execution_time",
                        severity=severity
                    )
                    
                    alerts.append({
                        "id": alert_id,
                        "benchmark_name": name,
                        "metric": "execution_time",
                        "previous_value": baseline_result.avg_execution_time,
                        "current_value": current_result.avg_execution_time,
                        "percent_change": percent_change,
                        "severity": severity
                    })
            
            # Check memory usage
            if (current_result.avg_memory_usage and baseline_result.avg_memory_usage and
                current_result.avg_memory_usage > baseline_result.avg_memory_usage):
                
                percent_change = ((current_result.avg_memory_usage - baseline_result.avg_memory_usage) / 
                                 baseline_result.avg_memory_usage * 100.0)
                
                severity = self._get_severity("memory_usage", percent_change)
                
                if severity:
                    alert_id = self.database.store_regression_alert(
                        benchmark_name=name,
                        previous_value=baseline_result.avg_memory_usage,
                        current_value=current_result.avg_memory_usage,
                        percent_change=percent_change,
                        metric="memory_usage",
                        severity=severity
                    )
                    
                    alerts.append({
                        "id": alert_id,
                        "benchmark_name": name,
                        "metric": "memory_usage",
                        "previous_value": baseline_result.avg_memory_usage,
                        "current_value": current_result.avg_memory_usage,
                        "percent_change": percent_change,
                        "severity": severity
                    })
            
            # Check token count
            if (current_result.avg_token_count and baseline_result.avg_token_count and
                current_result.avg_token_count > baseline_result.avg_token_count):
                
                percent_change = ((current_result.avg_token_count - baseline_result.avg_token_count) / 
                                 baseline_result.avg_token_count * 100.0)
                
                severity = self._get_severity("token_count", percent_change)
                
                if severity:
                    alert_id = self.database.store_regression_alert(
                        benchmark_name=name,
                        previous_value=baseline_result.avg_token_count,
                        current_value=current_result.avg_token_count,
                        percent_change=percent_change,
                        metric="token_count",
                        severity=severity
                    )
                    
                    alerts.append({
                        "id": alert_id,
                        "benchmark_name": name,
                        "metric": "token_count",
                        "previous_value": baseline_result.avg_token_count,
                        "current_value": current_result.avg_token_count,
                        "percent_change": percent_change,
                        "severity": severity
                    })
        
        return alerts
    
    def _get_severity(self, metric: str, percent_change: float) -> Optional[str]:
        """Get the severity of a regression.
        
        Args:
            metric: Name of the metric
            percent_change: Percent change between previous and current values
            
        Returns:
            Severity level, or None if the change is below all thresholds
        """
        if metric not in self.thresholds:
            return None
        
        thresholds = self.thresholds[metric]
        
        if percent_change >= thresholds.get("high", float("inf")):
            return "high"
        elif percent_change >= thresholds.get("medium", float("inf")):
            return "medium"
        elif percent_change >= thresholds.get("low", float("inf")):
            return "low"
        else:
            return None


class ContinuousIntegrationRunner:
    """Runs benchmarks in a CI environment and reports results."""
    
    def __init__(self, 
                database: BenchmarkDatabase = None,
                regression_detector: RegressionDetector = None):
        """Initialize the CI runner.
        
        Args:
            database: Benchmark database
            regression_detector: Regression detector
        """
        self.database = database or BenchmarkDatabase()
        self.regression_detector = regression_detector or RegressionDetector(self.database)
    
    def run_ci_benchmarks(self, 
                         suite: BenchmarkSuite,
                         git_info: bool = True,
                         system_info: bool = True) -> Tuple[int, List[Dict[str, Any]]]:
        """Run benchmarks in a CI environment and store results.
        
        Args:
            suite: Benchmark suite to run
            git_info: Whether to collect Git information
            system_info: Whether to collect system information
            
        Returns:
            Tuple of (benchmark run ID, list of regression alerts)
        """
        # Collect Git information
        git_commit = None
        git_branch = None
        
        if git_info:
            try:
                import subprocess
                
                git_commit = subprocess.check_output(
                    ["git", "rev-parse", "HEAD"],
                    universal_newlines=True
                ).strip()
                
                git_branch = subprocess.check_output(
                    ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                    universal_newlines=True
                ).strip()
            except Exception as e:
                print(f"Error collecting Git information: {e}")
        
        # Collect system information
        sys_info = None
        
        if system_info:
            import platform
            import psutil
            
            sys_info = {
                "platform": platform.platform(),
                "processor": platform.processor(),
                "python_version": platform.python_version(),
                "memory": f"{psutil.virtual_memory().total / (1024**3):.2f} GB",
                "cpu_count": str(psutil.cpu_count(logical=False)),
                "logical_cpu_count": str(psutil.cpu_count(logical=True))
            }
        
        # Store benchmark results
        run_id = self.database.store_benchmark_run(
            suite=suite,
            git_commit=git_commit,
            git_branch=git_branch,
            system_info=sys_info
        )
        
        # Detect regressions
        alerts = self.regression_detector.detect_regressions(suite)
        
        return run_id, alerts
    
    def generate_ci_report(self, 
                          run_id: int, 
                          alerts: List[Dict[str, Any]],
                          output_dir: str = None) -> str:
        """Generate a CI report for a benchmark run.
        
        Args:
            run_id: ID of the benchmark run
            alerts: List of regression alerts
            output_dir: Directory to save the report
            
        Returns:
            Path to the generated report
        """
        # Get benchmark run data
        run_data, results = self.database.get_benchmark_run(run_id)
        
        # Create output directory if it doesn't exist
        if output_dir is None:
            output_dir = os.path.join(
                os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                "benchmark_reports"
            )
        
        os.makedirs(output_dir, exist_ok=True)
        
        # Generate report
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(output_dir, f"ci_report_{timestamp}.md")
        
        with open(output_path, 'w') as f:
            f.write(f"# Benchmark CI Report\n\n")
            
            f.write(f"## Run Information\n\n")
            f.write(f"- **Timestamp:** {run_data['timestamp']}\n")
            
            if run_data['git_commit']:
                f.write(f"- **Git Commit:** {run_data['git_commit']}\n")
            
            if run_data['git_branch']:
                f.write(f"- **Git Branch:** {run_data['git_branch']}\n")
            
            f.write("\n## System Information\n\n")
            
            for key, value in run_data['system_info'].items():
                f.write(f"- **{key}:** {value}\n")
            
            f.write("\n## Benchmark Results\n\n")
            
            f.write("| Benchmark | Execution Time (s) | Memory Usage (MB) | Token Count |\n")
            f.write("|-----------|-------------------|-------------------|-------------|\n")
            
            for name, result in results.items():
                f.write(f"| {name} | {result.avg_execution_time:.6f} | {result.avg_memory_usage:.2f} | {result.avg_token_count:.0f} |\n")
            
            if alerts:
                f.write("\n## Regression Alerts\n\n")
                
                f.write("| Benchmark | Metric | Previous | Current | Change | Severity |\n")
                f.write("|-----------|--------|----------|---------|--------|----------|\n")
                
                for alert in alerts:
                    metric_name = alert['metric'].replace('_', ' ').title()
                    
                    if alert['metric'] == 'execution_time':
                        previous = f"{alert['previous_value']:.6f} s"
                        current = f"{alert['current_value']:.6f} s"
                    elif alert['metric'] == 'memory_usage':
                        previous = f"{alert['previous_value']:.2f} MB"
                        current = f"{alert['current_value']:.2f} MB"
                    else:
                        previous = f"{alert['previous_value']:.0f}"
                        current = f"{alert['current_value']:.0f}"
                    
                    f.write(f"| {alert['benchmark_name']} | {metric_name} | {previous} | {current} | +{alert['percent_change']:.2f}% | {alert['severity'].upper()} |\n")
            else:
                f.write("\n## No Regression Alerts\n\n")
                f.write("No performance regressions were detected in this benchmark run.\n")
        
        return output_path


# Example usage
if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Run benchmarks in a CI environment")
    parser.add_argument("--suite", help="Path to benchmark suite file")
    parser.add_argument("--output-dir", help="Directory to save reports")
    
    args = parser.parse_args()
    
    if args.suite:
        from performance_benchmarking.performance_benchmarking import BenchmarkSuite
        
        # Load benchmark suite
        suite = BenchmarkSuite.load(args.suite)
        
        # Create CI runner
        db = BenchmarkDatabase()
        detector = RegressionDetector(db)
        runner = ContinuousIntegrationRunner(db, detector)
        
        # Run benchmarks
        run_id, alerts = runner.run_ci_benchmarks(suite)
        
        # Generate report
        report_path = runner.generate_ci_report(run_id, alerts, args.output_dir)
        
        print(f"CI report generated: {report_path}")
        
        # Exit with error code if high-severity alerts were found
        if any(alert["severity"] == "high" for alert in alerts):
            sys.exit(1)
    else:
        print("No benchmark suite specified")
        sys.exit(1)
