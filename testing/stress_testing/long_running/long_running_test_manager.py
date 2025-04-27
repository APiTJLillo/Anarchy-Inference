"""
Long-Running Test Manager for Anarchy Inference

This module provides classes for managing tests that run for extended periods,
including test scheduling, checkpoint management, and result monitoring.
"""

import os
import sys
import time
import threading
import datetime
from typing import Dict, List, Any, Optional, Callable
from dataclasses import dataclass
import json
import signal

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")

class LongRunningTestManager:
    """Manages tests that run for extended periods of time."""
    
    def __init__(self, interpreter, checkpoint_dir: str = "/tmp/anarchy_checkpoints"):
        """Initialize the long-running test manager.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
            checkpoint_dir: Directory to store checkpoints
        """
        self.interpreter = interpreter
        self.checkpoint_dir = checkpoint_dir
        self.running_tests = {}
        self.test_results = {}
        
        # Create checkpoint directory if it doesn't exist
        os.makedirs(checkpoint_dir, exist_ok=True)
    
    def schedule_test(self, test_id: str, code: str, duration_seconds: int, 
                     checkpoint_interval: int = 300) -> Dict[str, Any]:
        """Schedule a long-running test.
        
        Args:
            test_id: Unique identifier for the test
            code: The Anarchy Inference code to execute
            duration_seconds: Maximum duration in seconds
            checkpoint_interval: Interval between checkpoints in seconds
            
        Returns:
            Information about the scheduled test
        """
        if test_id in self.running_tests:
            return {
                "success": False,
                "error": f"Test with ID {test_id} is already running"
            }
        
        # Create test thread
        thread = threading.Thread(
            target=self._run_test,
            args=(test_id, code, duration_seconds, checkpoint_interval),
            name=f"LongRunningTest-{test_id}"
        )
        
        # Store test information
        self.running_tests[test_id] = {
            "thread": thread,
            "start_time": time.time(),
            "duration": duration_seconds,
            "checkpoint_interval": checkpoint_interval,
            "checkpoints": [],
            "status": "scheduled"
        }
        
        # Start the test
        thread.start()
        
        return {
            "success": True,
            "test_id": test_id,
            "start_time": datetime.datetime.now().isoformat(),
            "expected_end_time": (datetime.datetime.now() + 
                                 datetime.timedelta(seconds=duration_seconds)).isoformat(),
            "status": "running"
        }
    
    def get_test_status(self, test_id: str) -> Dict[str, Any]:
        """Get the status of a running or completed test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Status information about the test
        """
        # Check if test is running
        if test_id in self.running_tests:
            test_info = self.running_tests[test_id]
            elapsed = time.time() - test_info["start_time"]
            remaining = max(0, test_info["duration"] - elapsed)
            
            return {
                "test_id": test_id,
                "status": test_info["status"],
                "start_time": datetime.datetime.fromtimestamp(test_info["start_time"]).isoformat(),
                "elapsed_seconds": elapsed,
                "remaining_seconds": remaining,
                "checkpoint_count": len(test_info["checkpoints"]),
                "latest_checkpoint": test_info["checkpoints"][-1] if test_info["checkpoints"] else None
            }
        
        # Check if test has completed
        if test_id in self.test_results:
            return {
                "test_id": test_id,
                "status": "completed",
                "result": self.test_results[test_id]
            }
        
        return {
            "test_id": test_id,
            "status": "not_found",
            "error": f"No test found with ID {test_id}"
        }
    
    def stop_test(self, test_id: str) -> Dict[str, Any]:
        """Stop a running test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Result of the stop operation
        """
        if test_id not in self.running_tests:
            return {
                "success": False,
                "error": f"No running test found with ID {test_id}"
            }
        
        test_info = self.running_tests[test_id]
        
        # Set status to stopping
        test_info["status"] = "stopping"
        
        # Wait for thread to complete (with timeout)
        test_info["thread"].join(timeout=10)
        
        # If thread is still running, it's stuck
        if test_info["thread"].is_alive():
            return {
                "success": False,
                "error": f"Test {test_id} could not be stopped gracefully"
            }
        
        # Test stopped successfully
        return {
            "success": True,
            "test_id": test_id,
            "status": "stopped",
            "checkpoint_count": len(test_info["checkpoints"]),
            "elapsed_seconds": time.time() - test_info["start_time"]
        }
    
    def list_tests(self) -> Dict[str, List[str]]:
        """List all running and completed tests.
        
        Returns:
            Lists of running and completed test IDs
        """
        return {
            "running_tests": list(self.running_tests.keys()),
            "completed_tests": list(self.test_results.keys())
        }
    
    def _run_test(self, test_id: str, code: str, duration_seconds: int, checkpoint_interval: int):
        """Run a long-running test with checkpoints.
        
        Args:
            test_id: Unique identifier for the test
            code: The Anarchy Inference code to execute
            duration_seconds: Maximum duration in seconds
            checkpoint_interval: Interval between checkpoints in seconds
        """
        test_info = self.running_tests[test_id]
        test_info["status"] = "running"
        
        # Set up checkpoint directory for this test
        test_checkpoint_dir = os.path.join(self.checkpoint_dir, test_id)
        os.makedirs(test_checkpoint_dir, exist_ok=True)
        
        try:
            # Create a new interpreter instance for this test
            test_interpreter = anarchy.Interpreter()
            
            # Set up timeout handler
            def timeout_handler(signum, frame):
                raise TimeoutError("Test execution timed out")
            
            # Register timeout handler
            original_handler = signal.signal(signal.SIGALRM, timeout_handler)
            signal.alarm(duration_seconds)
            
            # Start execution time
            start_time = time.time()
            last_checkpoint_time = start_time
            
            # Execute the code
            result = test_interpreter.execute(code)
            
            # Test completed successfully
            test_info["status"] = "completed"
            self.test_results[test_id] = {
                "success": True,
                "result": result,
                "execution_time": time.time() - start_time,
                "checkpoints": test_info["checkpoints"]
            }
        
        except TimeoutError:
            # Test timed out
            test_info["status"] = "timed_out"
            self.test_results[test_id] = {
                "success": False,
                "error": "Test execution timed out",
                "execution_time": time.time() - start_time,
                "checkpoints": test_info["checkpoints"]
            }
        
        except Exception as e:
            # Test failed with an error
            test_info["status"] = "failed"
            self.test_results[test_id] = {
                "success": False,
                "error": str(e),
                "execution_time": time.time() - start_time,
                "checkpoints": test_info["checkpoints"]
            }
        
        finally:
            # Restore original signal handler
            signal.signal(signal.SIGALRM, original_handler)
            signal.alarm(0)
            
            # Remove from running tests
            if test_id in self.running_tests:
                del self.running_tests[test_id]

