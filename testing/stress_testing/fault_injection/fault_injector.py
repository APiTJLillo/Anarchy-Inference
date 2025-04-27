#!/usr/bin/env python3
"""
Fault Injector for Anarchy Inference.

This module provides functionality for deliberately introducing faults to test
the resilience and error handling capabilities of Anarchy Inference.
"""

import os
import time
import threading
import logging
import random
import json
from typing import Dict, List, Any, Optional, Tuple, Callable, Union
from enum import Enum
from dataclasses import dataclass
import subprocess
import signal
import tempfile
import traceback

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("fault_injector")

class FaultType(Enum):
    """Types of faults that can be injected."""
    MEMORY_CORRUPTION = "memory_corruption"  # Corrupt memory structures
    RESOURCE_EXHAUSTION = "resource_exhaustion"  # Exhaust system resources
    INVALID_INPUT = "invalid_input"  # Provide invalid input
    SYNTAX_ERROR = "syntax_error"  # Introduce syntax errors
    RUNTIME_ERROR = "runtime_error"  # Cause runtime errors
    TIMEOUT = "timeout"  # Force timeouts
    INTERRUPT = "interrupt"  # Send interrupts
    IO_ERROR = "io_error"  # Cause I/O errors


@dataclass
class FaultInjectionConfig:
    """Configuration for fault injection."""
    fault_types: List[FaultType]
    frequency: float  # Probability of injecting a fault (0.0 to 1.0)
    target_components: List[str]  # Components to target (e.g., "parser", "interpreter")
    seed: Optional[int] = None  # Random seed for reproducibility
    output_dir: Optional[str] = None  # Directory for output files


@dataclass
class FaultInjectionResult:
    """Results from fault injection testing."""
    config: FaultInjectionConfig
    injected_faults: List[Dict[str, Any]]  # Details of injected faults
    system_responses: List[Dict[str, Any]]  # System responses to faults
    recovery_success_rate: float  # Percentage of successful recoveries
    error_handling_quality: str  # Qualitative assessment of error handling
    issues_detected: List[str]  # Issues detected during testing


