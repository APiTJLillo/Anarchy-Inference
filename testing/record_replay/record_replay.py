"""
Record/Replay System for Anarchy Inference

This module provides functionality to record and replay the execution of Anarchy Inference code,
enabling deterministic testing and debugging.
"""

import os
import sys
import json
import time
import hashlib
import pickle
from typing import Dict, List, Any, Optional, Tuple, Set, Callable

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

class ExecutionState:
    """Represents the state of an Anarchy Inference execution at a specific point."""
    
    def __init__(self, 
                 checkpoint_name: str, 
                 variables: Dict[str, Any] = None,
                 call_stack: List[Dict[str, Any]] = None,
                 memory_state: Dict[str, Any] = None,
                 timestamp: float = None):
        """Initialize an execution state object.
        
        Args:
            checkpoint_name: Name of the checkpoint this state represents
            variables: Dictionary of variable names to values
            call_stack: List of function call frames
            memory_state: Dictionary of memory addresses to values
            timestamp: Time when this state was captured
        """
        self.checkpoint_name = checkpoint_name
        self.variables = variables or {}
        self.call_stack = call_stack or []
        self.memory_state = memory_state or {}
        self.timestamp = timestamp or time.time()
        self.execution_path = []
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the execution state to a dictionary for serialization."""
        return {
            "checkpoint_name": self.checkpoint_name,
            "variables": self.variables,
            "call_stack": self.call_stack,
            "memory_state": self.memory_state,
            "timestamp": self.timestamp,
            "execution_path": self.execution_path
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ExecutionState':
        """Create an execution state from a dictionary."""
        state = cls(
            checkpoint_name=data["checkpoint_name"],
            variables=data["variables"],
            call_stack=data["call_stack"],
            memory_state=data["memory_state"],
            timestamp=data["timestamp"]
        )
        state.execution_path = data.get("execution_path", [])
        return state
    
    def __eq__(self, other: 'ExecutionState') -> bool:
        """Compare two execution states for equality."""
        if not isinstance(other, ExecutionState):
            return False
        
        # Ignore timestamp in equality comparison
        return (self.checkpoint_name == other.checkpoint_name and
                self.variables == other.variables and
                self.call_stack == other.call_stack and
                self.memory_state == other.memory_state and
                self.execution_path == other.execution_path)


class StateSerializer:
    """Handles serialization and deserialization of execution states."""
    
    @staticmethod
    def serialize(state: ExecutionState, format: str = "json") -> str:
        """Serialize an execution state to a string.
        
        Args:
            state: The execution state to serialize
            format: The format to use (json or pickle)
            
        Returns:
            A string representation of the state
        """
        if format == "json":
            return json.dumps(state.to_dict(), indent=2, sort_keys=True)
        elif format == "pickle":
            return pickle.dumps(state).hex()
        else:
            raise ValueError(f"Unsupported serialization format: {format}")
    
    @staticmethod
    def deserialize(data: str, format: str = "json") -> ExecutionState:
        """Deserialize a string to an execution state.
        
        Args:
            data: The serialized state
            format: The format used (json or pickle)
            
        Returns:
            The deserialized execution state
        """
        if format == "json":
            return ExecutionState.from_dict(json.loads(data))
        elif format == "pickle":
            return pickle.loads(bytes.fromhex(data))
        else:
            raise ValueError(f"Unsupported serialization format: {format}")


class Recorder:
    """Records the execution of Anarchy Inference code."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the recorder.
        
        Args:
            interpreter: The Anarchy Inference interpreter to record
        """
        self.interpreter = interpreter
        self.states: Dict[str, ExecutionState] = {}
        self.current_execution_path = []
        self.recording = False
        self.checkpoint_hooks = {}
        self._install_hooks()
    
    def _install_hooks(self):
        """Install hooks into the interpreter to capture execution state."""
        # This would hook into the interpreter's execution pipeline
        # Implementation depends on the specific interpreter API
        pass
    
    def start_recording(self):
        """Start recording execution."""
        self.recording = True
        self.current_execution_path = []
    
    def stop_recording(self):
        """Stop recording execution."""
        self.recording = False
    
    def add_checkpoint(self, name: str, callback: Callable = None):
        """Add a checkpoint to record state at.
        
        Args:
            name: Name of the checkpoint
            callback: Optional function to call when checkpoint is reached
        """
        self.checkpoint_hooks[name] = callback
    
    def capture_state(self, checkpoint_name: str) -> ExecutionState:
        """Capture the current execution state.
        
        Args:
            checkpoint_name: Name of the checkpoint
            
        Returns:
            The captured execution state
        """
        if not self.recording:
            return None
        
        # Capture variables, call stack, and memory state from the interpreter
        variables = self._capture_variables()
        call_stack = self._capture_call_stack()
        memory_state = self._capture_memory_state()
        
        state = ExecutionState(
            checkpoint_name=checkpoint_name,
            variables=variables,
            call_stack=call_stack,
            memory_state=memory_state
        )
        
        state.execution_path = list(self.current_execution_path)
        self.current_execution_path.append(checkpoint_name)
        self.states[checkpoint_name] = state
        
        # Call the checkpoint hook if it exists
        if checkpoint_name in self.checkpoint_hooks and self.checkpoint_hooks[checkpoint_name]:
            self.checkpoint_hooks[checkpoint_name](state)
        
        return state
    
    def _capture_variables(self) -> Dict[str, Any]:
        """Capture the current variables from the interpreter."""
        # Implementation depends on the specific interpreter API
        return {}
    
    def _capture_call_stack(self) -> List[Dict[str, Any]]:
        """Capture the current call stack from the interpreter."""
        # Implementation depends on the specific interpreter API
        return []
    
    def _capture_memory_state(self) -> Dict[str, Any]:
        """Capture the current memory state from the interpreter."""
        # Implementation depends on the specific interpreter API
        return {}
    
    def save_recording(self, output_file: str, format: str = "json"):
        """Save the recorded states to a file.
        
        Args:
            output_file: Path to the output file
            format: Format to use (json or pickle)
        """
        data = {
            "states": {name: state.to_dict() for name, state in self.states.items()},
            "execution_path": self.current_execution_path,
            "timestamp": time.time()
        }
        
        with open(output_file, 'w' if format == "json" else 'wb') as f:
            if format == "json":
                json.dump(data, f, indent=2, sort_keys=True)
            else:
                pickle.dump(data, f)


class Replayer:
    """Replays recorded executions of Anarchy Inference code."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the replayer.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use for replay
        """
        self.interpreter = interpreter
        self.states: Dict[str, ExecutionState] = {}
        self.execution_path = []
        self.current_checkpoint_index = 0
    
    def load_recording(self, input_file: str, format: str = "json"):
        """Load recorded states from a file.
        
        Args:
            input_file: Path to the input file
            format: Format used (json or pickle)
        """
        with open(input_file, 'r' if format == "json" else 'rb') as f:
            if format == "json":
                data = json.load(f)
            else:
                data = pickle.load(f)
        
        self.states = {name: ExecutionState.from_dict(state_dict) 
                      for name, state_dict in data["states"].items()}
        self.execution_path = data["execution_path"]
        self.current_checkpoint_index = 0
    
    def replay_to_checkpoint(self, checkpoint_name: str) -> ExecutionState:
        """Replay execution up to a specific checkpoint.
        
        Args:
            checkpoint_name: Name of the checkpoint to replay to
            
        Returns:
            The execution state at the checkpoint
        """
        if checkpoint_name not in self.states:
            raise ValueError(f"Checkpoint {checkpoint_name} not found in recording")
        
        # Find the checkpoint in the execution path
        try:
            target_index = self.execution_path.index(checkpoint_name)
        except ValueError:
            raise ValueError(f"Checkpoint {checkpoint_name} not in execution path")
        
        # Replay from current position to target
        for i in range(self.current_checkpoint_index, target_index + 1):
            checkpoint = self.execution_path[i]
            self._apply_state(self.states[checkpoint])
        
        self.current_checkpoint_index = target_index
        return self.states[checkpoint_name]
    
    def replay_next_checkpoint(self) -> Optional[ExecutionState]:
        """Replay execution to the next checkpoint.
        
        Returns:
            The execution state at the next checkpoint, or None if at the end
        """
        if self.current_checkpoint_index >= len(self.execution_path) - 1:
            return None
        
        self.current_checkpoint_index += 1
        checkpoint = self.execution_path[self.current_checkpoint_index]
        self._apply_state(self.states[checkpoint])
        return self.states[checkpoint]
    
    def _apply_state(self, state: ExecutionState):
        """Apply an execution state to the interpreter.
        
        Args:
            state: The execution state to apply
        """
        # Implementation depends on the specific interpreter API
        # This would restore variables, call stack, and memory state
        pass


class ExpectationManager:
    """Manages expectation files for record/replay testing."""
    
    def __init__(self, exp_dir: str = None):
        """Initialize the expectation manager.
        
        Args:
            exp_dir: Directory containing expectation files
        """
        self.exp_dir = exp_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "test_expectations"
        )
        
        # Create the expectation directory if it doesn't exist
        if not os.path.exists(self.exp_dir):
            os.makedirs(self.exp_dir)
    
    def get_expectation_path(self, test_file: str, checkpoint: str) -> str:
        """Get the path to an expectation file.
        
        Args:
            test_file: Path to the test file
            checkpoint: Name of the checkpoint
            
        Returns:
            Path to the expectation file
        """
        test_name = os.path.splitext(os.path.basename(test_file))[0]
        return os.path.join(self.exp_dir, f"{test_name}.{checkpoint}.exp")
    
    def save_expectation(self, state: ExecutionState, test_file: str, format: str = "json"):
        """Save an execution state as an expectation.
        
        Args:
            state: The execution state to save
            test_file: Path to the test file
            format: Format to use (json or pickle)
        """
        exp_path = self.get_expectation_path(test_file, state.checkpoint_name)
        
        with open(exp_path, 'w' if format == "json" else 'wb') as f:
            if format == "json":
                json.dump(state.to_dict(), f, indent=2, sort_keys=True)
            else:
                pickle.dump(state, f)
    
    def load_expectation(self, test_file: str, checkpoint: str, format: str = "json") -> Optional[ExecutionState]:
        """Load an expectation for a test and checkpoint.
        
        Args:
            test_file: Path to the test file
            checkpoint: Name of the checkpoint
            format: Format used (json or pickle)
            
        Returns:
            The loaded execution state, or None if not found
        """
        exp_path = self.get_expectation_path(test_file, checkpoint)
        
        if not os.path.exists(exp_path):
            return None
        
        with open(exp_path, 'r' if format == "json" else 'rb') as f:
            if format == "json":
                data = json.load(f)
                return ExecutionState.from_dict(data)
            else:
                return pickle.load(f)
    
    def compare_with_expectation(self, state: ExecutionState, test_file: str) -> Tuple[bool, Dict[str, Any]]:
        """Compare an execution state with its expectation.
        
        Args:
            state: The execution state to compare
            test_file: Path to the test file
            
        Returns:
            A tuple of (matches, diff) where matches is a boolean and diff is a dictionary
            describing the differences
        """
        expected_state = self.load_expectation(test_file, state.checkpoint_name)
        
        if expected_state is None:
            return False, {"error": "Expectation not found"}
        
        if state == expected_state:
            return True, {}
        
        # Generate a detailed diff
        diff = self._generate_diff(expected_state, state)
        return False, diff
    
    def _generate_diff(self, expected: ExecutionState, actual: ExecutionState) -> Dict[str, Any]:
        """Generate a detailed diff between two execution states.
        
        Args:
            expected: The expected execution state
            actual: The actual execution state
            
        Returns:
            A dictionary describing the differences
        """
        diff = {}
        
        # Compare variables
        var_diff = {}
        all_vars = set(expected.variables.keys()) | set(actual.variables.keys())
        for var in all_vars:
            if var not in expected.variables:
                var_diff[var] = {"type": "added", "value": actual.variables[var]}
            elif var not in actual.variables:
                var_diff[var] = {"type": "removed", "value": expected.variables[var]}
            elif expected.variables[var] != actual.variables[var]:
                var_diff[var] = {
                    "type": "changed",
                    "expected": expected.variables[var],
                    "actual": actual.variables[var]
                }
        
        if var_diff:
            diff["variables"] = var_diff
        
        # Compare execution path
        if expected.execution_path != actual.execution_path:
            diff["execution_path"] = {
                "expected": expected.execution_path,
                "actual": actual.execution_path
            }
        
        # Add other comparisons as needed
        
        return diff
    
    def update_all_expectations(self, test_dir: str, interpreter: 'anarchy.Interpreter', format: str = "json"):
        """Update all expectation files based on current code.
        
        Args:
            test_dir: Directory containing test files
            interpreter: The Anarchy Inference interpreter to use
            format: Format to use (json or pickle)
        """
        recorder = Recorder(interpreter)
        
        for root, _, files in os.walk(test_dir):
            for file in files:
                if file.endswith(".ai"):  # Anarchy Inference files
                    test_file = os.path.join(root, file)
                    self._update_test_expectations(test_file, recorder, format)
    
    def _update_test_expectations(self, test_file: str, recorder: Recorder, format: str):
        """Update expectations for a single test file.
        
        Args:
            test_file: Path to the test file
            recorder: The recorder to use
            format: Format to use (json or pickle)
        """
        # Read the test file to find checkpoint markers
        with open(test_file, 'r') as f:
            content = f.read()
        
        # Extract checkpoint names from the file
        # This is a simplified example; actual implementation would depend on
        # how checkpoints are marked in Anarchy Inference code
        checkpoints = []
        for line in content.split('\n'):
            if '# CHECKPOINT:' in line:
                checkpoint_name = line.split('# CHECKPOINT:')[1].strip()
                checkpoints.append(checkpoint_name)
                recorder.add_checkpoint(checkpoint_name)
        
        # Execute the test with recording
        recorder.start_recording()
        try:
            with open(test_file, 'r') as f:
                code = f.read()
            recorder.interpreter.execute(code)
        finally:
            recorder.stop_recording()
        
        # Save the recorded states as expectations
        for checkpoint in checkpoints:
            if checkpoint in recorder.states:
                self.save_expectation(recorder.states[checkpoint], test_file, format)


class RecordReplayTester:
    """Runs tests using the record/replay system."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter', exp_manager: ExpectationManager = None):
        """Initialize the tester.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
            exp_manager: The expectation manager to use
        """
        self.interpreter = interpreter
        self.exp_manager = exp_manager or ExpectationManager()
        self.recorder = Recorder(interpreter)
        self.replayer = Replayer(interpreter)
        self.results = {}
    
    def run_test(self, test_file: str) -> Dict[str, Any]:
        """Run a test and compare with expectations.
        
        Args:
            test_file: Path to the test file
            
        Returns:
            A dictionary with test results
        """
        # Read the test file to find checkpoint markers
        with open(test_file, 'r') as f:
            content = f.read()
        
        # Extract checkpoint names from the file
        checkpoints = []
        for line in content.split('\n'):
            if '# CHECKPOINT:' in line:
                checkpoint_name = line.split('# CHECKPOINT:')[1].strip()
                checkpoints.append(checkpoint_name)
                self.recorder.add_checkpoint(checkpoint_name)
        
        # Execute the test with recording
        self.recorder.start_recording()
        try:
            with open(test_file, 'r') as f:
                code = f.read()
            self.interpreter.execute(code)
        finally:
            self.recorder.stop_recording()
        
        # Compare with expectations
        results = {
            "test_file": test_file,
            "checkpoints": {},
            "success": True
        }
        
        for checkpoint in checkpoints:
            if checkpoint not in self.recorder.states:
                results["checkpoints"][checkpoint] = {
                    "success": False,
                    "error": "Checkpoint not reached"
                }
                results["success"] = False
                continue
            
            state = self.recorder.states[checkpoint]
            matches, diff = self.exp_manager.compare_with_expectation(state, test_file)
            
            results["checkpoints"][checkpoint] = {
                "success": matches,
                "diff": diff if not matches else {}
            }
            
            if not matches:
                results["success"] = False
        
        self.results[test_file] = results
        return results
    
    def run_all_tests(self, test_dir: str) -> Dict[str, Dict[str, Any]]:
        """Run all tests in a directory.
        
        Args:
            test_dir: Directory containing test files
            
        Returns:
            A dictionary mapping test files to their results
        """
        self.results = {}
        
        for root, _, files in os.walk(test_dir):
            for file in files:
                if file.endswith(".ai"):  # Anarchy Inference files
                    test_file = os.path.join(root, file)
                    self.run_test(test_file)
        
        return self.results
    
    def generate_report(self, output_file: str = None) -> str:
        """Generate a test report in markdown format.
        
        Args:
            output_file: Path to the output file
            
        Returns:
            The report as a string
        """
        if not self.results:
            return "No test results available. Run tests first."
        
        report = "# Anarchy Inference Record/Replay Test Report\n\n"
        
        # Summary
        total_tests = len(self.results)
        passed_tests = sum(1 for result in self.results.values() if result["success"])
        
        report += f"## Summary\n\n"
        report += f"- **Total Tests**: {total_tests}\n"
        report += f"- **Passed**: {passed_tests}\n"
        report += f"- **Failed**: {total_tests - passed_tests}\n"
        report += f"- **Success Rate**: {passed_tests / total_tests:.2%}\n\n"
        
        # Test details
        report += f"## Test Details\n\n"
        
        for test_file, result in self.results.items():
            test_name = os.path.basename(test_file)
            status = "✅ Passed" if result["success"] else "❌ Failed"
            
            report += f"### {test_name} - {status}\n\n"
            
            for checkpoint, cp_result in result["checkpoints"].items():
                cp_status = "✅ Passed" if cp_result["success"] else "❌ Failed"
                report += f"- **Checkpoint**: {checkpoint} - {cp_status}\n"
                
                if not cp_result["success"]:
                    report += "  - **Differences**:\n"
                    for key, diff in cp_result["diff"].items():
                        report += f"    - {key}: {diff}\n"
            
            report += "\n"
        
        # Add timestamp
        report += f"Generated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report)
            print(f"Report written to {output_file}")
        
        return report
