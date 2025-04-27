# Module System Improvements Documentation

This document provides comprehensive documentation for the improved module system in Anarchy Inference, including new features, syntax, usage examples, and best practices.

## Table of Contents

1. [Overview](#overview)
2. [Module Versioning](#module-versioning)
3. [Module Aliases](#module-aliases)
4. [Partial Re-exports](#partial-re-exports)
5. [Conditional Compilation](#conditional-compilation)
6. [Module Documentation](#module-documentation)
7. [Circular Dependency Resolution](#circular-dependency-resolution)
8. [Integration with Existing Features](#integration-with-existing-features)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Overview

The improved module system builds upon the existing foundation to provide more powerful features for code organization, dependency management, and conditional compilation. These improvements enhance the language's capabilities for large-scale application development while maintaining token efficiency.

## Module Versioning

Module versioning allows you to specify version requirements for modules, ensuring compatibility between different parts of a program.

### Syntax

#### Version Declaration
```
λ module_name v"1.0.0" {
    // Module contents
}
```

#### Version Import Requirements
```
λ⟨ module_name v">=1.0.0" ⟩
```

```
⟑ module_name v"^1.2.3"::{item1, item2}
```

### Version Constraint Types

- **Exact**: `v"1.0.0"` - Requires exactly version 1.0.0
- **Greater than or equal**: `v">=1.0.0"` - Requires version 1.0.0 or higher
- **Less than**: `v"<2.0.0"` - Requires any version less than 2.0.0
- **Range**: `v">=1.0.0,<2.0.0"` - Requires any version between 1.0.0 (inclusive) and 2.0.0 (exclusive)
- **Caret**: `v"^1.2.3"` - Compatible with 1.2.3 up to but not including 2.0.0
- **Tilde**: `v"~1.2.3"` - Compatible with 1.2.3 up to but not including 1.3.0

### Usage Examples

```
// Module declaration with version
λ math v"1.0.0" {
    ⊢ ƒ add(a, b) {
        ⟼ a + b
    }
}

// Module import with version constraint
λ⟨ math v"^1.0.0" ⟩

// Import specific items with version constraint
⟑ math v">=1.0.0,<2.0.0"::{add, subtract}
```

## Module Aliases

Module aliases allow renaming modules during import to avoid naming conflicts and improve code readability.

### Syntax

#### Basic Alias
```
⟑ long_module_name as short_name
```

#### Aliased Item Import
```
⟑ module_name as m::{item1, item2}
```

### Usage Examples

```
// Import with alias
⟑ very_long_module_name as short

// Use the alias
short::function()

// Import specific items with module alias
⟑ math as m::{add, subtract}

// Use the alias with items
m::add(1, 2)
```

## Partial Re-exports

Partial re-exports allow modules to selectively re-export items from imported modules, creating a clean public API while hiding implementation details.

### Syntax

#### Basic Re-export
```
λ my_module {
    ⟑ other_module::{item1, item2}
    
    // Re-export item1 from other_module
    ⊢ ⟑ other_module::item1
    
    // Re-export with a different name
    ⊢ ⟑ other_module::item2 as my_item
}
```

#### Grouped Re-exports
```
λ my_module {
    ⊢ ⟑ module1::{item1, item2}
    ⊢ ⟑ module2::{item3, item4 as alternative_name}
}
```

### Usage Examples

```
// Module with re-exports
λ api {
    ⟑ math::{add, subtract, multiply, divide}
    ⟑ string::{concat, split}
    
    // Re-export only selected items
    ⊢ ⟑ math::{add, subtract}
    ⊢ ⟑ string::concat as join
}

// Client code only sees the re-exported items
⟑ api::{add, subtract, join}
```

## Conditional Compilation

Conditional compilation allows code to be included or excluded based on compile-time conditions, such as platform, feature flags, or version.

### Syntax

#### Feature Declaration
```
λ module_name {
    #[feature="web"]
    ƒ web_specific() {
        // Web-specific implementation
    }
    
    #[feature="native"]
    ƒ native_specific() {
        // Native-specific implementation
    }
}
```

#### Conditional Module Sections
```
λ module_name {
    #[if(feature="web")]
    λ web {
        // Web-specific module
    }
    
    #[if(feature="native")]
    λ native {
        // Native-specific module
    }
}
```

#### Feature Enabling
```
// In build configuration or command line
--features="web,debug"
```

### Usage Examples

```
λ ui {
    #[if(feature="web")]
    λ web {
        ⊢ ƒ render() {
            // Web-specific rendering
        }
    }
    
    #[if(feature="native")]
    λ native {
        ⊢ ƒ render() {
            // Native-specific rendering
        }
    }
    
    // Common code used by both platforms
    ⊢ ƒ layout() {
        // Layout logic
    }
}
```

## Module Documentation

Module documentation provides a way to document modules, functions, and other items to improve code readability and generate documentation.

### Syntax

#### Module Documentation
```
/// Module for mathematical operations
/// 
/// This module provides basic arithmetic operations
/// and mathematical constants.
λ math {
    /// The value of pi
    ⊢ ι PI = 3.14159
    
    /// Add two numbers
    /// 
    /// # Examples
    /// 
    /// ```
    /// ⌽ math::add(2, 3)  // Outputs: 5
    /// ```
    ⊢ ƒ add(a, b) {
        ⟼ a + b
    }
}
```

### Documentation Format

Documentation comments support Markdown formatting and can include:

- Descriptions
- Examples
- Parameter descriptions
- Return value descriptions
- Notes and warnings
- Code blocks

### Usage Examples

```
/// Network module for HTTP operations
///
/// This module provides functions for making HTTP requests
/// and handling responses.
λ network {
    /// Make an HTTP GET request
    ///
    /// # Parameters
    ///
    /// * `url` - The URL to request
    ///
    /// # Returns
    ///
    /// The response body as a string
    ///
    /// # Examples
    ///
    /// ```
    /// ι response = network::get("https://example.com")
    /// ⌽ response
    /// ```
    ⊢ ƒ get(url) {
        // Implementation
    }
}
```

## Circular Dependency Resolution

The improved module system handles circular dependencies between modules to prevent infinite recursion during module loading.

### Implementation Details

1. **Two-Phase Loading**: Modules are loaded in two phases:
   - Declaration phase: Register all module exports
   - Definition phase: Resolve all imports

2. **Lazy Loading**: Module contents are loaded only when needed

3. **Forward Declarations**: Types and functions can be forward-declared

### Usage Examples

```
// module_a.ai
λ module_a {
    ⟑ module_b::item_b
    
    ⊢ ƒ item_a() {
        ⟼ item_b() + 1
    }
}

// module_b.ai
λ module_b {
    ⟑ module_a::item_a
    
    ⊢ ƒ item_b() {
        ⟼ item_a() - 1
    }
}
```

## Integration with Existing Features

### Garbage Collection

The module system integrates with the garbage collection system to ensure proper memory management:

- Module loading and unloading properly handles garbage collection
- Module-level references are tracked to prevent memory leaks
- Unused modules can be collected when no longer referenced

### String Dictionary

The module system integrates with the string dictionary system:

- Modules can have their own string dictionaries
- String usage is optimized across module boundaries
- String keys are shared between modules when possible

### Agent Memory Management

The module system integrates with agent memory management:

- Frequently used modules are prioritized in memory
- Module caching improves performance
- Memory usage is optimized for agent operations

## Best Practices

### Module Organization

1. **Single Responsibility**: Each module should have a single responsibility
2. **Logical Grouping**: Group related functionality into modules
3. **Hierarchical Structure**: Use nested modules for logical grouping
4. **File Structure**: Match file names to module names
5. **Visibility Control**: Only make items public if they need to be accessed from outside

### Versioning

1. **Semantic Versioning**: Follow semantic versioning (MAJOR.MINOR.PATCH)
2. **Version Constraints**: Use appropriate version constraints for dependencies
3. **Compatibility**: Maintain backward compatibility within the same major version
4. **Documentation**: Document breaking changes in new versions

### Imports and Exports

1. **Minimal Imports**: Import only what you need
2. **Explicit Imports**: Prefer explicit imports over wildcard imports
3. **Re-exports**: Use re-exports to create a clean public API
4. **Aliases**: Use aliases to avoid naming conflicts and improve readability

### Conditional Compilation

1. **Feature Flags**: Use feature flags for platform-specific code
2. **Default Features**: Provide sensible default features
3. **Documentation**: Document available features and their effects
4. **Testing**: Test all feature combinations

## Examples

### Basic Module Structure

```
// math.ai
λ math v"1.0.0" {
    ⊢ ι PI = 3.14159
    
    ⊢ ƒ add(a, b) {
        ⟼ a + b
    }
    
    ⊢ ƒ subtract(a, b) {
        ⟼ a - b
    }
    
    λ advanced {
        ⊢ ƒ multiply(a, b) {
            ⟼ a * b
        }
        
        ⊢ ƒ divide(a, b) {
            ⟼ a / b
        }
    }
}
```

### Module with Imports and Re-exports

```
// api.ai
λ api v"1.0.0" {
    λ⟨ math v"^1.0.0" ⟩
    λ⟨ string v"^1.0.0" ⟩
    
    ⟑ math::{add, subtract}
    ⟑ math::advanced::{multiply, divide}
    ⟑ string as str::{concat, split}
    
    // Re-export selected items
    ⊢ ⟑ math::{add, subtract}
    ⊢ ⟑ math::advanced::multiply as mul
    ⊢ ⟑ str::concat as join
    
    // Add new functionality
    ⊢ ƒ calculate(a, b, op) {
        ι result = 0
        
        if (op == "add") {
            result = add(a, b)
        } else if (op == "subtract") {
            result = subtract(a, b)
        } else if (op == "multiply") {
            result = multiply(a, b)
        } else if (op == "divide") {
            result = divide(a, b)
        }
        
        ⟼ result
    }
}
```

### Conditional Compilation Example

```
// ui.ai
λ ui v"1.0.0" {
    #[if(feature="web")]
    λ web {
        ⊢ ƒ render(element) {
            // Web-specific rendering
        }
        
        ⊢ ƒ handle_event(event) {
            // Web-specific event handling
        }
    }
    
    #[if(feature="native")]
    λ native {
        ⊢ ƒ render(element) {
            // Native-specific rendering
        }
        
        ⊢ ƒ handle_event(event) {
            // Native-specific event handling
        }
    }
    
    // Common code used by both platforms
    ⊢ ƒ create_element(type, props) {
        // Element creation logic
    }
    
    ⊢ ƒ layout(elements) {
        // Layout logic
        
        #[if(feature="web")]
        {
            // Web-specific layout adjustments
        }
        
        #[if(feature="native")]
        {
            // Native-specific layout adjustments
        }
    }
}
```

### Complete Application Example

```
// main.ai
/// Main application entry point
λ app v"1.0.0" {
    // Import modules with version constraints
    λ⟨ math v"^1.0.0" ⟩
    λ⟨ ui v"^1.0.0" ⟩
    λ⟨ network v"^1.0.0" ⟩
    
    // Import specific items with aliases
    ⟑ math as m::{add, subtract}
    ⟑ ui::{create_element, layout}
    ⟑ network as net::{get, post}
    
    // Conditional modules
    #[if(feature="debug")]
    λ debug {
        ⊢ ƒ log(message) {
            ⌽ "DEBUG: " + message
        }
        
        ⊢ ƒ assert(condition, message) {
            if (!condition) {
                ⌽ "ASSERTION FAILED: " + message
            }
        }
    }
    
    // Main function
    ⊢ ƒ main() {
        // Create UI elements
        ι elements = []
        elements.push(create_element("button", { text: "Add" }))
        elements.push(create_element("button", { text: "Subtract" }))
        
        // Layout elements
        layout(elements)
        
        // Make network request
        ι data = net::get("https://api.example.com/data")
        
        // Process data
        ι result = m::add(data.value1, data.value2)
        
        #[if(feature="debug")]
        {
            debug::log("Result: " + result)
        }
        
        ⟼ result
    }
}
```

This documentation provides a comprehensive guide to the improved module system in Anarchy Inference. By leveraging these features, developers can create more maintainable, modular, and efficient code while preserving the language's token efficiency goals.
