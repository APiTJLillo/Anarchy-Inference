"""
Concurrency Stress Tester for Anarchy Inference

This module provides classes for testing the system's behavior under high concurrency conditions,
including thread management, contention generation, and race condition detection.
"""

import os
import sys
import time
import threading
import multiprocessing
from typing import Dict, List, Any, Optional, Callable
from dataclasses import dataclass
import random

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")

class ThreadManager:
    """Creates and manages multiple execution threads for concurrency testing."""
    
    def __init__(self, interpreter):
        """Initialize the thread manager.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
        """
        self.interpreter = interpreter
        self.threads = []
        self.results = {}
        self.lock = threading.Lock()
    
    def create_threads(self, code: str, thread_count: int) -> List[threading.Thread]:
        """Create multiple threads executing the same code.
        
        Args:
            code: The Anarchy Inference code to execute
            thread_count: The number of threads to create
            
        Returns:
            A list of created threads
        """
        self.threads = []
        self.results = {}
        
        for i in range(thread_count):
            thread = threading.Thread(
                target=self._execute_code,
                args=(code, i),
                name=f"AnarchyThread-{i}"
            )
            self.threads.append(thread)
        
        return self.threads
    
    def create_threads_with_different_code(self, code_list: List[str]) -> List[threading.Thread]:
        """Create threads executing different code snippets.
        
        Args:
            code_list: List of Anarchy Inference code snippets to execute
            
        Returns:
            A list of created threads
        """
        self.threads = []
        self.results = {}
        
        for i, code in enumerate(code_list):
            thread = threading.Thread(
                target=self._execute_code,
                args=(code, i),
                name=f"AnarchyThread-{i}"
            )
            self.threads.append(thread)
        
        return self.threads
    
    def start_all_threads(self):
        """Start all created threads."""
        for thread in self.threads:
            thread.start()
    
    def wait_for_all_threads(self, timeout: Optional[float] = None) -> Dict[int, Any]:
        """Wait for all threads to complete and return their results.
        
        Args:
            timeout: Maximum time to wait for each thread in seconds
            
        Returns:
            A dictionary mapping thread IDs to their execution results
        """
        for thread in self.threads:
            thread.join(timeout=timeout)
        
        return self.results
    
    def _execute_code(self, code: str, thread_id: int):
        """Execute Anarchy Inference code in a thread and store the result.
        
        Args:
            code: The code to execute
            thread_id: The ID of the thread
        """
        try:
            # Create a new interpreter instance for each thread to avoid conflicts
            thread_interpreter = anarchy.Interpreter()
            result = thread_interpreter.execute(code)
            
            # Store the result
            with self.lock:
                self.results[thread_id] = {
                    "success": True,
                    "result": result
                }
        
        except Exception as e:
            # Store the error
            with self.lock:
                self.results[thread_id] = {
                    "success": False,
                    "error": str(e)
                }

