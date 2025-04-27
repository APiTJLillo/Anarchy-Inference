#!/usr/bin/env python3
"""
Load Test Controller for Anarchy Inference.

This module provides functionality for applying varying loads to the Anarchy Inference
interpreter to identify breaking points and performance degradation patterns.
"""

import os
import time
import threading
import logging
import random
import json
from typing import Dict, List, Any, Optional, Tuple, Callable
from enum import Enum
from dataclasses import dataclass
import subprocess
import multiprocessing
import queue
import signal

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("load_test_controller")

class LoadPattern(Enum):
    """Patterns for applying load during testing."""
    CONSTANT = "constant"         # Steady, unchanging load
    STEP = "step"                 # Load increases in steps
    RAMP = "ramp"                 # Load increases linearly
    SPIKE = "spike"               # Sudden spike in load
    WAVE = "wave"                 # Sinusoidal pattern
    RANDOM = "random"             # Random fluctuations


@dataclass
class LoadTestConfig:
    """Configuration for load testing."""
    pattern: LoadPattern
    initial_load: int             # Initial number of operations/requests
    max_load: int                 # Maximum number of operations/requests
    duration_seconds: int         # Total test duration
    step_size: int = 10           # For step pattern: size of each step
    step_duration: int = 30       # For step pattern: duration of each step
    ramp_rate: float = 1.0        # For ramp pattern: operations per second increase rate
    spike_timing: List[int] = None  # For spike pattern: when spikes occur (seconds from start)
    spike_height: int = 100       # For spike pattern: height of spike
    wave_period: int = 60         # For wave pattern: period of wave in seconds
    random_seed: int = None       # For random pattern: random seed for reproducibility
    output_dir: str = None        # Directory for output files


@dataclass
class LoadTestResult:
    """Results from a load test."""
    config: LoadTestConfig
    success: bool
    error_message: Optional[str] = None
    response_times: List[float] = None
    throughput: List[Tuple[float, int]] = None  # (timestamp, operations_per_second)
    error_rates: List[Tuple[float, float]] = None  # (timestamp, error_rate)
    breaking_point: Optional[int] = None  # Load level where system breaks
    resource_usage: Dict[str, Any] = None


class LoadGenerator:
    """Generates load according to specified patterns."""
    
    def __init__(self, config: LoadTestConfig):
        """Initialize the load generator.
        
        Args:
            config: Configuration for load generation
        """
        self.config = config
        self.current_load = config.initial_load
        self.start_time = None
        self.running = False
        self.thread = None
        self.load_queue = queue.Queue()
        
        # Set random seed if specified
        if config.random_seed is not None:
            random.seed(config.random_seed)
    
    def start(self):
        """Start load generation."""
        if self.running:
            return
        
        self.running = True
        self.start_time = time.time()
        self.thread = threading.Thread(target=self._generate_load)
        self.thread.daemon = True
        self.thread.start()
    
    def stop(self):
        """Stop load generation."""
        if not self.running:
            return
        
        self.running = False
        if self.thread:
            self.thread.join(timeout=2.0)
    
    def get_current_load(self) -> int:
        """Get the current load level.
        
        Returns:
            Current number of operations/requests
        """
        try:
            return self.load_queue.get_nowait()
        except queue.Empty:
            return self.current_load
    
    def _generate_load(self):
        """Generate load according to the specified pattern."""
        while self.running:
            elapsed_seconds = time.time() - self.start_time
            
            # Stop if we've reached the duration
            if elapsed_seconds >= self.config.duration_seconds:
                self.running = False
                break
            
            # Calculate load based on pattern
            if self.config.pattern == LoadPattern.CONSTANT:
                self.current_load = self.config.initial_load
            
            elif self.config.pattern == LoadPattern.STEP:
                step_number = int(elapsed_seconds / self.config.step_duration)
                self.current_load = min(
                    self.config.initial_load + step_number * self.config.step_size,
                    self.config.max_load
                )
            
            elif self.config.pattern == LoadPattern.RAMP:
                self.current_load = min(
                    self.config.initial_load + int(elapsed_seconds * self.config.ramp_rate),
                    self.config.max_load
                )
            
            elif self.config.pattern == LoadPattern.SPIKE:
                self.current_load = self.config.initial_load
                
                # Check if we're in a spike
                if self.config.spike_timing:
                    for spike_time in self.config.spike_timing:
                        if abs(elapsed_seconds - spike_time) < 5:  # 5-second spike
                            self.current_load = self.config.spike_height
                            break
            
            elif self.config.pattern == LoadPattern.WAVE:
                # Sinusoidal wave between initial_load and max_load
                import math
                amplitude = (self.config.max_load - self.config.initial_load) / 2
                offset = self.config.initial_load + amplitude
                self.current_load = int(
                    offset + amplitude * math.sin(
                        2 * math.pi * elapsed_seconds / self.config.wave_period
                    )
                )
            
            elif self.config.pattern == LoadPattern.RANDOM:
                # Random load between initial_load and max_load
                self.current_load = random.randint(
                    self.config.initial_load,
                    self.config.max_load
                )
            
            # Put the current load in the queue
            try:
                self.load_queue.put_nowait(self.current_load)
            except queue.Full:
                pass
            
            # Sleep briefly
            time.sleep(1.0)