class FaultInjector:
    """Injects faults into Anarchy Inference to test resilience."""
    
    def __init__(self, config: FaultInjectionConfig):
        """Initialize the fault injector.
        
        Args:
            config: Configuration for fault injection
        """
        self.config = config
        self.injected_faults = []
        self.system_responses = []
        
        # Set random seed if specified
        if config.seed is not None:
            random.seed(config.seed)
        
        # Create output directory if specified
        if config.output_dir and not os.path.exists(config.output_dir):
            os.makedirs(config.output_dir)
    
    def run_tests(self, test_count: int = 100) -> FaultInjectionResult:
        """Run fault injection tests.
        
        Args:
            test_count: Number of tests to run
            
        Returns:
            Results of fault injection testing
        """
        logger.info(f"Starting fault injection testing with {test_count} tests")
        
        successful_recoveries = 0
        
        for i in range(test_count):
            # Decide whether to inject a fault
            if random.random() < self.config.frequency:
                # Select a fault type
                fault_type = random.choice(self.config.fault_types)
                
                # Select a target component
                target_component = random.choice(self.config.target_components)
                
                # Inject the fault
                fault_details = self._inject_fault(fault_type, target_component)
                
                if fault_details:
                    self.injected_faults.append(fault_details)
                    
                    # Test system response
                    response = self._test_system_response(fault_details)
                    self.system_responses.append(response)
                    
                    # Check if recovery was successful
                    if response.get("recovery_successful", False):
                        successful_recoveries += 1
            
            # Log progress
            if (i + 1) % 10 == 0:
                logger.info(f"Completed {i + 1}/{test_count} tests")
        
        # Calculate recovery success rate
        recovery_success_rate = 0.0
        if self.injected_faults:
            recovery_success_rate = successful_recoveries / len(self.injected_faults)
        
        # Assess error handling quality
        error_handling_quality = self._assess_error_handling()
        
        # Identify issues
        issues_detected = self._identify_issues()
        
        logger.info(f"Fault injection testing completed")
        logger.info(f"Injected {len(self.injected_faults)} faults")
        logger.info(f"Recovery success rate: {recovery_success_rate:.2%}")
        
        return FaultInjectionResult(
            config=self.config,
            injected_faults=self.injected_faults,
            system_responses=self.system_responses,
            recovery_success_rate=recovery_success_rate,
            error_handling_quality=error_handling_quality,
            issues_detected=issues_detected
        )
    
    def _inject_fault(self, fault_type: FaultType, target_component: str) -> Dict[str, Any]:
        """Inject a fault into the system.
        
        Args:
            fault_type: Type of fault to inject
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        try:
            logger.info(f"Injecting {fault_type.value} fault into {target_component}")
            
            fault_details = {
                "type": fault_type.value,
                "target_component": target_component,
                "timestamp": time.time(),
                "details": {}
            }
            
            # Inject the fault based on its type
            if fault_type == FaultType.MEMORY_CORRUPTION:
                fault_details["details"] = self._inject_memory_corruption(target_component)
            
            elif fault_type == FaultType.RESOURCE_EXHAUSTION:
                fault_details["details"] = self._inject_resource_exhaustion(target_component)
            
            elif fault_type == FaultType.INVALID_INPUT:
                fault_details["details"] = self._inject_invalid_input(target_component)
            
            elif fault_type == FaultType.SYNTAX_ERROR:
                fault_details["details"] = self._inject_syntax_error(target_component)
            
            elif fault_type == FaultType.RUNTIME_ERROR:
                fault_details["details"] = self._inject_runtime_error(target_component)
            
            elif fault_type == FaultType.TIMEOUT:
                fault_details["details"] = self._inject_timeout(target_component)
            
            elif fault_type == FaultType.INTERRUPT:
                fault_details["details"] = self._inject_interrupt(target_component)
            
            elif fault_type == FaultType.IO_ERROR:
                fault_details["details"] = self._inject_io_error(target_component)
            
            return fault_details
        
        except Exception as e:
            logger.error(f"Error injecting fault: {e}")
            return None
    
    def _inject_memory_corruption(self, target_component: str) -> Dict[str, Any]:
        """Inject memory corruption.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        # Create a program that will cause memory corruption
        if target_component == "interpreter":
            # Create a program with a large recursive function that will overflow the stack
            program = """
            // Memory corruption test - stack overflow
            λ⟨ recursive_function ⟩(n)
                if n <= 0 {
                    return 0
                } else {
                    // Create a large local array to consume stack space
                    local_array ← []
                    for i in range(1000) {
                        local_array.push(i)
                    }
                    return recursive_function(n - 1) + 1
                }
            
            // Call with a large value to cause stack overflow
            result ← recursive_function(500)
            return result
            """
        
        elif target_component == "garbage_collector":
            # Create a program that will stress the garbage collector
            program = """
            // Memory corruption test - garbage collector stress
            λ⟨ create_cycle ⟩
                a ← {}
                b ← {}
                a.ref ← b
                b.ref ← a
                return [a, b]
            
            cycles ← []
            for i in range(10000) {
                cycles.push(create_cycle())
            }
            
            return cycles.length()
            """
        
        else:
            # Default memory corruption test
            program = """
            // Memory corruption test - large object allocation
            data ← []
            for i in range(1000000) {
                data.push(i)
            }
            return data.length()
            """
        
        return {
            "method": "program_execution",
            "program": program
        }
    
    def _inject_resource_exhaustion(self, target_component: str) -> Dict[str, Any]:
        """Inject resource exhaustion.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        if target_component == "memory_manager":
            # Create a program that allocates a lot of memory
            program = """
            // Resource exhaustion test - memory allocation
            data ← []
            for i in range(10000000) {
                data.push(i.to_string())
            }
            return data.length()
            """
        
        elif target_component == "file_system":
            # Create a program that opens many files
            program = """
            // Resource exhaustion test - file handles
            files ← []
            for i in range(1000) {
                file_name ← "/tmp/test_file_" + i.to_string() + ".txt"
                fs.write_file(file_name, "Test content")
                files.push(file_name)
            }
            
            // Read all files
            content ← ""
            for file in files {
                content ← content + fs.read_file(file)
            }
            
            // Clean up
            for file in files {
                fs.remove(file)
            }
            
            return files.length()
            """
        
        else:
            # Default resource exhaustion test
            program = """
            // Resource exhaustion test - CPU intensive
            result ← 0
            for i in range(100000000) {
                result ← (result + i) % 1000000007
            }
            return result
            """
        
        return {
            "method": "program_execution",
            "program": program
        }
    
    def _inject_invalid_input(self, target_component: str) -> Dict[str, Any]:
        """Inject invalid input.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        if target_component == "parser":
            # Create a program with invalid syntax
            program = """
            // Invalid input test - parser
            x ← 10
            y ← 20
            // Missing closing brace
            if x > y {
                z ← x + y
            
            return z
            """
        
        elif target_component == "type_system":
            # Create a program with type errors
            program = """
            // Invalid input test - type system
            x ← "hello"
            y ← 10
            z ← x + y  // String + number
            return z
            """
        
        else:
            # Default invalid input test
            program = """
            // Invalid input test - general
            x ← 10
            y ← "20"
            z ← x * y  // Number * string
            return z
            """
        
        return {
            "method": "program_execution",
            "program": program
        }
    
    def _inject_syntax_error(self, target_component: str) -> Dict[str, Any]:
        """Inject syntax errors.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        if target_component == "lexer":
            # Create a program with lexer errors
            program = """
            // Syntax error test - lexer
            x ← 10
            y ← @invalid_token
            return x + y
            """
        
        elif target_component == "parser":
            # Create a program with parser errors
            program = """
            // Syntax error test - parser
            x ← 10
            if x > 5 {
                y ← 20
            } else {
                y ← 30
            } else {  // Invalid second else
                y ← 40
            }
            return y
            """
        
        else:
            # Default syntax error test
            program = """
            // Syntax error test - general
            x ← 10
            y ← 20
            return x + + y  // Double operator
            """
        
        return {
            "method": "program_execution",
            "program": program
        }
    
    def _inject_runtime_error(self, target_component: str) -> Dict[str, Any]:
        """Inject runtime errors.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        if target_component == "division":
            # Create a program with division by zero
            program = """
            // Runtime error test - division by zero
            x ← 10
            y ← 0
            z ← x / y
            return z
            """
        
        elif target_component == "array_access":
            # Create a program with array index out of bounds
            program = """
            // Runtime error test - array index out of bounds
            arr ← [1, 2, 3]
            value ← arr[10]
            return value
            """
        
        else:
            # Default runtime error test
            program = """
            // Runtime error test - undefined variable
            x ← 10
            y ← 20
            return z  // z is undefined
            """
        
        return {
            "method": "program_execution",
            "program": program
        }
    
    def _inject_timeout(self, target_component: str) -> Dict[str, Any]:
        """Inject timeouts.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        if target_component == "interpreter":
            # Create a program with an infinite loop
            program = """
            // Timeout test - infinite loop
            x ← 0
            while true {
                x ← x + 1
            }
            return x
            """
        
        elif target_component == "io_operations":
            # Create a program with slow I/O operations
            program = """
            // Timeout test - slow I/O
            for i in range(1000) {
                file_name ← "/tmp/timeout_test_" + i.to_string() + ".txt"
                fs.write_file(file_name, "Test content")
                content ← fs.read_file(file_name)
                fs.remove(file_name)
                // Add a sleep to slow down operations
                time.sleep(0.1)
            }
            return "Done"
            """
        
        else:
            # Default timeout test
            program = """
            // Timeout test - busy wait
            start ← time()
            while time() - start < 60 {
                // Busy wait for 60 seconds
            }
            return "Done"
            """
        
        return {
            "method": "program_execution_with_timeout",
            "program": program,
            "timeout": 5  # 5 second timeout
        }
    
    def _inject_interrupt(self, target_component: str) -> Dict[str, Any]:
        """Inject interrupts.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        # Create a program that will run for a while
        program = """
        // Interrupt test
        result ← 0
        for i in range(1000000) {
            result ← result + i
        }
        return result
        """
        
        return {
            "method": "program_execution_with_interrupt",
            "program": program,
            "interrupt_delay": 2  # Interrupt after 2 seconds
        }
    
    def _inject_io_error(self, target_component: str) -> Dict[str, Any]:
        """Inject I/O errors.
        
        Args:
            target_component: Component to target
            
        Returns:
            Details of the injected fault
        """
        if target_component == "file_system":
            # Create a program that tries to access a non-existent file
            program = """
            // I/O error test - file system
            content ← fs.read_file("/nonexistent/file.txt")
            return content
            """
        
        elif target_component == "network":
            # Create a program that tries to connect to a non-existent server
            program = """
            // I/O error test - network
            response ← http.get("http://nonexistent.server.local/")
            return response
            """
        
        else:
            # Default I/O error test
            program = """
            // I/O error test - general
            // Try to write to a read-only location
            fs.write_file("/etc/passwd", "test")
            return "Done"
            """
        
        return {
            "method": "program_execution",
            "program": program
        }
    
    def _test_system_response(self, fault_details: Dict[str, Any]) -> Dict[str, Any]:
        """Test the system's response to an injected fault.
        
        Args:
            fault_details: Details of the injected fault
            
        Returns:
            System response details
        """
        response = {
            "fault_type": fault_details["type"],
            "target_component": fault_details["target_component"],
            "timestamp": time.time(),
            "error_message": None,
            "exit_code": None,
            "execution_time": None,
            "recovery_successful": False
        }
        
        try:
            # Get the method and program
            method = fault_details["details"].get("method", "program_execution")
            program = fault_details["details"].get("program", "")
            
            # Write the program to a temporary file
            with tempfile.NamedTemporaryFile(suffix=".a.i", delete=False) as temp_file:
                temp_file_path = temp_file.name
                temp_file.write(program.encode("utf-8"))
            
            start_time = time.time()
            
            if method == "program_execution":
                # Run the program
                process = subprocess.Popen(
                    ["anarchy", temp_file_path],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE
                )
                
                stdout, stderr = process.communicate()
                
                response["exit_code"] = process.returncode
                response["stdout"] = stdout.decode("utf-8", errors="replace")
                response["stderr"] = stderr.decode("utf-8", errors="replace")
                
                # Check if error message is present
                if stderr:
                    response["error_message"] = stderr.decode("utf-8", errors="replace")
                
                # Check if recovery was successful
                response["recovery_successful"] = process.returncode == 0 or "recovered from error" in stderr.decode("utf-8", errors="replace").lower()
            
            elif method == "program_execution_with_timeout":
                timeout = fault_details["details"].get("timeout", 5)
                
                # Run the program with timeout
                process = subprocess.Popen(
                    ["anarchy", temp_file_path],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE
                )
                
                try:
                    stdout, stderr = process.communicate(timeout=timeout)
                    
                    response["exit_code"] = process.returncode
                    response["stdout"] = stdout.decode("utf-8", errors="replace")
                    response["stderr"] = stderr.decode("utf-8", errors="replace")
                    
                    # Check if error message is present
                    if stderr:
                        response["error_message"] = stderr.decode("utf-8", errors="replace")
                    
                    # Check if recovery was successful
                    response["recovery_successful"] = process.returncode == 0 or "recovered from error" in stderr.decode("utf-8", errors="replace").lower()
                
                except subprocess.TimeoutExpired:
                    # Timeout occurred
                    process.kill()
                    stdout, stderr = process.communicate()
                    
                    response["exit_code"] = -1
                    response["error_message"] = "Timeout expired"
                    response["recovery_successful"] = False
            
            elif method == "program_execution_with_interrupt":
                interrupt_delay = fault_details["details"].get("interrupt_delay", 2)
                
                # Run the program
                process = subprocess.Popen(
                    ["anarchy", temp_file_path],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE
                )
                
                # Wait for the specified delay
                time.sleep(interrupt_delay)
                
                # Send interrupt signal
                process.send_signal(signal.SIGINT)
                
                # Wait for process to complete
                stdout, stderr = process.communicate()
                
                response["exit_code"] = process.returncode
                response["stdout"] = stdout.decode("utf-8", errors="replace")
                response["stderr"] = stderr.decode("utf-8", errors="replace")
                
                # Check if error message is present
                if stderr:
                    response["error_message"] = stderr.decode("utf-8", errors="replace")
                
                # Check if recovery was successful
                response["recovery_successful"] = "handled interrupt" in stderr.decode("utf-8", errors="replace").lower()
            
            # Calculate execution time
            response["execution_time"] = time.time() - start_time
            
            # Clean up
            try:
                os.remove(temp_file_path)
            except:
                pass
        
        except Exception as e:
            response["error_message"] = str(e)
            response["recovery_successful"] = False
            logger.error(f"Error testing system response: {e}")
            traceback.print_exc()
        
        return response
    
    def _assess_error_handling(self) -> str:
        """Assess the quality of error handling.
        
        Returns:
            Qualitative assessment of error handling
        """
        if not self.system_responses:
            return "No data available"
        
        # Calculate recovery rate
        recovery_count = sum(1 for r in self.system_responses if r.get("recovery_successful", False))
        recovery_rate = recovery_count / len(self.system_responses)
        
        # Check error message quality
        error_message_count = sum(1 for r in self.system_responses if r.get("error_message"))
        error_message_rate = error_message_count / len(self.system_responses)
        
        # Assess based on recovery rate and error message quality
        if recovery_rate >= 0.9 and error_message_rate >= 0.9:
            return "Excellent"
        elif recovery_rate >= 0.7 and error_message_rate >= 0.7:
            return "Good"
        elif recovery_rate >= 0.5 and error_message_rate >= 0.5:
            return "Fair"
        else:
            return "Poor"
    
    def _identify_issues(self) -> List[str]:
        """Identify issues in error handling.
        
        Returns:
            List of identified issues
        """
        issues = []
        
        # Check for common issues
        for response in self.system_responses:
            fault_type = response.get("fault_type")
            recovery_successful = response.get("recovery_successful", False)
            error_message = response.get("error_message")
            
            if not recovery_successful:
                issues.append(f"Failed to recover from {fault_type} fault")
            
            if not error_message and not recovery_successful:
                issues.append(f"No error message for {fault_type} fault")
            
            if response.get("exit_code") == 0 and not recovery_successful:
                issues.append(f"Exit code 0 despite failure for {fault_type} fault")
        
        # Remove duplicates
        return list(set(issues))
    
    def generate_report(self, result: FaultInjectionResult) -> str:
        """Generate a report of the fault injection results.
        
        Args:
            result: Fault injection results
            
        Returns:
            Markdown-formatted report
        """
        # Create report header
        report = f"# Fault Injection Test Report\n\n"
        report += f"## Configuration\n\n"
        report += f"- **Fault Types**: {', '.join(t.value for t in self.config.fault_types)}\n"
        report += f"- **Frequency**: {self.config.frequency:.2f}\n"
        report += f"- **Target Components**: {', '.join(self.config.target_components)}\n\n"
        
        # Add summary
        report += f"## Summary\n\n"
        report += f"- **Injected Faults**: {len(result.injected_faults)}\n"
        report += f"- **Recovery Success Rate**: {result.recovery_success_rate:.2%}\n"
        report += f"- **Error Handling Quality**: {result.error_handling_quality}\n\n"
        
        # Add issues
        if result.issues_detected:
            report += f"## Issues Detected\n\n"
            for issue in result.issues_detected:
                report += f"- {issue}\n"
            report += "\n"
        
        # Add fault details
        report += f"## Fault Details\n\n"
        report += f"| Fault Type | Target Component | Recovery Successful |\n"
        report += f"|------------|------------------|---------------------|\n"
        
        for i, (fault, response) in enumerate(zip(result.injected_faults[:10], result.system_responses[:10])):
            fault_type = fault["type"]
            target_component = fault["target_component"]
            recovery_successful = response.get("recovery_successful", False)
            
            report += f"| {fault_type} | {target_component} | {'Yes' if recovery_successful else 'No'} |\n"
        
        if len(result.injected_faults) > 10:
            report += f"| ... | ... | ... |\n"
        
        # Add timestamp
        report += f"\n\nGenerated on: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        
        return report
    
    def save_results(self, result: FaultInjectionResult):
        """Save fault injection results to files.
        
        Args:
            result: Fault injection results
        """
        if not self.config.output_dir:
            return
        
        # Create timestamp
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        
        # Save report
        report_path = os.path.join(
            self.config.output_dir,
            f"fault_injection_report_{timestamp}.md"
        )
        with open(report_path, "w") as f:
            f.write(self.generate_report(result))
        
        # Save raw data
        data_path = os.path.join(
            self.config.output_dir,
            f"fault_injection_data_{timestamp}.json"
        )
        with open(data_path, "w") as f:
            # Convert result to dictionary
            result_dict = {
                "config": {
                    "fault_types": [t.value for t in self.config.fault_types],
                    "frequency": self.config.frequency,
                    "target_components": self.config.target_components
                },
                "injected_faults": result.injected_faults,
                "system_responses": result.system_responses,
                "recovery_success_rate": result.recovery_success_rate,
                "error_handling_quality": result.error_handling_quality,
                "issues_detected": result.issues_detected
            }
            json.dump(result_dict, f, indent=2)
        
        logger.info(f"Results saved to {report_path} and {data_path}")


