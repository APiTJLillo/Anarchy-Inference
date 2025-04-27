// Simple test file for core Anarchy Inference language features

m{
  // Main function - entry point
  main() {
    // Test variable declarations
    num = 42;
    str = "Hello, Anarchy!";
    
    // Test print statements
    print("=== Core Features Test ===");
    print("Variable values:");
    print("num: " + num);
    print("str: " + str);
    
    // Test function calls
    print("\nFunction calls:");
    print("add(3, 4) = " + add(3, 4));
    
    print("\n=== Test Complete ===");
    
    // Return statement
    return "Test completed successfully";
  }
  
  // Function to add two numbers
  add(a, b) {
    return a + b;
  }
}
