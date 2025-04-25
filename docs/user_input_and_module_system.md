# User Input Emoji and Module System Documentation

This document provides documentation for the newly implemented features in Anarchy Inference:
1. User Input Emoji (🎤) support
2. Module System for code organization

## User Input Emoji (🎤)

### Overview

The User Input Emoji (🎤) is a special token that represents user input in Anarchy Inference code. When the interpreter encounters this token during execution, it will pause and prompt the user for input.

### Syntax

```
🎤
```

### Usage Examples

#### Basic Input

```
ι user_input = 🎤
⌽ "You entered: " + user_input
```

#### Input in Expressions

```
ι result = 5 + 🎤
⌽ "5 plus your input equals: " + result
```

#### Input in Function Calls

```
ƒ greet(name) {
    ⟼ "Hello, " + name + "!"
}

⌽ greet(🎤)
```

### Implementation Details

The User Input Emoji is implemented as a new token type in the lexer and a new node type in the AST. When the interpreter encounters this node during execution, it will:

1. Pause execution
2. Display a prompt to the user
3. Wait for user input
4. Resume execution with the user's input value

## Module System

### Overview

The Module System allows code organization and reuse by enabling developers to split their code into logical units. It supports:

1. Hierarchical organization with nested modules
2. Visibility control (public/private)
3. File-based module structure
4. Import system for accessing module items

### Syntax

#### Module Declaration

```
λ module_name {
    // Module contents
}
```

#### Public Module Declaration

```
⊢ λ module_name {
    // Module contents
}
```

#### Nested Modules

```
λ parent_module {
    λ child_module {
        // Child module contents
    }
}
```

#### File-Based Modules

```
λ⟨ module_name ⟩
```

This will look for a file named `module_name.ai` in the same directory and import its contents.

#### Visibility

By default, items in modules are private. To make an item public, use the `⊢` symbol:

```
λ my_module {
    // Private function
    ƒ private_function() {
        // ...
    }
    
    // Public function
    ⊢ ƒ public_function() {
        // ...
    }
}
```

#### Importing Items

```
// Import a specific item
⟑ module_name::item_name

// Import multiple items
⟑ module_name::{item1, item2}

// Import all public items
⟑ module_name::*
```

#### Module Path Resolution

```
// Access an item in a module
module_name::item_name

// Access an item in a nested module
parent_module::child_module::item_name
```

### Usage Examples

#### Basic Module

```
// math.ai
λ math {
    ⊢ ƒ add(a, b) {
        ⟼ a + b
    }
    
    ⊢ ƒ subtract(a, b) {
        ⟼ a - b
    }
    
    ƒ internal_helper() {
        // Private helper function
    }
}
```

#### Using Modules

```
// main.ai
⟑ math::{add, subtract}

ƒ main() {
    ⌽ add(5, 3)      // Outputs: 8
    ⌽ subtract(10, 4) // Outputs: 6
}
```

#### Nested Modules

```
// geometry.ai
λ geometry {
    ⊢ λ shapes {
        ⊢ ƒ circle_area(radius) {
            ⟼ 3.14159 * radius * radius
        }
        
        ⊢ ƒ rectangle_area(width, height) {
            ⟼ width * height
        }
    }
    
    ⊢ λ transformations {
        ⊢ ƒ scale(shape, factor) {
            // Implementation
        }
    }
}
```

#### File-Based Modules

```
// main.ai
λ⟨ math ⟩
λ⟨ geometry ⟩

ƒ main() {
    ⌽ math::add(5, 3)
    ⌽ geometry::shapes::circle_area(5)
}
```

### Implementation Details

The module system is implemented with the following components:

1. **Lexer**: Extended to recognize module-related tokens
2. **Parser**: Updated to parse module declarations, imports, and path resolution
3. **AST**: New node types for module-related constructs
4. **Interpreter**: Enhanced to handle module loading, visibility, and scope management

### Token Efficiency

The module system is designed with token efficiency in mind:

1. Short symbols for common operations (`λ`, `⊢`, `⟑`)
2. Minimal syntax for imports and exports
3. Reuse of existing symbols where possible
4. File-based modules to reduce code duplication

### Best Practices

1. **Organization**: Group related functionality into modules
2. **Visibility**: Only make items public if they need to be accessed from outside
3. **Naming**: Use clear, descriptive names for modules
4. **File Structure**: Match file names to module names
5. **Imports**: Import only what you need, avoid wildcard imports for large modules
6. **Nesting**: Use nested modules for logical grouping, but avoid deep nesting