class ContentionGenerator:
    """Creates resource contention scenarios for concurrency testing."""
    
    def __init__(self, interpreter):
        """Initialize the contention generator.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
        """
        self.interpreter = interpreter
    
    def generate_memory_contention(self, thread_count: int, memory_per_thread_mb: int) -> str:
        """Generate code that creates memory contention between threads.
        
        Args:
            thread_count: Number of threads to create
            memory_per_thread_mb: Memory to allocate per thread in MB
            
        Returns:
            Anarchy Inference code that creates memory contention
        """
        # Calculate array size based on memory per thread
        # Assuming each number takes about 8 bytes
        array_size = (memory_per_thread_mb * 1024 * 1024) // 8
        
        code = f"""
        // Memory Contention Test
        
        // Shared memory pool
        shared_pool ← []
        
        // Mutex for synchronization
        mutex ← create_mutex()
        
        // Worker function
        λ⟨ worker, id ⟩
            // Allocate memory
            local_array ← []
            for i in range({array_size}) {{
                local_array.push(i)
            }}
            
            // Access shared memory pool
            mutex.lock()
            shared_pool.push(local_array)
            mutex.unlock()
            
            // Perform operations on the array
            sum ← 0
            for i in range(local_array.length()) {{
                sum ← sum + local_array[i]
            }}
            
            return {{
                "thread_id": id,
                "array_size": local_array.length(),
                "sum": sum
            }}
        
        // Create and start threads
        threads ← []
        for i in range({thread_count}) {{
            thread ← create_thread(worker, i)
            threads.push(thread)
        }}
        
        // Wait for all threads to complete
        results ← []
        for thread in threads {{
            result ← thread.join()
            results.push(result)
        }}
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "memory_per_thread_mb": {memory_per_thread_mb},
            "shared_pool_size": shared_pool.length(),
            "thread_results": results
        }}
        """
        
        return code
    
    def generate_cpu_contention(self, thread_count: int, work_iterations: int) -> str:
        """Generate code that creates CPU contention between threads.
        
        Args:
            thread_count: Number of threads to create
            work_iterations: Number of computational iterations per thread
            
        Returns:
            Anarchy Inference code that creates CPU contention
        """
        code = f"""
        // CPU Contention Test
        
        // Worker function
        λ⟨ worker, id ⟩
            start_time ← time()
            
            // Perform CPU-intensive work
            result ← 0
            for i in range({work_iterations}) {{
                // Compute prime factorization
                n ← (i * id) % 10000 + 1000
                factors ← []
                
                while n % 2 == 0 {{
                    factors.push(2)
                    n ← n / 2
                }}
                
                factor ← 3
                while factor * factor <= n {{
                    while n % factor == 0 {{
                        factors.push(factor)
                        n ← n / factor
                    }}
                    factor ← factor + 2
                }}
                
                if n > 1 {{
                    factors.push(n)
                }}
                
                result ← result + factors.length()
            }}
            
            end_time ← time()
            
            return {{
                "thread_id": id,
                "result": result,
                "execution_time": end_time - start_time
            }}
        
        // Create and start threads
        threads ← []
        for i in range({thread_count}) {{
            thread ← create_thread(worker, i)
            threads.push(thread)
        }}
        
        // Wait for all threads to complete
        results ← []
        for thread in threads {{
            result ← thread.join()
            results.push(result)
        }}
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "work_iterations": {work_iterations},
            "thread_results": results
        }}
        """
        
        return code
    
    def generate_lock_contention(self, thread_count: int, lock_operations: int) -> str:
        """Generate code that creates lock contention between threads.
        
        Args:
            thread_count: Number of threads to create
            lock_operations: Number of lock operations per thread
            
        Returns:
            Anarchy Inference code that creates lock contention
        """
        code = f"""
        // Lock Contention Test
        
        // Shared counters
        counters ← []
        for i in range(10) {{
            counters.push(0)
        }}
        
        // Mutexes for synchronization
        mutexes ← []
        for i in range(10) {{
            mutexes.push(create_mutex())
        }}
        
        // Worker function
        λ⟨ worker, id ⟩
            local_sum ← 0
            lock_times ← []
            
            for i in range({lock_operations}) {{
                // Select a random counter and mutex
                index ← (i * id) % 10
                
                // Measure lock acquisition time
                lock_start ← time()
                mutexes[index].lock()
                lock_end ← time()
                lock_times.push(lock_end - lock_start)
                
                // Update counter
                counters[index] ← counters[index] + 1
                local_sum ← local_sum + counters[index]
                
                // Release lock
                mutexes[index].unlock()
                
                // Do some work between lock operations
                for j in range(100) {{
                    local_sum ← local_sum + j
                }}
            }}
            
            // Calculate average lock time
            avg_lock_time ← 0
            if lock_times.length() > 0 {{
                avg_lock_time ← sum(lock_times) / lock_times.length()
            }}
            
            return {{
                "thread_id": id,
                "local_sum": local_sum,
                "avg_lock_time": avg_lock_time,
                "max_lock_time": max(lock_times)
            }}
        
        // Create and start threads
        threads ← []
        for i in range({thread_count}) {{
            thread ← create_thread(worker, i)
            threads.push(thread)
        }}
        
        // Wait for all threads to complete
        results ← []
        for thread in threads {{
            result ← thread.join()
            results.push(result)
        }}
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "lock_operations": {lock_operations},
            "final_counters": counters,
            "expected_total": {thread_count} * {lock_operations},
            "thread_results": results
        }}
        """
        
        return code

