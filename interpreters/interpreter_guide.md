# Anarchy Inference Interpreter - User Guide

## Introduction

The Anarchy Inference Interpreter is a prototype implementation that allows you to run programs written in the Anarchy Inference language. This interpreter demonstrates the practical application of the language's token-efficient syntax and provides a foundation for further development.

## Installation

The interpreter is written in Python and requires Python 3.6 or higher. It also depends on the `requests` library for HTTP functionality.

1. Ensure you have Python 3.6+ installed
2. Install the required dependencies:
   ```
   pip install requests
   ```
3. Download the interpreter files:
   - `anarchy_simple_fixed.py` - The main interpreter file

## Running Anarchy Inference Programs

To run an Anarchy Inference program, use the following command:

```
python anarchy_simple_fixed.py path/to/your/program.ai
```

For example:

```
python anarchy_simple_fixed.py examples/hello_world.ai
```

## Language Features

The current interpreter prototype supports the following Anarchy Inference language features:

### Program Structure

All Anarchy Inference programs must be enclosed in a module declaration and contain a `main()` function as the entry point:

```
m{
  main(){
    // Your code here
    return 0;
  }
}
```

### Variables and Data Types

Variables are declared implicitly by assignment:

```
x = 42;           // Number
name = "Alice";   // String
items = [];       // Array
user = {};        // Object
```

Supported data types:
- Numbers (integers and floats)
- Strings (enclosed in single or double quotes)
- Booleans (`true` and `false`)
- Arrays
- Objects
- Null (`null`)

### Control Structures

#### Conditional Statements

```
if(condition){
  // Code to execute if condition is true
}else{
  // Code to execute if condition is false
}
```

#### Loops

```
for(i=0; i<10; i++){
  // Code to repeat
}

while(condition){
  // Code to repeat while condition is true
}
```

### Functions

Functions are defined using the following syntax:

```
function_name(param1, param2){
  // Function body
  return result;
}
```

### Built-in Functions

The interpreter provides several built-in functions:

#### Input/Output
- `print(value1, value2, ...)` - Outputs values to the console

#### File Operations
- `read(path)` - Reads the contents of a file
- `write(path, content)` - Writes content to a file
- `append(path, content)` - Appends content to a file
- `exists(path)` - Checks if a file exists
- `readdir(path)` - Lists files in a directory

#### Network Operations
- `get(url, options)` - Performs an HTTP GET request

#### JSON Handling
- `JSON.parse(text)` - Parses a JSON string into an object
- `JSON.stringify(obj, indent)` - Converts an object to a JSON string

#### Date and Time
- `Date.now()` - Returns the current date and time as a string

## Example Programs

### Hello World

```
m{
  main(){
    print("Hello, Anarchy Inference!");
    return 0;
  }
}
```

### File Operations

```
m{
  main(){
    // Write to a file
    content = "This is a test file.";
    write("test.txt", content);
    
    // Read from a file
    data = read("test.txt");
    print(data);
    
    return 0;
  }
}
```

### Web Request

```
m{
  main(){
    // Make an HTTP GET request
    response = get("https://jsonplaceholder.typicode.com/todos/1");
    
    if(response.code == 200){
      print("Response:", response.body);
      
      // Parse JSON response
      data = JSON.parse(response.body);
      print("Title:", data.title);
    }
    
    return 0;
  }
}
```

## Current Limitations

This is a prototype interpreter with the following limitations:

1. **Limited Error Handling**: Error messages may not be descriptive enough to pinpoint issues in your code.

2. **Partial Implementation**: Some language features described in the language reference may not be fully implemented.

3. **Performance**: The interpreter is designed for demonstration purposes and is not optimized for performance.

4. **String Concatenation**: String concatenation with the `+` operator may not work correctly in all cases.

5. **Comparison Operations**: Comparison operations between different types may cause errors.

6. **Nested Expressions**: Complex nested expressions might not be evaluated correctly.

7. **Scope Management**: Variable scoping is simplified and may not behave as expected in complex programs.

## Debugging Tips

If you encounter issues with your Anarchy Inference programs:

1. **Simplify Expressions**: Break complex expressions into simpler ones with intermediate variables.

2. **Check Types**: Ensure you're not comparing or operating on incompatible types.

3. **Add Print Statements**: Use `print()` to debug variable values at different points in your program.

4. **Inspect Error Messages**: Error messages will indicate the line where the error occurred.

## Future Improvements

The interpreter is under active development. Planned improvements include:

1. Better error handling and reporting
2. Full implementation of all language features
3. Performance optimizations
4. Support for more complex expressions and operations
5. Improved variable scoping
6. Interactive REPL mode for testing code snippets

## Contributing

Contributions to the Anarchy Inference Interpreter are welcome! Visit the [GitHub repository](https://github.com/APiTJLillo/Anarchy-Inference) to learn more about how you can contribute.

## License

The Anarchy Inference Interpreter is open-source software.
