# Anarchy Inference Language Documentation

## Introduction

Anarchy Inference is a token-minimal programming language designed specifically for LLM-generated code. This language uses symbolic tokens and compact syntax to dramatically reduce token usage while maintaining readability and functionality.

## Language Basics

### Program Structure

Anarchy Inference programs are organized into modules, each containing functions and variables:

```
λmodule_name{
    // Module code goes here
    
    ƒmain(){
        // Main function code
        ⟼(0)  // Return value
    }
}
```

### Variables

Variables are declared using type prefixes:

- `σ` - String variables
- `ι` - Integer variables
- `ξ` - Complex/object variables

Example:
```
σname = "Anarchy Inference";
ιcount = 42;
ξdata = { "key": "value" };
```

### Functions

Functions are defined using the `ƒ` symbol:

```
ƒadd(ιa, ιb){
    ⟼(a + b)  // Return value using ⟼ symbol
}
```

### Control Flow

Conditional statements use `ι` for if, `ε` for else:

```
ι(x > 10){
    // Code for x > 10
}ε ι(x > 5){
    // Code for 5 < x <= 10
}ε{
    // Code for x <= 5
}
```

Loops use `∀` for iteration:

```
// Array iteration
∀(array, φ(item, index){
    // Process each item
});

// Numeric range
∀(0, 10, φ(i){
    // Loop from 0 to 9
});
```

### Error Handling

Try-catch blocks use `÷{}{}` syntax:

```
÷{
    // Code that might throw an error
}{
    // Error handling code
    ⌽("Error: " + ⚠.message);
}
```

## Built-in Functions

Anarchy Inference provides several built-in functions with symbolic names:

- `⌽(message)` - Print to console
- `📖(path)` - Read file
- `✍(path, content)` - Write file
- `↗(url)` - HTTP GET request
- `⎋.parse(json)` - Parse JSON
- `⎋.stringify(obj)` - Convert to JSON string
- `🔤(value)` - Convert to string
- `🔢(value)` - Convert to number
- `?(path)` - Check if file exists
- `⧉(source, dest)` - Copy file
- `✂(path)` - Delete file
- `!(command)` - Execute shell command
- `⏰(ms)` - Sleep for milliseconds

## Data Structures

### Arrays

```
// Create empty array
ξarray = ∅;

// Add element to array
＋(array, "new item");

// Access element
σitem = array[0];
```

### Objects

```
// Create object
ξperson = {
    "name": "John",
    "age": 30
};

// Access property
σname = person.name;
```

## Mathematical Operations

Anarchy Inference uses standard operators for basic math:

- `+` - Addition
- `-` - Subtraction
- `*` - Multiplication
- `/` - Division
- `%` - Modulo

For more complex operations:

- `∑(array)` - Sum of array elements
- `∏(array)` - Product of array elements
- `√(number)` - Square root
- `∫(func, a, b)` - Numerical integration

## Examples

### Hello World

```
ƒmain(){
    ⌽("Hello, World!");
    ⟼(0)
}
```

### File Processing

```
ƒprocessFile(σfilePath){
    ÷{
        σcontent = 📖(filePath);
        ξlines = content.split("\n");
        
        ⌽("File has " + 🔤(lines.length) + " lines");
        
        ∀(lines, φ(line, i){
            ⌽("Line " + 🔤(i+1) + ": " + line);
        });
        
        ⟼(⊤)
    }{
        ⌽("Error processing file: " + ⚠.message);
        ⟼(⊥)
    }
}
```

### Web Request

```
ƒfetchData(σurl){
    ÷{
        ξresponse = ↗(url);
        
        ι(response.s≠200){
            ⌽("Error: " + response.s);
            ⟼(⊥);
        }
        
        ξdata = ⎋.parse(response.b);
        ⟼(data)
    }{
        ⌽("Request failed: " + ⚠.message);
        ⟼(⊥)
    }
}
```

## Advanced Features

### Asynchronous Programming

```
ƒasync fetchMultiple(ξurls){
    ξresults = ∅;
    
    ∀(urls, φ(url){
        ⟿(fetchData(url), φ(data){
            ＋(results, data);
        });
    });
    
    ⟿.all(φ(){
        ⌽("All requests completed");
        ⌽("Received " + 🔤(results.length) + " results");
    });
}
```

### UI Components

```
ƒcreateUI(){
    ξwindow = □("My Application", 800, 600);
    
    ξbutton = window.createButton("Click Me", 10, 10, 100, 30);
    button.onClick(φ(){
        ⌽("Button clicked!");
    });
    
    window.show();
}
```

## Best Practices

1. Use symbolic variables consistently to maintain readability
2. Group related functions within the same module
3. Use error handling for all I/O operations
4. Comment complex operations for clarity
5. Prefer built-in functions over custom implementations for token efficiency

## Further Resources

- [GitHub Repository](https://github.com/APiTJLillo/Anarchy-Inference)
- [Example Applications](https://github.com/APiTJLillo/Anarchy-Inference/tree/master/examples)
- [Benchmark Results](benchmark_results.md)
