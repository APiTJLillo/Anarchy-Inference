#!/usr/bin/env python3
"""
Test script for Anarchy Inference language features.
This script runs a series of tests on the Anarchy Inference language
to verify functionality and report results.
"""

import os
import sys
import subprocess
import time
from datetime import datetime

def run_test(interpreter_path, test_file_path):
    """Run a test file with the specified interpreter and return the results."""
    try:
        result = subprocess.run(
            ["python3", interpreter_path, test_file_path],
            capture_output=True,
            text=True,
            check=False
        )
        return {
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode,
            "success": result.returncode == 0
        }
    except Exception as e:
        return {
            "stdout": "",
            "stderr": str(e),
            "returncode": -1,
            "success": False
        }

def create_test_file(file_path, content):
    """Create a test file with the specified content."""
    with open(file_path, "w") as f:
        f.write(content)
    return file_path

def main():
    """Main function to run tests and report results."""
    print("Anarchy Inference Language Test Suite")
    print("=====================================")
    print(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("Running tests...\n")
    
    # Define test files
    test_files = {
        "basic": {
            "name": "Basic Language Features",
            "content": """
// Basic language features test
m{
  main() {
    print("Hello, Anarchy Inference!");
    num = 42;
    str = "Test string";
    print("Number: " + num);
    print("String: " + str);
    return 0;
  }
}
"""
        },
        "arithmetic": {
            "name": "Arithmetic Operations",
            "content": """
// Arithmetic operations test
m{
  main() {
    a = 10;
    b = 5;
    print("Addition: " + (a + b));
    print("Subtraction: " + (a - b));
    print("Multiplication: " + (a * b));
    print("Division: " + (a / b));
    return 0;
  }
}
"""
        },
        "functions": {
            "name": "Function Declarations and Calls",
            "content": """
// Function test
m{
  main() {
    print("Calling add function: " + add(3, 4));
    print("Calling greet function: " + greet("Anarchy"));
    return 0;
  }
  
  add(a, b) {
    return a + b;
  }
  
  greet(name) {
    return "Hello, " + name + "!";
  }
}
"""
        },
        "conditionals": {
            "name": "Conditional Statements",
            "content": """
// Conditional statements test
m{
  main() {
    a = 10;
    b = 5;
    
    if(a > b) {
      print("a is greater than b");
    }
    
    if(a < b) {
      print("This should not print");
    }
    
    return 0;
  }
}
"""
        },
        "loops": {
            "name": "Loop Statements",
            "content": """
// Loop statements test
m{
  main() {
    sum = 0;
    for(i = 1; i <= 5; i = i + 1) {
      sum = sum + i;
    }
    print("Sum of numbers 1 to 5: " + sum);
    return 0;
  }
}
"""
        },
        "collections": {
            "name": "Collections",
            "content": """
// Collections test
m{
  main() {
    list = [1, 2, 3, 4, 5];
    print("List: " + list);
    print("First element: " + list[0]);
    
    obj = {"name": "Anarchy", "type": "Language"};
    print("Object: " + obj);
    print("Name property: " + obj["name"]);
    
    return 0;
  }
}
"""
        },
        "error_handling": {
            "name": "Error Handling",
            "content": """
// Error handling test
m{
  main() {
    try {
      result = 10 / 0;
      print("This should not print");
    } catch(e) {
      print("Caught division by zero error");
    }
    return 0;
  }
}
"""
        },
        "gc": {
            "name": "Garbage Collection",
            "content": """
// Garbage collection test
m{
  main() {
    // Create objects that should be garbage collected
    for(i = 0; i < 100; i = i + 1) {
      temp_obj = {"id": i, "data": "This is temporary data"};
    }
    print("Created 100 temporary objects that should be garbage collected");
    print("Memory should be reclaimed after objects go out of scope");
    return 0;
  }
}
"""
        },
        "modules": {
            "name": "Module System",
            "content": """
// Module system test
m{
  main() {
    print("Testing module system");
    print("Module declaration works");
    return 0;
  }
}
"""
        }
    }
    
    # Create test directory
    test_dir = "anarchy_test_suite"
    os.makedirs(test_dir, exist_ok=True)
    
    # Define interpreters to test
    interpreters = [
        {"name": "anarchy_simple.py", "path": "interpreters/anarchy_simple.py"},
        {"name": "anarchy_simple_fixed.py", "path": "interpreters/anarchy_simple_fixed.py"},
        {"name": "anarchy.py", "path": "interpreters/anarchy.py"}
    ]
    
    # Run tests for each interpreter
    results = {}
    
    for interpreter in interpreters:
        print(f"\nTesting with {interpreter['name']}:")
        print("-" * (12 + len(interpreter['name'])))
        
        interpreter_results = {}
        
        for test_key, test_data in test_files.items():
            test_file_path = os.path.join(test_dir, f"{test_key}_{interpreter['name'].replace('.py', '')}.a.i")
            create_test_file(test_file_path, test_data["content"])
            
            print(f"  Running {test_data['name']} test...", end="")
            sys.stdout.flush()
            
            start_time = time.time()
            result = run_test(interpreter["path"], test_file_path)
            end_time = time.time()
            
            result["duration"] = end_time - start_time
            interpreter_results[test_key] = result
            
            if result["success"]:
                print(" ✓ Passed")
            else:
                print(" ✗ Failed")
                if result["stderr"]:
                    print(f"    Error: {result['stderr'].strip()}")
        
        results[interpreter["name"]] = interpreter_results
    
    # Generate summary report
    print("\nTest Summary Report")
    print("==================")
    
    for interpreter_name, interpreter_results in results.items():
        passed = sum(1 for r in interpreter_results.values() if r["success"])
        total = len(interpreter_results)
        print(f"\n{interpreter_name}: {passed}/{total} tests passed")
        
        for test_key, result in interpreter_results.items():
            status = "✓ Passed" if result["success"] else "✗ Failed"
            test_name = test_files[test_key]["name"]
            print(f"  {status} - {test_name}")
    
    # Generate detailed report file
    report_path = os.path.join(test_dir, "test_report.txt")
    with open(report_path, "w") as f:
        f.write("Anarchy Inference Language Test Report\n")
        f.write("====================================\n")
        f.write(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        
        for interpreter_name, interpreter_results in results.items():
            passed = sum(1 for r in interpreter_results.values() if r["success"])
            total = len(interpreter_results)
            f.write(f"\n{interpreter_name}: {passed}/{total} tests passed\n")
            f.write("-" * (len(interpreter_name) + 20) + "\n")
            
            for test_key, result in interpreter_results.items():
                status = "PASSED" if result["success"] else "FAILED"
                test_name = test_files[test_key]["name"]
                f.write(f"\n{test_name} - {status}\n")
                f.write(f"Duration: {result['duration']:.4f} seconds\n")
                
                if result["stdout"]:
                    f.write("\nOutput:\n")
                    f.write(result["stdout"])
                
                if result["stderr"]:
                    f.write("\nErrors:\n")
                    f.write(result["stderr"])
                
                f.write("\n" + "-" * 40 + "\n")
    
    print(f"\nDetailed test report saved to {report_path}")
    print("\nTest suite completed.")

if __name__ == "__main__":
    main()
