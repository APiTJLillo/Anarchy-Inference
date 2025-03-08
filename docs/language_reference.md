# Anarchy Language Reference

## Basic Syntax

### Libraries
Libraries are defined using the `λ` symbol followed by a name and a block of code:
```
λ libraryName {
    // Library contents
}
```

### Functions
Functions are defined using the `ƒ` symbol followed by a name and parameters:
```
ƒ functionName(param1, param2) {
    // Function body
}
```

### Variables
Variables are declared using the `ι` symbol followed by a name and value:
```
ι variable = value;
```

All statements must end with a semicolon.

### Lambda Expressions
Lambda expressions are defined in three ways:
1. Single parameter: `λ param { ... }`
2. Multiple parameters: `λ(param1,param2) { ... }`
3. No parameters: `λ _ { ... }`

### String Literals
Strings are enclosed in double quotes:
```
ι message = "Hello, world!";
```

### Operators
Basic arithmetic operators are supported:
- `+` - Addition/string concatenation
- `-` - Subtraction
- `*` - Multiplication
- `/` - Division
- `=` - Assignment

## UI Components

The UI library provides components for building graphical interfaces using symbolic syntax:

### Window Creation
Create a window using the `□` symbol:
```
ι window = ⬢.□("Window Title", width, height);
```

### Text Elements
Add text using the `⬚` symbol:
```
ι text = ⬢.⬚("Text content");
```

### Text Updates
Update text content using the `⟳` symbol:
```
text.⟳("New content");
```

### Buttons
Add buttons using the `⚈` symbol:
```
⬢.⚈("Button Text", λ _ {
    // Button click handler
});
```

### Input Fields
Add input fields using the `⌸` symbol:
```
⬢.⌸("Input Label", λ value {
    // Input change handler
});
```

## Example UI Application

Here's a complete example of a UI application:

```
λ ui {
    ƒ start() {
        ι window = ⬢.□("Example App", 800, 600);
        ι text = ⬢.⬚("Hello World");
        
        ⬢.⚈("Click me", λ _ {
            text.⟳("Button clicked!");
        });
        
        ⬢.⌸("Enter text", λ value {
            text.⟳(value);
        });
    }
}

ui.start();
```

## Important Rules
1. All statements must end with a semicolon
2. String literals use double quotes
3. Function parameters are comma-separated
4. UI components must be created within a library

## Data Types

### Numbers
Numbers are represented as 64-bit integers:
```
ι x = 42;
ι y = -7;
```

### Strings
Strings are enclosed in double quotes:
```
ι msg = "Hello, world!";
```

### Booleans
Boolean values use special Unicode symbols:
- `⊤` represents true
- `⊥` represents false

Example:
```
ι is_valid = ⊤;
ι is_error = ⊥;
```

### Collections
Collections are created and manipulated using special operators:
- `∅` - Create empty collection
- `＋` - Add to collection
- `∑` - Sum collection

Example:
```
ι coll = ∅;
＋(coll,1);
＋(coll,2);
ι sum = ∑(coll);
```

## Control Flow

### Error Handling
Try-catch blocks use the `÷` symbol:
```
÷{
    // Try block
    ι x = 42;
    ι y = 0;
    ⟼(x/y)
}{
    // Catch block
    ⟼("Error caught!")
}
```

### Return Statements
Return values use the `⟼` symbol:
```
⟼(value)
```

### Print Statements
Print to output using the `⌽` symbol:
```
⌽("Hello, world!")
```

## Concurrency

### Channels
Channels provide async communication:
- `⟿(size)` - Create channel with buffer
- `⇢(chan,val)` - Send to channel
- `⇠(chan)` - Receive from channel

Example:
```
ι chan = ⟿(5);
⇢(chan,42);
ι val = ⇠(chan);
```

### Shared State
Shared state for concurrent access:
- `⟰(name)` - Create shared state
- `⇡(state,key,val)` - Set state value
- `⇣(state,key)` - Get state value

Example:
```
ι state = ⟰("mystate");
⇡(state,"counter",0);
ι val = ⇣(state,"counter");
```

## Networking

### HTTP Operations
- `⇓(url)` - HTTP GET
- `⇑(url,data)` - HTTP POST
- `⥮(url,handler)` - WebSocket connection

### TCP Operations
- `⊲(port,handler)` - Listen on port
- `⇉(conn,addr,port)` - Forward connection

## Type System

### Basic Types
- `ι` - Integer type
- `σ` - String type
- `ξ` - Generic type
- `⊤/⊥` - Boolean type

### Type Checking
The language performs static type checking at compile time while supporting type inference. 