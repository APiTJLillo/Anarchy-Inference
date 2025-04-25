# Module System Design for Anarchy Inference

This document outlines the design for the module system to be implemented in Anarchy Inference.

## Overview

The module system will allow code organization and reuse by enabling developers to split their code into logical units. The design is inspired by Rust's module system but adapted for Anarchy Inference's unique syntax and token efficiency goals.

## Key Features

1. **Hierarchical Organization**: Modules can be nested to create a logical hierarchy
2. **Visibility Control**: Items can be public or private to control access
3. **File-Based Structure**: Modules can be defined in separate files
4. **Import System**: Modules can import items from other modules
5. **Token Efficiency**: Module system designed to minimize token usage

## Syntax

### Module Declaration

Modules will be declared using the existing `λ` symbol (already used for library declarations) with an extended syntax:

```
λ module_name {
    // Module contents
}
```

For nested modules:

```
λ parent_module {
    λ child_module {
        // Child module contents
    }
}
```

### File-Based Modules

For file-based modules, we'll use a simple import syntax:

```
λ⟨ module_name ⟩
```

This will look for a file named `module_name.ai` in the same directory and import its contents.

### Visibility

By default, items in modules will be private. To make an item public, we'll use the existing `⊢` symbol:

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

### Importing Items

To import items from modules, we'll use the `⟑` symbol:

```
// Import a specific item
⟑ module_name::item_name

// Import multiple items
⟑ module_name::{item1, item2}

// Import all public items
⟑ module_name::*
```

### Module Path Resolution

Module paths will be resolved using the `::` operator:

```
// Access an item in a module
module_name::item_name

// Access an item in a nested module
parent_module::child_module::item_name
```

## Implementation Details

### AST Changes

The AST will need new node types:

1. `ModuleDeclaration`: For module declarations
2. `ModuleImport`: For importing modules from files
3. `ImportDeclaration`: For importing items from modules
4. `ModulePath`: For representing module paths

### Parser Changes

The parser will need to be extended to handle:

1. Module declarations with the `λ` symbol
2. File imports with the `λ⟨ module_name ⟩` syntax
3. Item imports with the `⟑` symbol
4. Module path resolution with the `::` operator

### Interpreter Changes

The interpreter will need:

1. A module registry to store and retrieve modules
2. Scope management for module items
3. File loading for file-based modules
4. Visibility checking for module items

### File Structure

The file structure will follow this convention:

```
project/
├── main.ai
├── module1.ai
├── module2.ai
└── nested/
    ├── module3.ai
    └── module4.ai
```

## Module Resolution Algorithm

1. When a module is imported, first check if it's already loaded
2. If not, look for a file with the module name in the current directory
3. If not found, check parent directories up to the project root
4. If still not found, check standard library locations
5. If found, parse and load the module
6. Register the module in the module registry

## Examples

### Basic Module

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

### Using Modules

```
// main.ai
⟑ math::{add, subtract}

ƒ main() {
    ⌽ add(5, 3)      // Outputs: 8
    ⌽ subtract(10, 4) // Outputs: 6
}
```

### Nested Modules

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

### File-Based Modules

```
// main.ai
λ⟨ math ⟩
λ⟨ geometry ⟩

ƒ main() {
    ⌽ math::add(5, 3)
    ⌽ geometry::shapes::circle_area(5)
}
```

## Token Efficiency Considerations

The module system is designed with token efficiency in mind:

1. Short symbols for common operations (`λ`, `⊢`, `⟑`)
2. Minimal syntax for imports and exports
3. Reuse of existing symbols where possible
4. File-based modules to reduce code duplication

## Compatibility

The module system is designed to be compatible with existing Anarchy Inference code. Existing library declarations with `λ` will continue to work as before, with the module system extending this functionality.

## Future Extensions

1. **Conditional Compilation**: Module-level feature flags
2. **Versioning**: Module version requirements
3. **External Dependencies**: Importing modules from external sources
4. **Module Aliases**: Renaming modules during import
5. **Partial Exports**: Selectively re-exporting items from imported modules
