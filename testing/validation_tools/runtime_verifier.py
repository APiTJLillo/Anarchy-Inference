"""
Runtime Verification for Anarchy Inference

This module provides runtime verification capabilities to monitor program execution
and verify correctness during runtime.
"""

import os
import json
import time
import threading
import traceback
from typing import List, Dict, Any, Optional, Set, Tuple, Callable

from .static_analyzer import ValidationResult

class RuntimeMonitor:
    """Base class for runtime monitors."""
    
    def __init__(self, name: str, description: str):
        """
        Initialize a runtime monitor.
        
        Args:
            name: Name of the monitor
            description: Description of what the monitor checks for
        """
        self.name = name
        self.description = description
        self.active = False
        self.results = []
    
    def start(self):
        """Start the monitor."""
        self.active = True
        self.results = []
    
    def stop(self):
        """Stop the monitor."""
        self.active = False
    
    def report(self) -> List[ValidationResult]:
        """Get the current results."""
        return self.results.copy()
    
    def add_result(self, result: ValidationResult):
        """Add a result to the monitor."""
        self.results.append(result)


class AssertionMonitor(RuntimeMonitor):
    """Monitor for runtime assertions."""
    
    def __init__(self):
        """Initialize the assertion monitor."""
        super().__init__("AssertionMonitor", "Monitors runtime assertions")
        self.original_assert_handler = None
    
    def assertion_handler(self, expression, message):
        """Custom assertion handler that records failures."""
        if not expression:
            # Get the call stack to find where the assertion failed
            stack = traceback.extract_stack()
            # The frame before the assertion handler is where the assertion was made
            frame = stack[-2]
            file_path, line_number, func_name, _ = frame
            
            # Create a validation result for the assertion failure
            result = ValidationResult(
                tool="RuntimeVerifier",
                rule_id="ASSERT001",
                message=f"Assertion failed: {message}" if message else "Assertion failed",
                file_path=file_path,
                line_number=line_number,
                severity="error",
                code_snippet="",  # We don't have the code snippet here
                suggestion="Ensure the condition is met before this point",
                category="assertion"
            )
            
            self.add_result(result)
            
            # Re-raise the assertion to maintain normal behavior
            raise AssertionError(message)
    
    def start(self):
        """Start monitoring assertions."""
        super().start()
        # Save the original assert function
        self.original_assert_handler = __builtins__.__dict__['assert']
        # Replace with our custom handler
        __builtins__.__dict__['assert'] = self.assertion_handler
    
    def stop(self):
        """Stop monitoring assertions."""
        super().stop()
        # Restore the original assert function
        if self.original_assert_handler:
            __builtins__.__dict__['assert'] = self.original_assert_handler
            self.original_assert_handler = None


class InvariantMonitor(RuntimeMonitor):
    """Monitor for program invariants."""
    
    def __init__(self):
        """Initialize the invariant monitor."""
        super().__init__("InvariantMonitor", "Monitors program invariants")
        self.invariants = {}  # Maps invariant names to check functions
    
    def add_invariant(self, name: str, check_func: Callable[[], bool], message: str = None):
        """
        Add an invariant to monitor.
        
        Args:
            name: Name of the invariant
            check_func: Function that returns True if the invariant holds
            message: Optional message to include in the result
        """
        self.invariants[name] = (check_func, message or f"Invariant '{name}' violated")
    
    def check_invariants(self, file_path: str, line_number: int):
        """
        Check all registered invariants.
        
        Args:
            file_path: Path to the file where the check is being made
            line_number: Line number where the check is being made
        """
        if not self.active:
            return
        
        for name, (check_func, message) in self.invariants.items():
            try:
                if not check_func():
                    result = ValidationResult(
                        tool="RuntimeVerifier",
                        rule_id="INV001",
                        message=message,
                        file_path=file_path,
                        line_number=line_number,
                        severity="error",
                        code_snippet="",  # We don't have the code snippet here
                        suggestion="Review the code to ensure the invariant is maintained",
                        category="invariant"
                    )
                    self.add_result(result)
            except Exception as e:
                # Record an error if the invariant check itself fails
                result = ValidationResult(
                    tool="RuntimeVerifier",
                    rule_id="INV002",
                    message=f"Error checking invariant '{name}': {str(e)}",
                    file_path=file_path,
                    line_number=line_number,
                    severity="error",
                    code_snippet="",
                    suggestion="Fix the invariant check function",
                    category="invariant"
                )
                self.add_result(result)