class CheckpointManager:
    """Manages checkpoints for long-running tests."""
    
    def __init__(self, checkpoint_dir: str = "/tmp/anarchy_checkpoints"):
        """Initialize the checkpoint manager.
        
        Args:
            checkpoint_dir: Directory to store checkpoints
        """
        self.checkpoint_dir = checkpoint_dir
        os.makedirs(checkpoint_dir, exist_ok=True)
    
    def create_checkpoint(self, test_id: str, state: Dict[str, Any]) -> str:
        """Create a checkpoint for a test.
        
        Args:
            test_id: Unique identifier for the test
            state: Test state to checkpoint
            
        Returns:
            Path to the checkpoint file
        """
        # Create test checkpoint directory
        test_checkpoint_dir = os.path.join(self.checkpoint_dir, test_id)
        os.makedirs(test_checkpoint_dir, exist_ok=True)
        
        # Create checkpoint filename with timestamp
        timestamp = int(time.time())
        checkpoint_file = os.path.join(test_checkpoint_dir, f"checkpoint_{timestamp}.json")
        
        # Add timestamp to state
        state["timestamp"] = timestamp
        state["checkpoint_time"] = datetime.datetime.now().isoformat()
        
        # Write checkpoint to file
        with open(checkpoint_file, 'w') as f:
            json.dump(state, f, indent=2)
        
        return checkpoint_file
    
    def load_checkpoint(self, checkpoint_file: str) -> Dict[str, Any]:
        """Load a checkpoint from a file.
        
        Args:
            checkpoint_file: Path to the checkpoint file
            
        Returns:
            The checkpoint state
        """
        with open(checkpoint_file, 'r') as f:
            return json.load(f)
    
    def list_checkpoints(self, test_id: str) -> List[str]:
        """List all checkpoints for a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            List of checkpoint file paths
        """
        test_checkpoint_dir = os.path.join(self.checkpoint_dir, test_id)
        
        if not os.path.exists(test_checkpoint_dir):
            return []
        
        # Get all checkpoint files and sort by timestamp
        checkpoint_files = [
            os.path.join(test_checkpoint_dir, f)
            for f in os.listdir(test_checkpoint_dir)
            if f.startswith("checkpoint_") and f.endswith(".json")
        ]
        
        checkpoint_files.sort()
        return checkpoint_files
    
    def get_latest_checkpoint(self, test_id: str) -> Optional[str]:
        """Get the latest checkpoint for a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Path to the latest checkpoint file, or None if no checkpoints exist
        """
        checkpoint_files = self.list_checkpoints(test_id)
        
        if not checkpoint_files:
            return None
        
        return checkpoint_files[-1]
    
    def delete_checkpoints(self, test_id: str) -> int:
        """Delete all checkpoints for a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Number of checkpoints deleted
        """
        checkpoint_files = self.list_checkpoints(test_id)
        
        for checkpoint_file in checkpoint_files:
            os.remove(checkpoint_file)
        
        # Try to remove the test checkpoint directory
        test_checkpoint_dir = os.path.join(self.checkpoint_dir, test_id)
        try:
            os.rmdir(test_checkpoint_dir)
        except OSError:
            # Directory not empty or doesn't exist
            pass
        
        return len(checkpoint_files)

