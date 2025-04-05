// string_dict_example.a.i - Example of using string dictionary in Anarchy-Inference

// First, let's define some strings in our dictionary
// Initialize the default dictionary
let default_dict = "default";
🔄(default_dict);

// Define strings in the dictionary
📝("welcome", "Welcome to Anarchy-Inference!");
📝("greeting", "Hello, {}!");
📝("farewell", "Goodbye, {}. See you soon!");
📝("count", "The count is: {}");
📝("error", "Error: {}");
📝("success", "Operation completed successfully: {}");

// Now let's use these strings with the print function
ƒmain(){
    // Print using string dictionary references
    ⌽(:welcome);
    
    // Print with formatting
    ιname="World";
    ⌽(:greeting, name);
    
    // Use in a loop
    ιi=0;
    ∞{
        ⌽(:count, i);
        i=i+1;
        
        // Exit after 5 iterations
        ÷{
            ⟼(i≥5)
        }{
            ⟼(⊥)
        }
    }
    
    // Use with error handling
    ÷{
        // Simulate an error
        ιx=42;
        ιy=0;
        ιresult=x/y;
        ⌽(:success, result);
    }{
        ⌽(:error, "Division by zero");
    }
    
    // Say goodbye
    ⌽(:farewell, name);
}

// Call the main function
main();
