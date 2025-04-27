"""
Stress Test Generator for Anarchy Inference

This module provides classes for generating stress tests that push the Anarchy Inference
interpreter to its limits, including memory-intensive, computation-intensive, and
concurrency-intensive tests.
"""

import os
import sys
import random
import string
from typing import Dict, List, Any, Optional, Callable
from enum import Enum

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")

class StressTestCategory(Enum):
    """Categories of stress tests that can be generated."""
    MEMORY = "memory"
    COMPUTATIONAL = "computational"
    CONCURRENCY = "concurrency"
    IO = "io"
    LONG_RUNNING = "long_running"

class StressIntensity(Enum):
    """Intensity levels for stress tests."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    EXTREME = "extreme"

class StressTestGenerator:
    """Generates stress tests for the Anarchy Inference interpreter."""
    
    def __init__(self, seed: Optional[int] = None):
        """Initialize the stress test generator.
        
        Args:
            seed: Random seed for reproducible test generation
        """
        if seed is not None:
            random.seed(seed)
    
    def generate_test(self, category: StressTestCategory, intensity: StressIntensity) -> str:
        """Generate a stress test of the specified category and intensity.
        
        Args:
            category: The category of stress test to generate
            intensity: The intensity level of the stress test
            
        Returns:
            The generated Anarchy Inference code for the stress test
        """
        if category == StressTestCategory.MEMORY:
            return self._generate_memory_stress_test(intensity)
        elif category == StressTestCategory.COMPUTATIONAL:
            return self._generate_computational_stress_test(intensity)
        elif category == StressTestCategory.CONCURRENCY:
            return self._generate_concurrency_stress_test(intensity)
        elif category == StressTestCategory.IO:
            return self._generate_io_stress_test(intensity)
        elif category == StressTestCategory.LONG_RUNNING:
            return self._generate_long_running_stress_test(intensity)
        else:
            raise ValueError(f"Unknown stress test category: {category}")
    
    def _generate_memory_stress_test(self, intensity: StressIntensity) -> str:
        """Generate a memory-intensive stress test.
        
        Args:
            intensity: The intensity level of the stress test
            
        Returns:
            The generated Anarchy Inference code for the memory stress test
        """
        # Determine test parameters based on intensity
        if intensity == StressIntensity.LOW:
            array_size = 1000
            object_count = 100
            nesting_depth = 5
        elif intensity == StressIntensity.MEDIUM:
            array_size = 10000
            object_count = 1000
            nesting_depth = 10
        elif intensity == StressIntensity.HIGH:
            array_size = 100000
            object_count = 10000
            nesting_depth = 20
        elif intensity == StressIntensity.EXTREME:
            array_size = 1000000
            object_count = 100000
            nesting_depth = 50
        
        # Generate the test code
        code = f"""
        // Memory Stress Test - {intensity.value.capitalize()} Intensity
        
        // Function to create a large array
        λ⟨ create_large_array ⟩
            size ← {array_size}
            array ← []
            for i in range(size) {{
                array.push(i)
            }}
            return array
        
        // Function to create a large object
        λ⟨ create_large_object ⟩
            count ← {object_count}
            obj ← {{}}
            for i in range(count) {{
                key ← "key_" + i.to_string()
                obj[key] ← i
            }}
            return obj
        
        // Function to create nested objects
        λ⟨ create_nested_object, depth ⟩
            if depth <= 0 {{
                return {{}}
            }}
            
            obj ← {{
                "value": depth,
                "nested": create_nested_object(depth - 1)
            }}
            
            return obj
        
        // Create large arrays
        arrays ← []
        for i in range(10) {{
            arrays.push(create_large_array())
        }}
        
        // Create large objects
        objects ← []
        for i in range(10) {{
            objects.push(create_large_object())
        }}
        
        // Create deeply nested object
        nested ← create_nested_object({nesting_depth})
        
        // Create circular references
        obj1 ← {{"name": "obj1"}}
        obj2 ← {{"name": "obj2"}}
        obj1["ref"] ← obj2
        obj2["ref"] ← obj1
        
        // Force garbage collection
        gc()
        
        // Return memory usage statistics
        return {{
            "arrays_length": arrays.length(),
            "objects_length": objects.length(),
            "nesting_depth": {nesting_depth}
        }}
        """
        
        return code
    
    def _generate_computational_stress_test(self, intensity: StressIntensity) -> str:
        """Generate a computation-intensive stress test.
        
        Args:
            intensity: The intensity level of the stress test
            
        Returns:
            The generated Anarchy Inference code for the computational stress test
        """
        # Determine test parameters based on intensity
        if intensity == StressIntensity.LOW:
            iterations = 100
            recursion_depth = 10
            matrix_size = 10
        elif intensity == StressIntensity.MEDIUM:
            iterations = 1000
            recursion_depth = 50
            matrix_size = 50
        elif intensity == StressIntensity.HIGH:
            iterations = 10000
            recursion_depth = 100
            matrix_size = 100
        elif intensity == StressIntensity.EXTREME:
            iterations = 100000
            recursion_depth = 500
            matrix_size = 200
        
        # Generate the test code
        code = f"""
        // Computational Stress Test - {intensity.value.capitalize()} Intensity
        
        // Recursive Fibonacci function
        λ⟨ fibonacci, n ⟩
            if n <= 1 {{
                return n
            }}
            return fibonacci(n - 1) + fibonacci(n - 2)
        
        // Matrix multiplication function
        λ⟨ matrix_multiply, a, b ⟩
            rows_a ← a.length()
            cols_a ← a[0].length()
            cols_b ← b[0].length()
            
            result ← []
            for i in range(rows_a) {{
                row ← []
                for j in range(cols_b) {{
                    sum ← 0
                    for k in range(cols_a) {{
                        sum ← sum + a[i][k] * b[k][j]
                    }}
                    row.push(sum)
                }}
                result.push(row)
            }}
            
            return result
        
        // Prime number calculation
        λ⟨ is_prime, n ⟩
            if n <= 1 {{
                return false
            }}
            if n <= 3 {{
                return true
            }}
            if n % 2 == 0 or n % 3 == 0 {{
                return false
            }}
            
            i ← 5
            while i * i <= n {{
                if n % i == 0 or n % (i + 2) == 0 {{
                    return false
                }}
                i ← i + 6
            }}
            
            return true
        
        // Create matrices
        λ⟨ create_matrix, size ⟩
            matrix ← []
            for i in range(size) {{
                row ← []
                for j in range(size) {{
                    row.push(i * j % 10)
                }}
                matrix.push(row)
            }}
            return matrix
        
        // Run computational tests
        start_time ← time()
        
        // Fibonacci calculation
        fib_result ← fibonacci({recursion_depth // 2})  // Using half depth to avoid excessive time
        
        // Matrix operations
        matrix_a ← create_matrix({matrix_size})
        matrix_b ← create_matrix({matrix_size})
        matrix_result ← matrix_multiply(matrix_a, matrix_b)
        
        // Prime number calculations
        prime_count ← 0
        for i in range({iterations}) {{
            if is_prime(i) {{
                prime_count ← prime_count + 1
            }}
        }}
        
        // Sorting
        λ⟨ quick_sort, arr, low, high ⟩
            if low < high {{
                // Partition
                pivot ← arr[high]
                i ← low - 1
                
                for j in range(low, high) {{
                    if arr[j] <= pivot {{
                        i ← i + 1
                        temp ← arr[i]
                        arr[i] ← arr[j]
                        arr[j] ← temp
                    }}
                }}
                
                temp ← arr[i + 1]
                arr[i + 1] ← arr[high]
                arr[high] ← temp
                
                pivot_index ← i + 1
                
                // Recursive sort
                quick_sort(arr, low, pivot_index - 1)
                quick_sort(arr, pivot_index + 1, high)
            }}
            
            return arr
        
        // Create and sort array
        array_to_sort ← []
        for i in range({iterations // 10}) {{
            array_to_sort.push(random(1000))
        }}
        
        sorted_array ← quick_sort(array_to_sort, 0, array_to_sort.length() - 1)
        
        end_time ← time()
        execution_time ← end_time - start_time
        
        // Return results
        return {{
            "fibonacci_result": fib_result,
            "matrix_size": {matrix_size},
            "prime_count": prime_count,
            "sorted_array_length": sorted_array.length(),
            "execution_time": execution_time
        }}
        """
        
        return code
    
    def _generate_concurrency_stress_test(self, intensity: StressIntensity) -> str:
        """Generate a concurrency-intensive stress test.
        
        Args:
            intensity: The intensity level of the stress test
            
        Returns:
            The generated Anarchy Inference code for the concurrency stress test
        """
        # Determine test parameters based on intensity
        if intensity == StressIntensity.LOW:
            thread_count = 5
            iterations = 100
        elif intensity == StressIntensity.MEDIUM:
            thread_count = 10
            iterations = 1000
        elif intensity == StressIntensity.HIGH:
            thread_count = 20
            iterations = 10000
        elif intensity == StressIntensity.EXTREME:
            thread_count = 50
            iterations = 100000
        
        # Generate the test code
        code = f"""
        // Concurrency Stress Test - {intensity.value.capitalize()} Intensity
        
        // Shared counter
        counter ← 0
        
        // Mutex for synchronization
        mutex ← create_mutex()
        
        // Worker function
        λ⟨ worker, id, iterations ⟩
            local_sum ← 0
            
            for i in range(iterations) {{
                // Some computational work
                value ← (i * id) % 100
                local_sum ← local_sum + value
                
                // Update shared counter with synchronization
                mutex.lock()
                counter ← counter + 1
                mutex.unlock()
                
                // Simulate some non-synchronized work
                for j in range(100) {{
                    local_sum ← local_sum + (j % 10)
                }}
            }}
            
            return local_sum
        
        // Create and start threads
        threads ← []
        results ← []
        
        for i in range({thread_count}) {{
            thread ← create_thread(worker, i, {iterations})
            threads.push(thread)
        }}
        
        // Wait for all threads to complete
        for thread in threads {{
            result ← thread.join()
            results.push(result)
        }}
        
        // Return results
        return {{
            "thread_count": {thread_count},
            "iterations_per_thread": {iterations},
            "final_counter": counter,
            "expected_counter": {thread_count} * {iterations},
            "thread_results": results
        }}
        """
        
        return code
    
    def _generate_io_stress_test(self, intensity: StressIntensity) -> str:
        """Generate an I/O-intensive stress test.
        
        Args:
            intensity: The intensity level of the stress test
            
        Returns:
            The generated Anarchy Inference code for the I/O stress test
        """
        # Determine test parameters based on intensity
        if intensity == StressIntensity.LOW:
            file_count = 5
            file_size_kb = 10
            operations = 100
        elif intensity == StressIntensity.MEDIUM:
            file_count = 10
            file_size_kb = 100
            operations = 1000
        elif intensity == StressIntensity.HIGH:
            file_count = 20
            file_size_kb = 1000
            operations = 10000
        elif intensity == StressIntensity.EXTREME:
            file_count = 50
            file_size_kb = 10000
            operations = 100000
        
        # Generate the test code
        code = f"""
        // I/O Stress Test - {intensity.value.capitalize()} Intensity
        
        // Function to generate random content
        λ⟨ generate_content, size_kb ⟩
            content ← ""
            chars ← "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
            
            for i in range(size_kb * 1024) {{
                index ← random(chars.length())
                content ← content + chars[index]
            }}
            
            return content
        
        // Create temporary directory
        temp_dir ← "/tmp/anarchy_stress_test_" + time().to_string()
        fs.mkdir(temp_dir)
        
        // Create files
        files ← []
        for i in range({file_count}) {{
            filename ← temp_dir + "/file_" + i.to_string() + ".txt"
            content ← generate_content({file_size_kb})
            fs.write_file(filename, content)
            files.push(filename)
        }}
        
        // Perform file operations
        operation_count ← 0
        
        for i in range({operations}) {{
            // Select random operation and file
            operation ← i % 4  // 0: read, 1: write, 2: append, 3: stat
            file_index ← i % files.length()
            filename ← files[file_index]
            
            if operation == 0 {{
                // Read operation
                content ← fs.read_file(filename)
                operation_count ← operation_count + 1
            }} else if operation == 1 {{
                // Write operation
                content ← generate_content(1)  // 1KB content
                fs.write_file(filename, content)
                operation_count ← operation_count + 1
            }} else if operation == 2 {{
                // Append operation
                content ← generate_content(1)  // 1KB content
                fs.append_file(filename, content)
                operation_count ← operation_count + 1
            }} else if operation == 3 {{
                // Stat operation
                stats ← fs.stat(filename)
                operation_count ← operation_count + 1
            }}
        }}
        
        // Clean up
        for filename in files {{
            fs.remove(filename)
        }}
        fs.rmdir(temp_dir)
        
        // Return results
        return {{
            "file_count": {file_count},
            "file_size_kb": {file_size_kb},
            "operations_performed": operation_count
        }}
        """
        
        return code
    
    def _generate_long_running_stress_test(self, intensity: StressIntensity) -> str:
        """Generate a long-running stress test.
        
        Args:
            intensity: The intensity level of the stress test
            
        Returns:
            The generated Anarchy Inference code for the long-running stress test
        """
        # Determine test parameters based on intensity
        if intensity == StressIntensity.LOW:
            iterations = 1000
            sleep_ms = 10
        elif intensity == StressIntensity.MEDIUM:
            iterations = 10000
            sleep_ms = 5
        elif intensity == StressIntensity.HIGH:
            iterations = 100000
            sleep_ms = 2
        elif intensity == StressIntensity.EXTREME:
            iterations = 1000000
            sleep_ms = 1
        
        # Generate the test code
        code = f"""
        // Long-Running Stress Test - {intensity.value.capitalize()} Intensity
        
        // Initialize state
        state ← {{
            "iteration": 0,
            "memory_usage": [],
            "checkpoints": [],
            "start_time": time()
        }}
        
        // Function to create a checkpoint
        λ⟨ create_checkpoint ⟩
            checkpoint ← {{
                "iteration": state["iteration"],
                "time": time(),
                "memory": memory_usage()
            }}
            
            state["checkpoints"].push(checkpoint)
            return checkpoint
        
        // Main loop
        for i in range({iterations}) {{
            state["iteration"] ← i
            
            // Perform some work
            result ← 0
            for j in range(1000) {{
                result ← result + (j * i) % 100
            }}
            
            // Create checkpoint every 10% of iterations
            if i % ({iterations // 10}) == 0 {{
                create_checkpoint()
                
                // Record memory usage
                state["memory_usage"].push(memory_usage())
                
                // Log progress
                elapsed ← time() - state["start_time"]
                print("Iteration " + i.to_string() + " / {iterations} - Elapsed: " + elapsed.to_string() + "s")
            }}
            
            // Sleep to extend duration
            sleep({sleep_ms})
        }}
        
        // Calculate statistics
        end_time ← time()
        total_time ← end_time - state["start_time"]
        
        // Check for memory growth
        memory_growth ← 0
        if state["memory_usage"].length() > 1 {{
            first_memory ← state["memory_usage"][0]
            last_memory ← state["memory_usage"][state["memory_usage"].length() - 1]
            memory_growth ← last_memory - first_memory
        }}
        
        // Return results
        return {{
            "iterations": {iterations},
            "total_time": total_time,
            "checkpoints": state["checkpoints"].length(),
            "memory_growth": memory_growth
        }}
        """
        
        return code

class ParameterGenerator:
    """Generates extreme or boundary values for stress test parameters."""
    
    def __init__(self, seed: Optional[int] = None):
        """Initialize the parameter generator.
        
        Args:
            seed: Random seed for reproducible parameter generation
        """
        if seed is not None:
            random.seed(seed)
    
    def generate_int(self, min_val: int = -1000000, max_val: int = 1000000) -> int:
        """Generate a random integer within the specified range.
        
        Args:
            min_val: Minimum value (inclusive)
            max_val: Maximum value (inclusive)
            
        Returns:
            A random integer
        """
        return random.randint(min_val, max_val)
    
    def generate_boundary_int(self) -> int:
        """Generate an integer at or near a boundary value.
        
        Returns:
            An integer at or near a boundary value
        """
        boundaries = [
            0, 1, -1,
            2**31 - 1, -2**31,  # 32-bit integer limits
            2**63 - 1, -2**63,  # 64-bit integer limits
        ]
        
        # Either return a boundary value or a value near a boundary
        if random.random() < 0.5:
            return random.choice(boundaries)
        else:
            boundary = random.choice(boundaries)
            offset = random.randint(-10, 10)
            return boundary + offset
    
    def generate_string(self, max_length: int = 1000) -> str:
        """Generate a random string.
        
        Args:
            max_length: Maximum length of the string
            
        Returns:
            A random string
        """
        length = random.randint(0, max_length)
        return ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(length))
    
    def generate_long_string(self, min_length: int = 10000, max_length: int = 100000) -> str:
        """Generate a very long random string.
        
        Args:
            min_length: Minimum length of the string
            max_length: Maximum length of the string
            
        Returns:
            A long random string
        """
        length = random.randint(min_length, max_length)
        
        # Generate in chunks for efficiency
        chunk_size = 1000
        chunks = []
        
        for _ in range(length // chunk_size):
            chunk = ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(chunk_size))
            chunks.append(chunk)
        
        # Add the remainder
        remainder = length % chunk_size
        if remainder > 0:
            chunk = ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(remainder))
            chunks.append(chunk)
        
        return ''.join(chunks)
    
    def generate_nested_structure(self, max_depth: int = 10, max_breadth: int = 10) -> Dict:
        """Generate a deeply nested dictionary structure.
        
        Args:
            max_depth: Maximum nesting depth
            max_breadth: Maximum number of items at each level
            
        Returns:
            A deeply nested dictionary
        """
        def _generate_nested(current_depth: int) -> Any:
            if current_depth >= max_depth or random.random() < 0.2:
                # Leaf node - return a simple value
                if random.random() < 0.3:
                    return self.generate_int()
                elif random.random() < 0.6:
                    return self.generate_string(100)
                else:
                    return random.random() * 1000
            
            # Create a nested dictionary or list
            if random.random() < 0.5:
                # Dictionary
                result = {}
                breadth = random.randint(1, max_breadth)
                for i in range(breadth):
                    key = f"key_{i}"
                    result[key] = _generate_nested(current_depth + 1)
                return result
            else:
                # List
                result = []
                breadth = random.randint(1, max_breadth)
                for _ in range(breadth):
                    result.append(_generate_nested(current_depth + 1))
                return result
        
        return _generate_nested(0)

class WorkloadModeler:
    """Designs workloads with specific stress characteristics."""
    
    def __init__(self, seed: Optional[int] = None):
        """Initialize the workload modeler.
        
        Args:
            seed: Random seed for reproducible workload generation
        """
        if seed is not None:
            random.seed(seed)
        
        self.parameter_generator = ParameterGenerator(seed)
    
    def generate_memory_intensive_workload(self, intensity: StressIntensity) -> str:
        """Generate a memory-intensive workload.
        
        Args:
            intensity: The intensity level of the workload
            
        Returns:
            Anarchy Inference code for the workload
        """
        # Delegate to the stress test generator
        generator = StressTestGenerator()
        return generator._generate_memory_stress_test(intensity)
    
    def generate_cpu_intensive_workload(self, intensity: StressIntensity) -> str:
        """Generate a CPU-intensive workload.
        
        Args:
            intensity: The intensity level of the workload
            
        Returns:
            Anarchy Inference code for the workload
        """
        # Delegate to the stress test generator
        generator = StressTestGenerator()
        return generator._generate_computational_stress_test(intensity)
    
    def generate_mixed_workload(self, intensity: StressIntensity) -> str:
        """Generate a mixed workload with various stress characteristics.
        
        Args:
            intensity: The intensity level of the workload
            
        Returns:
            Anarchy Inference code for the workload
        """
        # Determine test parameters based on intensity
        if intensity == StressIntensity.LOW:
            memory_size = 1000
            computation_iterations = 100
            io_operations = 10
        elif intensity == StressIntensity.MEDIUM:
            memory_size = 10000
            computation_iterations = 1000
            io_operations = 100
        elif intensity == StressIntensity.HIGH:
            memory_size = 100000
            computation_iterations = 10000
            io_operations = 1000
        elif intensity == StressIntensity.EXTREME:
            memory_size = 1000000
            computation_iterations = 100000
            io_operations = 10000
        
        # Generate the workload code
        code = f"""
        // Mixed Stress Workload - {intensity.value.capitalize()} Intensity
        
        // Memory-intensive operations
        λ⟨ memory_stress ⟩
            data ← []
            for i in range({memory_size}) {{
                data.push(i)
            }}
            return data
        
        // CPU-intensive operations
        λ⟨ cpu_stress ⟩
            result ← 0
            for i in range({computation_iterations}) {{
                result ← result + (i * i) % 1000
            }}
            return result
        
        // I/O-intensive operations
        λ⟨ io_stress ⟩
            temp_file ← "/tmp/anarchy_mixed_stress_" + time().to_string() + ".txt"
            
            // Write operations
            for i in range({io_operations}) {{
                content ← "Line " + i.to_string() + ": " + random(1000).to_string()
                fs.append_file(temp_file, content + "\\n")
            }}
            
            // Read operations
            content ← fs.read_file(temp_file)
            
            // Clean up
            fs.remove(temp_file)
            
            return content.length()
        
        // Run all stress operations
        memory_result ← memory_stress()
        cpu_result ← cpu_stress()
        io_result ← io_stress()
        
        // Return results
        return {{
            "memory_size": memory_result.length(),
            "cpu_result": cpu_result,
            "io_operations": {io_operations}
        }}
        """
        
        return code

class TestSequencer:
    """Arranges tests to maximize stress on specific components."""
    
    def __init__(self):
        """Initialize the test sequencer."""
        self.generator = StressTestGenerator()
    
    def generate_test_sequence(self, focus_area: StressTestCategory, intensity: StressIntensity) -> List[str]:
        """Generate a sequence of tests focused on a specific area.
        
        Args:
            focus_area: The category to focus on
            intensity: The intensity level of the tests
            
        Returns:
            A list of Anarchy Inference code snippets for the test sequence
        """
        sequence = []
        
        # Generate a sequence of tests with increasing intensity
        intensities = [
            StressIntensity.LOW,
            StressIntensity.MEDIUM,
            StressIntensity.HIGH
        ]
        
        # If the requested intensity is EXTREME, include it as well
        if intensity == StressIntensity.EXTREME:
            intensities.append(StressIntensity.EXTREME)
        elif intensity == StressIntensity.HIGH:
            # Only go up to HIGH
            intensities = intensities[:3]
        elif intensity == StressIntensity.MEDIUM:
            # Only go up to MEDIUM
            intensities = intensities[:2]
        elif intensity == StressIntensity.LOW:
            # Only use LOW
            intensities = intensities[:1]
        
        # Generate tests for each intensity level
        for level in intensities:
            test = self.generator.generate_test(focus_area, level)
            sequence.append(test)
        
        return sequence
    
    def generate_mixed_test_sequence(self, intensity: StressIntensity) -> List[str]:
        """Generate a sequence of tests across different categories.
        
        Args:
            intensity: The intensity level of the tests
            
        Returns:
            A list of Anarchy Inference code snippets for the test sequence
        """
        sequence = []
        
        # Generate tests for each category
        for category in StressTestCategory:
            test = self.generator.generate_test(category, intensity)
            sequence.append(test)
        
        return sequence