class RaceConditionDetector:
    """Identifies potential race conditions in concurrent code."""
    
    def __init__(self, interpreter):
        """Initialize the race condition detector.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
        """
        self.interpreter = interpreter
    
    def generate_race_condition_test(self, thread_count: int, iterations: int) -> str:
        """Generate code that is likely to trigger race conditions.
        
        Args:
            thread_count: Number of threads to create
            iterations: Number of iterations per thread
            
        Returns:
            Anarchy Inference code that may trigger race conditions
        """
        code = f"""
        // Race Condition Test
        
        // Shared counter without synchronization
        counter ← 0
        
        // Shared counter with synchronization
        protected_counter ← 0
        mutex ← create_mutex()
        
        // Worker function
        λ⟨ worker, id ⟩
            local_counter ← 0
            
            for i in range({iterations}) {{
                // Update unprotected counter (potential race condition)
                counter ← counter + 1
                
                // Update protected counter (with synchronization)
                mutex.lock()
                protected_counter ← protected_counter + 1
                mutex.unlock()
                
                // Update local counter
                local_counter ← local_counter + 1
            }}
            
            return {{
                "thread_id": id,
                "local_counter": local_counter
            }}
        
        // Create and start threads
        threads ← []
        for i in range({thread_count}) {{
            thread ← create_thread(worker, i)
            threads.push(thread)
        }}
        
        // Wait for all threads to complete
        results ← []
        for thread in threads {{
            result ← thread.join()
            results.push(result)
        }}
        
        // Check for race conditions
        expected_total ← {thread_count} * {iterations}
        race_detected ← counter != expected_total
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "iterations": {iterations},
            "expected_total": expected_total,
            "unprotected_counter": counter,
            "protected_counter": protected_counter,
            "race_condition_detected": race_detected,
            "thread_results": results
        }}
        """
        
        return code
    
    def detect_race_conditions(self, code: str, thread_count: int, runs: int = 5) -> Dict[str, Any]:
        """Run code multiple times to detect race conditions.
        
        Args:
            code: The Anarchy Inference code to test
            thread_count: Number of threads to use
            runs: Number of times to run the test
            
        Returns:
            Results indicating whether race conditions were detected
        """
        results = []
        
        for run in range(runs):
            # Create a thread manager
            thread_manager = ThreadManager(self.interpreter)
            
            # Create threads
            thread_manager.create_threads(code, thread_count)
            
            # Start all threads
            thread_manager.start_all_threads()
            
            # Wait for threads to complete
            run_results = thread_manager.wait_for_all_threads()
            
            # Store results
            results.append(run_results)
        
        # Analyze results for inconsistencies that indicate race conditions
        inconsistencies = self._analyze_results(results)
        
        return {
            "runs": runs,
            "thread_count": thread_count,
            "race_conditions_detected": len(inconsistencies) > 0,
            "inconsistencies": inconsistencies
        }
    
    def _analyze_results(self, results: List[Dict[int, Any]]) -> List[Dict[str, Any]]:
        """Analyze results from multiple runs to detect inconsistencies.
        
        Args:
            results: Results from multiple runs
            
        Returns:
            List of detected inconsistencies
        """
        inconsistencies = []
        
        # Compare results across runs
        if len(results) < 2:
            return inconsistencies
        
        # Extract values to compare
        values_to_compare = {}
        
        for run_index, run_results in enumerate(results):
            for thread_id, thread_result in run_results.items():
                if not thread_result["success"]:
                    continue
                
                result = thread_result["result"]
                
                # Check if result is a dictionary with values we can compare
                if isinstance(result, dict):
                    for key, value in result.items():
                        if key not in values_to_compare:
                            values_to_compare[key] = []
                        
                        # Extend the list if needed
                        while len(values_to_compare[key]) <= run_index:
                            values_to_compare[key].append({})
                        
                        values_to_compare[key][run_index][thread_id] = value
        
        # Look for inconsistencies
        for key, runs_data in values_to_compare.items():
            # Skip keys that are expected to be different (like thread_id)
            if key in ["thread_id", "local_counter", "execution_time"]:
                continue
            
            # Compare values across runs
            reference_values = {}
            
            for run_index, run_data in enumerate(runs_data):
                for thread_id, value in run_data.items():
                    if thread_id not in reference_values:
                        reference_values[thread_id] = value
                    elif reference_values[thread_id] != value:
                        # Inconsistency detected
                        inconsistencies.append({
                            "key": key,
                            "thread_id": thread_id,
                            "run_index": run_index,
                            "expected": reference_values[thread_id],
                            "actual": value
                        })
        
        return inconsistencies