class ExceptionMonitor(RuntimeMonitor):
    """Monitor for uncaught exceptions."""
    
    def __init__(self):
        """Initialize the exception monitor."""
        super().__init__("ExceptionMonitor", "Monitors uncaught exceptions")
        self.original_excepthook = None
    
    def exception_handler(self, exc_type, exc_value, exc_traceback):
        """Custom exception handler that records exceptions."""
        if self.active:
            # Get the traceback information
            tb = traceback.extract_tb(exc_traceback)
            if tb:
                # Get the last frame, which is where the exception was raised
                frame = tb[-1]
                file_path, line_number, func_name, _ = frame
                
                # Create a validation result for the exception
                result = ValidationResult(
                    tool="RuntimeVerifier",
                    rule_id="EXC001",
                    message=f"Uncaught exception: {exc_type.__name__}: {str(exc_value)}",
                    file_path=file_path,
                    line_number=line_number,
                    severity="error",
                    code_snippet="",  # We don't have the code snippet here
                    suggestion="Add appropriate exception handling",
                    category="exception"
                )
                
                self.add_result(result)
        
        # Call the original handler
        if self.original_excepthook:
            self.original_excepthook(exc_type, exc_value, exc_traceback)
    
    def start(self):
        """Start monitoring exceptions."""
        super().start()
        # Save the original excepthook
        self.original_excepthook = sys.excepthook
        # Replace with our custom handler
        sys.excepthook = self.exception_handler
    
    def stop(self):
        """Stop monitoring exceptions."""
        super().stop()
        # Restore the original excepthook
        if self.original_excepthook:
            sys.excepthook = self.original_excepthook
            self.original_excepthook = None


class StateTransitionMonitor(RuntimeMonitor):
    """Monitor for state transitions."""
    
    def __init__(self):
        """Initialize the state transition monitor."""
        super().__init__("StateTransitionMonitor", "Monitors state transitions")
        self.states = {}  # Maps object IDs to their current states
        self.transitions = {}  # Maps (from_state, to_state) to allowed flag
    
    def add_state_transition_rule(self, from_state: str, to_state: str, allowed: bool = True):
        """
        Add a state transition rule.
        
        Args:
            from_state: Starting state
            to_state: Ending state
            allowed: Whether this transition is allowed
        """
        self.transitions[(from_state, to_state)] = allowed
    
    def set_state(self, obj_id: int, state: str, file_path: str, line_number: int):
        """
        Set the state of an object and check if the transition is allowed.
        
        Args:
            obj_id: ID of the object
            state: New state
            file_path: Path to the file where the state change is being made
            line_number: Line number where the state change is being made
        """
        if not self.active:
            self.states[obj_id] = state
            return
        
        prev_state = self.states.get(obj_id)
        if prev_state is not None and prev_state != state:
            # Check if this transition is allowed
            allowed = self.transitions.get((prev_state, state), True)  # Default to allowed
            if not allowed:
                result = ValidationResult(
                    tool="RuntimeVerifier",
                    rule_id="STATE001",
                    message=f"Invalid state transition from '{prev_state}' to '{state}'",
                    file_path=file_path,
                    line_number=line_number,
                    severity="error",
                    code_snippet="",  # We don't have the code snippet here
                    suggestion="Review the state transition logic",
                    category="state_transition"
                )
                self.add_result(result)
        
        # Update the state
        self.states[obj_id] = state


class TemporalPropertyMonitor(RuntimeMonitor):
    """Monitor for temporal properties."""
    
    def __init__(self):
        """Initialize the temporal property monitor."""
        super().__init__("TemporalPropertyMonitor", "Monitors temporal properties")
        self.events = []  # List of (event_name, timestamp) tuples
        self.properties = []  # List of (property_name, check_func) tuples
    
    def record_event(self, event_name: str):
        """
        Record an event.
        
        Args:
            event_name: Name of the event
        """
        if self.active:
            self.events.append((event_name, time.time()))
    
    def add_property(self, name: str, check_func: Callable[[List[Tuple[str, float]]], bool], message: str = None):
        """
        Add a temporal property to check.
        
        Args:
            name: Name of the property
            check_func: Function that takes the event list and returns True if the property holds
            message: Optional message to include in the result
        """
        self.properties.append((name, check_func, message or f"Temporal property '{name}' violated"))
    
    def check_properties(self, file_path: str, line_number: int):
        """
        Check all registered temporal properties.
        
        Args:
            file_path: Path to the file where the check is being made
            line_number: Line number where the check is being made
        """
        if not self.active:
            return
        
        for name, check_func, message in self.properties:
            try:
                if not check_func(self.events):
                    result = ValidationResult(
                        tool="RuntimeVerifier",
                        rule_id="TEMP001",
                        message=message,
                        file_path=file_path,
                        line_number=line_number,
                        severity="error",
                        code_snippet="",  # We don't have the code snippet here
                        suggestion="Review the event sequence to ensure the property is satisfied",
                        category="temporal_property"
                    )
                    self.add_result(result)
            except Exception as e:
                # Record an error if the property check itself fails
                result = ValidationResult(
                    tool="RuntimeVerifier",
                    rule_id="TEMP002",
                    message=f"Error checking temporal property '{name}': {str(e)}",
                    file_path=file_path,
                    line_number=line_number,
                    severity="error",
                    code_snippet="",
                    suggestion="Fix the property check function",
                    category="temporal_property"
                )
                self.add_result(result)