def run_fault_injection_tests(config: FaultInjectionConfig, test_count: int = 100) -> FaultInjectionResult:
    """Run fault injection tests with the specified configuration.
    
    Args:
        config: Fault injection configuration
        test_count: Number of tests to run
        
    Returns:
        Fault injection results
    """
    injector = FaultInjector(config)
    result = injector.run_tests(test_count)
    injector.save_results(result)
    return result


def main():
    """Main entry point for the fault injection module."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Anarchy Inference Fault Injection")
    
    parser.add_argument(
        "--fault-types",
        choices=[t.value for t in FaultType],
        nargs="+",
        default=[t.value for t in FaultType],
        help="Types of faults to inject"
    )
    
    parser.add_argument(
        "--frequency",
        type=float,
        default=0.5,
        help="Probability of injecting a fault (0.0 to 1.0)"
    )
    
    parser.add_argument(
        "--target-components",
        nargs="+",
        default=["parser", "interpreter", "garbage_collector"],
        help="Components to target"
    )
    
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducibility"
    )
    
    parser.add_argument(
        "--output-dir",
        default="./results",
        help="Directory for output files"
    )
    
    parser.add_argument(
        "--test-count",
        type=int,
        default=100,
        help="Number of tests to run"
    )
    
    args = parser.parse_args()
    
    # Create configuration
    config = FaultInjectionConfig(
        fault_types=[FaultType(t) for t in args.fault_types],
        frequency=args.frequency,
        target_components=args.target_components,
        seed=args.seed,
        output_dir=args.output_dir
    )
    
    # Run the tests
    result = run_fault_injection_tests(config, args.test_count)
    
    # Print summary
    print(f"\nFault Injection Summary:")
    print(f"- Injected Faults: {len(result.injected_faults)}")
    print(f"- Recovery Success Rate: {result.recovery_success_rate:.2%}")
    print(f"- Error Handling Quality: {result.error_handling_quality}")
    
    if result.issues_detected:
        print(f"\nIssues Detected:")
        for issue in result.issues_detected:
            print(f"- {issue}")
    
    print(f"\nDetailed report available in: {config.output_dir}")
    
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