class DeadlockDetector:
    """Detects deadlock situations in concurrent code."""
    
    def __init__(self, interpreter):
        """Initialize the deadlock detector.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
        """
        self.interpreter = interpreter
    
    def generate_deadlock_test(self, thread_count: int = 2) -> str:
        """Generate code that may cause deadlocks.
        
        Args:
            thread_count: Number of threads to create (default is 2 for deadlock)
            
        Returns:
            Anarchy Inference code that may cause deadlocks
        """
        code = f"""
        // Deadlock Test
        
        // Create mutexes
        mutex_a ← create_mutex()
        mutex_b ← create_mutex()
        
        // Worker function that acquires locks in different orders
        λ⟨ worker, id ⟩
            if id % 2 == 0 {{
                // Even threads: acquire A then B
                print("Thread " + id.to_string() + " acquiring mutex A")
                mutex_a.lock()
                
                // Sleep to increase chance of deadlock
                sleep(100)
                
                print("Thread " + id.to_string() + " acquiring mutex B")
                mutex_b.lock()
                
                // Critical section
                result ← "Thread " + id.to_string() + " completed"
                
                // Release locks in reverse order
                mutex_b.unlock()
                mutex_a.unlock()
            }} else {{
                // Odd threads: acquire B then A
                print("Thread " + id.to_string() + " acquiring mutex B")
                mutex_b.lock()
                
                // Sleep to increase chance of deadlock
                sleep(100)
                
                print("Thread " + id.to_string() + " acquiring mutex A")
                mutex_a.lock()
                
                // Critical section
                result ← "Thread " + id.to_string() + " completed"
                
                // Release locks in reverse order
                mutex_a.unlock()
                mutex_b.unlock()
            }}
            
            return {{
                "thread_id": id,
                "completed": true
            }}
        
        // Create threads with timeout to prevent infinite waiting
        threads ← []
        for i in range({thread_count}) {{
            thread ← create_thread_with_timeout(worker, i, 5000)  // 5 second timeout
            threads.push(thread)
        }}
        
        // Wait for threads to complete
        results ← []
        deadlock_detected ← false
        
        for thread in threads {{
            try {{
                result ← thread.join()
                results.push(result)
            }} catch (e) {{
                // Timeout exception indicates potential deadlock
                deadlock_detected ← true
                results.push({{
                    "thread_id": thread.id,
                    "completed": false,
                    "error": "Timeout - potential deadlock"
                }})
            }}
        }}
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "deadlock_detected": deadlock_detected,
            "thread_results": results
        }}
        """
        
        return code
    
    def detect_deadlocks(self, code: str, timeout: int = 10) -> Dict[str, Any]:
        """Run code and detect if deadlocks occur.
        
        Args:
            code: The Anarchy Inference code to test
            timeout: Maximum execution time in seconds
            
        Returns:
            Results indicating whether deadlocks were detected
        """
        # Create a separate process to run the code with a timeout
        result_queue = multiprocessing.Queue()
        
        def run_code():
            try:
                result = self.interpreter.execute(code)
                result_queue.put({"success": True, "result": result})
            except Exception as e:
                result_queue.put({"success": False, "error": str(e)})
        
        process = multiprocessing.Process(target=run_code)
        process.start()
        
        # Wait for the process to complete or timeout
        process.join(timeout)
        
        # Check if the process is still running (indicating a deadlock)
        if process.is_alive():
            # Terminate the process
            process.terminate()
            process.join()
            
            return {
                "deadlock_detected": True,
                "timeout": timeout,
                "message": "Execution timed out, potential deadlock detected"
            }
        
        # Get the result
        if not result_queue.empty():
            result = result_queue.get()
            
            # Check if the code itself detected a deadlock
            if result["success"] and isinstance(result["result"], dict):
                code_detected_deadlock = result["result"].get("deadlock_detected", False)
                
                return {
                    "deadlock_detected": code_detected_deadlock,
                    "timeout": timeout,
                    "result": result["result"]
                }
            
            return {
                "deadlock_detected": False,
                "timeout": timeout,
                "result": result
            }
        
        return {
            "deadlock_detected": False,
            "timeout": timeout,
            "message": "No result returned"
        }