class RuntimeVerifier:
    """Runtime verification system for Anarchy Inference."""
    
    def __init__(self):
        """Initialize the runtime verifier with default monitors."""
        self.monitors = {}
        self._initialize_default_monitors()
    
    def _initialize_default_monitors(self):
        """Initialize the default set of monitors."""
        self.monitors["assertion"] = AssertionMonitor()
        self.monitors["invariant"] = InvariantMonitor()
        self.monitors["exception"] = ExceptionMonitor()
        self.monitors["state_transition"] = StateTransitionMonitor()
        self.monitors["temporal_property"] = TemporalPropertyMonitor()
    
    def add_monitor(self, name: str, monitor: RuntimeMonitor):
        """Add a custom monitor to the verifier."""
        self.monitors[name] = monitor
    
    def start_monitoring(self, monitor_names: List[str] = None):
        """
        Start monitoring with the specified monitors.
        
        Args:
            monitor_names: List of monitor names to start, or None for all
        """
        if monitor_names is None:
            monitor_names = list(self.monitors.keys())
        
        for name in monitor_names:
            if name in self.monitors:
                self.monitors[name].start()
    
    def stop_monitoring(self, monitor_names: List[str] = None):
        """
        Stop monitoring with the specified monitors.
        
        Args:
            monitor_names: List of monitor names to stop, or None for all
        """
        if monitor_names is None:
            monitor_names = list(self.monitors.keys())
        
        for name in monitor_names:
            if name in self.monitors:
                self.monitors[name].stop()
    
    def get_results(self, monitor_names: List[str] = None) -> List[ValidationResult]:
        """
        Get results from the specified monitors.
        
        Args:
            monitor_names: List of monitor names to get results from, or None for all
            
        Returns:
            List of ValidationResult objects
        """
        if monitor_names is None:
            monitor_names = list(self.monitors.keys())
        
        results = []
        for name in monitor_names:
            if name in self.monitors:
                results.extend(self.monitors[name].report())
        
        return results
    
    def generate_report(self, results: List[ValidationResult], output_format: str = "text") -> str:
        """
        Generate a report from the verification results.
        
        Args:
            results: List of ValidationResult objects
            output_format: Format of the report (text, json, html)
            
        Returns:
            Report as a string
        """
        if output_format == "json":
            return json.dumps([r.to_dict() for r in results], indent=2)
        
        elif output_format == "html":
            html = """
            <!DOCTYPE html>
            <html>
            <head>
                <title>Runtime Verification Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    .result { margin-bottom: 20px; padding: 10px; border: 1px solid #ddd; }
                    .error { border-left: 5px solid #f44336; }
                    .warning { border-left: 5px solid #ff9800; }
                    .info { border-left: 5px solid #2196F3; }
                    .critical { border-left: 5px solid #9c27b0; }
                    .suggestion { background-color: #e8f5e9; padding: 10px; margin-top: 10px; }
                </style>
            </head>
            <body>
                <h1>Runtime Verification Report</h1>
                <p>Total issues found: {}</p>
            """.format(len(results))
            
            for result in results:
                html += """
                <div class="result {}">
                    <h3>{} - {}</h3>
                    <p><strong>File:</strong> {}:{}</p>
                    <p><strong>Rule:</strong> {}</p>
                    <p><strong>Category:</strong> {}</p>
                """.format(
                    result.severity,
                    result.severity.upper(),
                    result.message,
                    result.file_path,
                    result.line_number,
                    result.rule_id,
                    result.category
                )
                
                if result.suggestion:
                    html += """
                    <div class="suggestion">
                        <strong>Suggestion:</strong> {}
                    </div>
                    """.format(result.suggestion)
                
                html += "</div>"
            
            html += """
            </body>
            </html>
            """
            
            return html
        
        else:  # text format
            report = "Runtime Verification Report\n"
            report += "===========================\n\n"
            report += f"Total issues found: {len(results)}\n\n"
            
            for result in results:
                report += f"[{result.severity.upper()}] {result.file_path}:{result.line_number} - {result.message}\n"
                report += f"Rule: {result.rule_id} ({result.category})\n"
                if result.suggestion:
                    report += f"Suggestion: {result.suggestion}\n"
                report += "\n"
            
            return report


# Example usage
if __name__ == "__main__":
    import sys
    
    verifier = RuntimeVerifier()
    
    # Start monitoring
    verifier.start_monitoring()
    
    # Example: Add an invariant
    inv_monitor = verifier.monitors["invariant"]
    inv_monitor.add_invariant(
        "positive_balance",
        lambda: account_balance >= 0,
        "Account balance must remain positive"
    )
    
    # Example: Record state transitions
    state_monitor = verifier.monitors["state_transition"]
    state_monitor.add_state_transition_rule("initialized", "running", True)
    state_monitor.add_state_transition_rule("running", "stopped", True)
    state_monitor.add_state_transition_rule("stopped", "initialized", False)
    
    # Run your program...
    
    # Stop monitoring
    verifier.stop_monitoring()
    
    # Get and report results
    results = verifier.get_results()
    report = verifier.generate_report(results, "text")
    print(report)
