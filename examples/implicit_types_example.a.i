// implicit_types_example.a.i - Example of using implicit type inference in Anarchy-Inference

// Define some strings in our dictionary
📝("start", "Starting implicit type inference example...");
📝("number_msg", "Number value: {}");
📝("string_msg", "String value: {}");
📝("bool_msg", "Boolean value: {}");
📝("sum_msg", "Sum of x and y: {}");
📝("done", "Example completed successfully!");

ƒmain(){
    // Print welcome message using string dictionary
    ⌽(:start);
    
    // Use implicit type inference (no ι, σ, etc. prefixes)
    x = 42;          // Number (implicitly ιx)
    name = "Alice";  // String (implicitly σname)
    active = ⊤;      // Boolean (implicitly βactive)
    
    // Print values
    ⌽(:number_msg, x);
    ⌽(:string_msg, name);
    ⌽(:bool_msg, active);
    
    // Arithmetic with implicitly typed variables
    y = 58;
    sum = x + y;
    ⌽(:sum_msg, sum);
    
    // Automatic type coercion
    num_str = "100";
    total = x + num_str;  // String "100" automatically coerced to number 100
    ⌽("Total: {}", total);
    
    // Boolean coercion
    zero = 0;
    if (zero) {  // 0 coerced to false
        ⌽("This won't print");
    }
    
    one = 1;
    if (one) {  // 1 coerced to true
        ⌽("This will print");
    }
    
    // Completion message
    ⌽(:done);
}

// Call the main function
main();