class SynchronizationTester:
    """Tests synchronization primitives under stress conditions."""
    
    def __init__(self, interpreter):
        """Initialize the synchronization tester.
        
        Args:
            interpreter: The Anarchy Inference interpreter instance
        """
        self.interpreter = interpreter
    
    def generate_mutex_stress_test(self, thread_count: int, operations: int) -> str:
        """Generate code that stress tests mutex operations.
        
        Args:
            thread_count: Number of threads to create
            operations: Number of mutex operations per thread
            
        Returns:
            Anarchy Inference code that stress tests mutexes
        """
        code = f"""
        // Mutex Stress Test
        
        // Shared counter
        counter ← 0
        
        // Mutex for synchronization
        mutex ← create_mutex()
        
        // Worker function
        λ⟨ worker, id ⟩
            local_counter ← 0
            lock_times ← []
            
            for i in range({operations}) {{
                // Measure lock acquisition time
                lock_start ← time()
                mutex.lock()
                lock_end ← time()
                lock_times.push(lock_end - lock_start)
                
                // Update counter
                counter ← counter + 1
                local_counter ← local_counter + 1
                
                // Release lock
                mutex.unlock()
            }}
            
            // Calculate average lock time
            avg_lock_time ← 0
            if lock_times.length() > 0 {{
                avg_lock_time ← sum(lock_times) / lock_times.length()
            }}
            
            return {{
                "thread_id": id,
                "local_counter": local_counter,
                "avg_lock_time": avg_lock_time,
                "max_lock_time": max(lock_times)
            }}
        
        // Create and start threads
        threads ← []
        for i in range({thread_count}) {{
            thread ← create_thread(worker, i)
            threads.push(thread)
        }}
        
        // Wait for all threads to complete
        results ← []
        for thread in threads {{
            result ← thread.join()
            results.push(result)
        }}
        
        // Check for correctness
        expected_total ← {thread_count} * {operations}
        correct ← counter == expected_total
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "operations": {operations},
            "expected_total": expected_total,
            "counter": counter,
            "correct": correct,
            "thread_results": results
        }}
        """
        
        return code
    
    def generate_condition_variable_test(self, producer_count: int, consumer_count: int, items: int) -> str:
        """Generate code that tests condition variables with producers and consumers.
        
        Args:
            producer_count: Number of producer threads
            consumer_count: Number of consumer threads
            items: Number of items to produce per producer
            
        Returns:
            Anarchy Inference code that tests condition variables
        """
        code = f"""
        // Condition Variable Test (Producer-Consumer)
        
        // Shared queue
        queue ← []
        
        // Synchronization primitives
        mutex ← create_mutex()
        not_empty ← create_condition_variable()
        not_full ← create_condition_variable()
        
        // Maximum queue size
        max_queue_size ← 10
        
        // Producer function
        λ⟨ producer, id ⟩
            items_produced ← 0
            
            for i in range({items}) {{
                // Create item
                item ← {{
                    "producer_id": id,
                    "item_id": i,
                    "value": id * 1000 + i
                }}
                
                // Add to queue with synchronization
                mutex.lock()
                
                // Wait if queue is full
                while queue.length() >= max_queue_size {{
                    not_full.wait(mutex)
                }}
                
                // Add item to queue
                queue.push(item)
                items_produced ← items_produced + 1
                
                // Signal that queue is not empty
                not_empty.signal()
                
                mutex.unlock()
            }}
            
            return {{
                "thread_id": id,
                "items_produced": items_produced
            }}
        
        // Consumer function
        λ⟨ consumer, id ⟩
            items_consumed ← 0
            consumed_items ← []
            
            // Consume until all items are produced and consumed
            total_items ← {producer_count} * {items}
            
            while items_consumed < total_items / {consumer_count} {{
                // Get item from queue with synchronization
                mutex.lock()
                
                // Wait if queue is empty
                while queue.length() == 0 {{
                    not_empty.wait(mutex)
                }}
                
                // Get item from queue
                item ← queue.shift()
                items_consumed ← items_consumed + 1
                consumed_items.push(item)
                
                // Signal that queue is not full
                not_full.signal()
                
                mutex.unlock()
            }}
            
            return {{
                "thread_id": id,
                "items_consumed": items_consumed,
                "consumed_items": consumed_items
            }}
        
        // Create and start producer threads
        producer_threads ← []
        for i in range({producer_count}) {{
            thread ← create_thread(producer, i)
            producer_threads.push(thread)
        }}
        
        // Create and start consumer threads
        consumer_threads ← []
        for i in range({consumer_count}) {{
            thread ← create_thread(consumer, i)
            consumer_threads.push(thread)
        }}
        
        // Wait for all threads to complete
        producer_results ← []
        for thread in producer_threads {{
            result ← thread.join()
            producer_results.push(result)
        }}
        
        consumer_results ← []
        for thread in consumer_threads {{
            result ← thread.join()
            consumer_results.push(result)
        }}
        
        // Check for correctness
        total_produced ← 0
        for result in producer_results {{
            total_produced ← total_produced + result["items_produced"]
        }}
        
        total_consumed ← 0
        for result in consumer_results {{
            total_consumed ← total_consumed + result["items_consumed"]
        }}
        
        correct ← total_produced == total_consumed
        
        // Return results
        return {{
            "producer_count": {producer_count},
            "consumer_count": {consumer_count},
            "items_per_producer": {items},
            "total_produced": total_produced,
            "total_consumed": total_consumed,
            "correct": correct,
            "producer_results": producer_results,
            "consumer_results": consumer_results
        }}
        """
        
        return code