class ProgressMonitor:
    """Monitors the progress of long-running tests."""
    
    def __init__(self):
        """Initialize the progress monitor."""
        self.monitored_tests = {}
    
    def start_monitoring(self, test_id: str, total_work: int) -> Dict[str, Any]:
        """Start monitoring a test's progress.
        
        Args:
            test_id: Unique identifier for the test
            total_work: Total amount of work to be done
            
        Returns:
            Information about the monitored test
        """
        self.monitored_tests[test_id] = {
            "start_time": time.time(),
            "total_work": total_work,
            "completed_work": 0,
            "progress": 0.0,
            "estimated_completion_time": None,
            "updates": []
        }
        
        return {
            "test_id": test_id,
            "status": "monitoring",
            "total_work": total_work
        }
    
    def update_progress(self, test_id: str, completed_work: int) -> Dict[str, Any]:
        """Update the progress of a monitored test.
        
        Args:
            test_id: Unique identifier for the test
            completed_work: Amount of work completed so far
            
        Returns:
            Updated progress information
        """
        if test_id not in self.monitored_tests:
            return {
                "success": False,
                "error": f"No monitored test found with ID {test_id}"
            }
        
        test_info = self.monitored_tests[test_id]
        
        # Update progress
        test_info["completed_work"] = completed_work
        test_info["progress"] = completed_work / test_info["total_work"]
        
        # Calculate estimated completion time
        elapsed_time = time.time() - test_info["start_time"]
        if test_info["progress"] > 0:
            estimated_total_time = elapsed_time / test_info["progress"]
            estimated_remaining_time = estimated_total_time - elapsed_time
            test_info["estimated_completion_time"] = time.time() + estimated_remaining_time
        
        # Record update
        test_info["updates"].append({
            "timestamp": time.time(),
            "completed_work": completed_work,
            "progress": test_info["progress"]
        })
        
        return {
            "test_id": test_id,
            "progress": test_info["progress"],
            "completed_work": completed_work,
            "total_work": test_info["total_work"],
            "elapsed_time": elapsed_time,
            "estimated_completion_time": (
                datetime.datetime.fromtimestamp(test_info["estimated_completion_time"]).isoformat()
                if test_info["estimated_completion_time"] else None
            )
        }
    
    def get_progress(self, test_id: str) -> Dict[str, Any]:
        """Get the current progress of a monitored test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Current progress information
        """
        if test_id not in self.monitored_tests:
            return {
                "success": False,
                "error": f"No monitored test found with ID {test_id}"
            }
        
        test_info = self.monitored_tests[test_id]
        elapsed_time = time.time() - test_info["start_time"]
        
        return {
            "test_id": test_id,
            "progress": test_info["progress"],
            "completed_work": test_info["completed_work"],
            "total_work": test_info["total_work"],
            "elapsed_time": elapsed_time,
            "estimated_completion_time": (
                datetime.datetime.fromtimestamp(test_info["estimated_completion_time"]).isoformat()
                if test_info["estimated_completion_time"] else None
            ),
            "update_count": len(test_info["updates"])
        }
    
    def stop_monitoring(self, test_id: str) -> Dict[str, Any]:
        """Stop monitoring a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Final progress information
        """
        if test_id not in self.monitored_tests:
            return {
                "success": False,
                "error": f"No monitored test found with ID {test_id}"
            }
        
        test_info = self.monitored_tests[test_id]
        elapsed_time = time.time() - test_info["start_time"]
        
        # Remove from monitored tests
        del self.monitored_tests[test_id]
        
        return {
            "test_id": test_id,
            "status": "completed",
            "final_progress": test_info["progress"],
            "completed_work": test_info["completed_work"],
            "total_work": test_info["total_work"],
            "total_elapsed_time": elapsed_time,
            "update_count": len(test_info["updates"])
        }