class LoadTestController:
    """Controls load testing for Anarchy Inference."""
    
    def __init__(self, config: LoadTestConfig):
        """Initialize the load test controller.
        
        Args:
            config: Configuration for load testing
        """
        self.config = config
        self.load_generator = LoadGenerator(config)
        self.workers = []
        self.response_times = []
        self.throughput = []
        self.error_rates = []
        self.errors = []
        self.breaking_point = None
        self.resource_usage = {}
        
        # Create output directory if specified
        if config.output_dir and not os.path.exists(config.output_dir):
            os.makedirs(config.output_dir)
    
    def run_test(self) -> LoadTestResult:
        """Run the load test.
        
        Returns:
            Results of the load test
        """
        try:
            logger.info(f"Starting load test with pattern: {self.config.pattern.value}")
            
            # Start load generation
            self.load_generator.start()
            
            # Start monitoring
            monitor_thread = threading.Thread(target=self._monitor_test)
            monitor_thread.daemon = True
            monitor_thread.start()
            
            # Run the test
            start_time = time.time()
            self._run_test_loop()
            
            # Stop load generation
            self.load_generator.stop()
            
            # Wait for monitor to finish
            monitor_thread.join(timeout=5.0)
            
            # Calculate results
            success = len(self.errors) == 0
            error_message = "; ".join(self.errors) if self.errors else None
            
            logger.info(f"Load test completed. Success: {success}")
            
            return LoadTestResult(
                config=self.config,
                success=success,
                error_message=error_message,
                response_times=self.response_times,
                throughput=self.throughput,
                error_rates=self.error_rates,
                breaking_point=self.breaking_point,
                resource_usage=self.resource_usage
            )
        
        except Exception as e:
            logger.error(f"Error in load test: {e}")
            return LoadTestResult(
                config=self.config,
                success=False,
                error_message=str(e)
            )
    
    def _run_test_loop(self):
        """Run the main test loop."""
        start_time = time.time()
        operations_count = 0
        errors_count = 0
        
        while time.time() - start_time < self.config.duration_seconds:
            current_load = self.load_generator.get_current_load()
            
            # Adjust number of workers based on current load
            self._adjust_workers(current_load)
            
            # Sleep briefly
            time.sleep(0.1)
    
    def _adjust_workers(self, target_count: int):
        """Adjust the number of worker processes.
        
        Args:
            target_count: Target number of workers
        """
        current_count = len(self.workers)
        
        # Add workers if needed
        while current_count < target_count:
            worker = self._create_worker()
            self.workers.append(worker)
            current_count += 1
        
        # Remove workers if needed
        while current_count > target_count:
            worker = self.workers.pop()
            self._stop_worker(worker)
            current_count -= 1
    
    def _create_worker(self):
        """Create a worker process.
        
        Returns:
            Worker process object
        """
        # Create a worker process that runs Anarchy Inference code
        worker = multiprocessing.Process(
            target=self._worker_process,
            args=(self.config,)
        )
        worker.daemon = True
        worker.start()
        return worker
    
    def _stop_worker(self, worker):
        """Stop a worker process.
        
        Args:
            worker: Worker process to stop
        """
        if worker.is_alive():
            worker.terminate()
            worker.join(timeout=1.0)
    
    def _worker_process(self, config: LoadTestConfig):
        """Worker process function.
        
        Args:
            config: Load test configuration
        """
        try:
            # Run a simple Anarchy Inference program
            while True:
                start_time = time.time()
                
                # Run the interpreter with a test program
                result = self._run_anarchy_program()
                
                # Record response time
                response_time = time.time() - start_time
                
                # Sleep briefly to avoid overwhelming the system
                time.sleep(0.01)
        
        except Exception as e:
            logger.error(f"Error in worker process: {e}")
    
    def _run_anarchy_program(self) -> bool:
        """Run an Anarchy Inference program.
        
        Returns:
            True if successful, False otherwise
        """
        try:
            # Simple test program
            program = """
            // Simple test program for load testing
            result ← 0
            for i in range(1000) {
                result ← result + i
            }
            return result
            """
            
            # Write program to temporary file
            temp_file = f"/tmp/anarchy_load_test_{os.getpid()}.a.i"
            with open(temp_file, "w") as f:
                f.write(program)
            
            # Run the interpreter
            process = subprocess.Popen(
                ["anarchy", temp_file],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            
            # Wait for completion with timeout
            try:
                stdout, stderr = process.communicate(timeout=5.0)
                success = process.returncode == 0
            except subprocess.TimeoutExpired:
                process.kill()
                success = False
            
            # Clean up
            try:
                os.remove(temp_file)
            except:
                pass
            
            return success
        
        except Exception as e:
            logger.error(f"Error running Anarchy program: {e}")
            return False
    
    def _monitor_test(self):
        """Monitor the load test and collect metrics."""
        start_time = time.time()
        operation_counts = []
        error_counts = []
        
        while time.time() - start_time < self.config.duration_seconds:
            # Record current time
            current_time = time.time()
            elapsed = current_time - start_time
            
            # Get current metrics
            current_load = self.load_generator.get_current_load()
            worker_count = len(self.workers)
            
            # Record throughput
            self.throughput.append((elapsed, worker_count))
            
            # Check for breaking point
            if worker_count < current_load and self.breaking_point is None:
                self.breaking_point = current_load
                logger.warning(f"Breaking point detected at load level: {current_load}")
            
            # Sleep briefly
            time.sleep(1.0)
    
    def generate_report(self) -> str:
        """Generate a report of the load test results.
        
        Returns:
            Markdown-formatted report
        """
        # Create report header
        report = f"# Load Test Report\n\n"
        report += f"## Configuration\n\n"
        report += f"- **Pattern**: {self.config.pattern.value}\n"
        report += f"- **Initial Load**: {self.config.initial_load}\n"
        report += f"- **Maximum Load**: {self.config.max_load}\n"
        report += f"- **Duration**: {self.config.duration_seconds} seconds\n\n"
        
        # Add breaking point information
        if self.breaking_point:
            report += f"## Breaking Point\n\n"
            report += f"System reached breaking point at load level: **{self.breaking_point}**\n\n"
        
        # Add throughput information
        if self.throughput:
            report += f"## Throughput\n\n"
            report += f"| Time (s) | Operations/s |\n"
            report += f"|----------|-------------|\n"
            for time_point, ops in self.throughput[:10]:  # Show first 10 points
                report += f"| {time_point:.2f} | {ops} |\n"
            
            if len(self.throughput) > 10:
                report += f"| ... | ... |\n"
        
        # Add error rate information
        if self.error_rates:
            report += f"\n## Error Rates\n\n"
            report += f"| Time (s) | Error Rate |\n"
            report += f"|----------|------------|\n"
            for time_point, rate in self.error_rates[:10]:  # Show first 10 points
                report += f"| {time_point:.2f} | {rate:.2%} |\n"
            
            if len(self.error_rates) > 10:
                report += f"| ... | ... |\n"
        
        # Add timestamp
        report += f"\n\nGenerated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        return report
    
    def save_results(self, result: LoadTestResult):
        """Save load test results to files.
        
        Args:
            result: Load test results
        """
        if not self.config.output_dir:
            return
        
        # Create timestamp
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        
        # Save report
        report_path = os.path.join(
            self.config.output_dir,
            f"load_test_report_{self.config.pattern.value}_{timestamp}.md"
        )
        with open(report_path, "w") as f:
            f.write(self.generate_report())
        
        # Save raw data
        data_path = os.path.join(
            self.config.output_dir,
            f"load_test_data_{self.config.pattern.value}_{timestamp}.json"
        )
        with open(data_path, "w") as f:
            # Convert result to dictionary
            result_dict = {
                "config": {
                    "pattern": self.config.pattern.value,
                    "initial_load": self.config.initial_load,
                    "max_load": self.config.max_load,
                    "duration_seconds": self.config.duration_seconds
                },
                "success": result.success,
                "error_message": result.error_message,
                "throughput": self.throughput,
                "error_rates": self.error_rates,
                "breaking_point": self.breaking_point,
                "resource_usage": self.resource_usage
            }
            json.dump(result_dict, f, indent=2)
        
        logger.info(f"Results saved to {report_path} and {data_path}")


def run_load_test(config: LoadTestConfig) -> LoadTestResult:
    """Run a load test with the specified configuration.
    
    Args:
        config: Load test configuration
        
    Returns:
        Load test results
    """
    controller = LoadTestController(config)
    result = controller.run_test()
    controller.save_results(result)
    return result


def main():
    """Main entry point for the load testing module."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Anarchy Inference Load Testing")
    
    parser.add_argument(
        "--pattern",
        choices=[p.value for p in LoadPattern],
        default=LoadPattern.STEP.value,
        help="Load pattern to use"
    )
    
    parser.add_argument(
        "--initial-load",
        type=int,
        default=10,
        help="Initial load level"
    )
    
    parser.add_argument(
        "--max-load",
        type=int,
        default=100,
        help="Maximum load level"
    )
    
    parser.add_argument(
        "--duration",
        type=int,
        default=300,
        help="Test duration in seconds"
    )
    
    parser.add_argument(
        "--output-dir",
        default="./results",
        help="Directory for output files"
    )
    
    args = parser.parse_args()
    
    # Create configuration
    config = LoadTestConfig(
        pattern=LoadPattern(args.pattern),
        initial_load=args.initial_load,
        max_load=args.max_load,
        duration_seconds=args.duration,
        output_dir=args.output_dir
    )
    
    # Run the test
    result = run_load_test(config)
    
    # Print summary
    print(f"\nLoad Test Summary:")
    print(f"- Pattern: {config.pattern.value}")
    print(f"- Success: {'Yes' if result.success else 'No'}")
    if result.breaking_point:
        print(f"- Breaking Point: {result.breaking_point}")
    print(f"\nDetailed report available in: {config.output_dir}")
    
    return 0 if result.success else 1


if __name__ == "__main__":
    import sys
    sys.exit(main())
