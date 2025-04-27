// Test file for core Anarchy Inference language features using correct syntax

m{
  // Main function - entry point
  main() {
    // Test variable declarations
    num = 42;
    str = "Hello, Anarchy!";
    bool_true = true;
    bool_false = false;
    
    // Test arithmetic operations
    sum = 10 + 5;
    diff = 10 - 5;
    prod = 10 * 5;
    quot = 10 / 5;
    
    // Test string operations
    concat = "Hello, " + "World!";
    
    // Test print statements
    print("=== Core Features Test ===");
    print("Variable values:");
    print("num: " + num);
    print("str: " + str);
    print("bool_true: " + bool_true);
    print("bool_false: " + bool_false);
    
    print("\nArithmetic operations:");
    print("10 + 5 = " + sum);
    print("10 - 5 = " + diff);
    print("10 * 5 = " + prod);
    print("10 / 5 = " + quot);
    
    print("\nString operations:");
    print("Concatenation: " + concat);
    
    // Test function calls
    print("\nFunction calls:");
    print("add(3, 4) = " + add(3, 4));
    print("greet('Anarchy') = " + greet("Anarchy"));
    
    // Test collections
    list = [1, 2, 3, 4, 5];
    print("\nCollections:");
    print("List: " + list);
    print("List sum: " + sum_list(list));
    
    print("\n=== Test Complete ===");
    
    // Return statement
    return "Test completed successfully";
  }
  
  // Function to add two numbers
  add(a, b) {
    return a + b;
  }
  
  // Function to greet someone
  greet(name) {
    return "Hello, " + name + "!";
  }
  
  // Function to sum a list
  sum_list(list) {
    total = 0;
    for(i = 0; i < list.length; i = i + 1) {
      total = total + list[i];
    }
    return total;
  }
}