class ResourceUsageTracker:
    """Tracks resource usage over time for long-running tests."""
    
    def __init__(self, sampling_interval: float = 10.0):
        """Initialize the resource usage tracker.
        
        Args:
            sampling_interval: Time between resource usage samples in seconds
        """
        self.sampling_interval = sampling_interval
        self.tracked_tests = {}
    
    def start_tracking(self, test_id: str) -> Dict[str, Any]:
        """Start tracking resource usage for a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Information about the tracked test
        """
        if test_id in self.tracked_tests:
            return {
                "success": False,
                "error": f"Already tracking test with ID {test_id}"
            }
        
        # Create tracking thread
        stop_event = threading.Event()
        thread = threading.Thread(
            target=self._track_resources,
            args=(test_id, stop_event),
            name=f"ResourceTracker-{test_id}"
        )
        
        # Store tracking information
        self.tracked_tests[test_id] = {
            "thread": thread,
            "stop_event": stop_event,
            "start_time": time.time(),
            "samples": [],
            "status": "tracking"
        }
        
        # Start tracking
        thread.start()
        
        return {
            "success": True,
            "test_id": test_id,
            "start_time": datetime.datetime.now().isoformat(),
            "sampling_interval": self.sampling_interval,
            "status": "tracking"
        }
    
    def stop_tracking(self, test_id: str) -> Dict[str, Any]:
        """Stop tracking resource usage for a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Summary of resource usage
        """
        if test_id not in self.tracked_tests:
            return {
                "success": False,
                "error": f"No tracked test found with ID {test_id}"
            }
        
        test_info = self.tracked_tests[test_id]
        
        # Signal thread to stop
        test_info["stop_event"].set()
        
        # Wait for thread to complete (with timeout)
        test_info["thread"].join(timeout=10)
        
        # Calculate statistics
        samples = test_info["samples"]
        
        if not samples:
            summary = {
                "success": True,
                "test_id": test_id,
                "status": "stopped",
                "error": "No samples collected"
            }
        else:
            # Calculate memory statistics
            memory_values = [sample["memory_mb"] for sample in samples]
            
            # Calculate CPU statistics
            cpu_values = [sample["cpu_percent"] for sample in samples]
            
            summary = {
                "success": True,
                "test_id": test_id,
                "status": "stopped",
                "tracking_duration": time.time() - test_info["start_time"],
                "sample_count": len(samples),
                "memory": {
                    "min_mb": min(memory_values),
                    "max_mb": max(memory_values),
                    "avg_mb": sum(memory_values) / len(memory_values),
                    "final_mb": memory_values[-1]
                },
                "cpu": {
                    "min_percent": min(cpu_values),
                    "max_percent": max(cpu_values),
                    "avg_percent": sum(cpu_values) / len(cpu_values)
                }
            }
        
        # Remove from tracked tests
        del self.tracked_tests[test_id]
        
        return summary
    
    def get_tracking_status(self, test_id: str) -> Dict[str, Any]:
        """Get the current tracking status for a test.
        
        Args:
            test_id: Unique identifier for the test
            
        Returns:
            Current tracking status
        """
        if test_id not in self.tracked_tests:
            return {
                "success": False,
                "error": f"No tracked test found with ID {test_id}"
            }
        
        test_info = self.tracked_tests[test_id]
        
        return {
            "test_id": test_id,
            "status": test_info["status"],
            "tracking_duration": time.time() - test_info["start_time"],
            "sample_count": len(test_info["samples"]),
            "latest_sample": test_info["samples"][-1] if test_info["samples"] else None
        }
    
    def _track_resources(self, test_id: str, stop_event: threading.Event):
        """Track resource usage for a test in a background thread.
        
        Args:
            test_id: Unique identifier for the test
            stop_event: Event to signal when tracking should stop
        """
        import psutil
        
        test_info = self.tracked_tests[test_id]
        process = psutil.Process(os.getpid())
        
        while not stop_event.is_set():
            try:
                # Get memory usage
                memory_info = process.memory_info()
                memory_mb = memory_info.rss / (1024 * 1024)  # Convert to MB
                
                # Get CPU usage
                cpu_percent = process.cpu_percent(interval=None)
                
                # Record sample
                sample = {
                    "timestamp": time.time(),
                    "memory_mb": memory_mb,
                    "cpu_percent": cpu_percent
                }
                
                test_info["samples"].append(sample)
                
                # Wait for next sample
                stop_event.wait(self.sampling_interval)
            
            except Exception as e:
                print(f"Error in resource tracking for test {test_id}: {e}")
                break
