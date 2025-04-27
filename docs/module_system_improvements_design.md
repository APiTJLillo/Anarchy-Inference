# Module System Improvements Design

This document outlines the design for improvements to the Anarchy Inference module system, building upon the existing implementation.

## 1. Module Versioning

### Overview
Module versioning will allow developers to specify version requirements for modules, ensuring compatibility between different parts of a program.

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

### Implementation Details

1. **Version Format**: Semantic versioning (MAJOR.MINOR.PATCH)
2. **Version Constraints**:
   - Exact: `v"1.0.0"`
   - Greater than or equal: `v">=1.0.0"`
   - Less than: `v"<2.0.0"`
   - Range: `v">=1.0.0,<2.0.0"`
   - Caret: `v"^1.2.3"` (compatible with 1.2.3 up to 2.0.0)
   - Tilde: `v"~1.2.3"` (compatible with 1.2.3 up to 1.3.0)
3. **Version Resolution**: When multiple version constraints exist, find the highest version that satisfies all constraints

### AST Changes
- Add version information to `ModuleDeclaration` and `ModuleImport` nodes
- Add version constraint parsing to the lexer and parser

## 2. Conditional Compilation

### Overview
Conditional compilation will allow code to be included or excluded based on compile-time conditions, such as platform, feature flags, or version.

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
    
    #[feature="debug"]
    ƒ debug_only() {
        // Debug-only functionality
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

### Implementation Details

1. **Feature Registry**: Track enabled features during compilation
2. **Conditional Parsing**: Skip parsing of disabled sections
3. **Feature Expressions**: Support boolean expressions like `web && !debug`
4. **Default Features**: Allow specifying default features for a module

### AST Changes
- Add attribute nodes for features and conditions
- Modify the parser to conditionally include or exclude nodes based on feature flags

## 3. Module Aliases

### Overview
Module aliases will allow renaming modules during import to avoid naming conflicts and improve code readability.

### Syntax

#### Basic Alias
```
⟑ long_module_name as short_name
```

#### Aliased Item Import
```
⟑ module_name as m::{item1, item2}
```

#### Using Aliases
```
// Instead of long_module_name::function()
short_name::function()

// Instead of module_name::item1
m::item1
```

### Implementation Details

1. **Alias Registry**: Track module aliases in the interpreter
2. **Name Resolution**: Resolve aliases during module path resolution
3. **Scope Management**: Aliases are valid only in the scope where they are defined

### AST Changes
- Add alias information to `ImportDeclaration` nodes
- Modify module path resolution to handle aliases

## 4. Partial Re-exports

### Overview
Partial re-exports will allow modules to selectively re-export items from imported modules, creating a clean public API while hiding implementation details.

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

### Implementation Details

1. **Export Registry**: Track re-exported items in the module
2. **Name Resolution**: Resolve re-exported items during module path resolution
3. **Visibility Control**: Re-exported items inherit the visibility of the re-export declaration

### AST Changes
- Add re-export flag to `ImportDeclaration` nodes
- Add support for renaming in re-exports

## 5. Circular Dependency Resolution

### Overview
Improved handling of circular dependencies between modules to prevent infinite recursion during module loading.

### Implementation Details

1. **Dependency Tracking**: Track module dependencies during loading
2. **Cycle Detection**: Detect circular dependencies and handle them gracefully
3. **Lazy Loading**: Load module contents only when needed
4. **Forward Declarations**: Allow forward declarations of types and functions

### Example
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

### AST Changes
- Add dependency tracking to the module loader
- Implement a two-phase loading process: declaration phase and definition phase

## 6. Module Documentation

### Overview
Support for module-level documentation to improve code readability and generate documentation.

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

### Implementation Details

1. **Documentation Comments**: Parse and store documentation comments
2. **Documentation Generation**: Generate HTML or Markdown documentation from comments
3. **Example Code**: Support for example code in documentation
4. **Metadata**: Support for metadata tags like `@param`, `@return`, `@deprecated`

### AST Changes
- Add documentation string to relevant AST nodes
- Implement a documentation parser for structured documentation

## 7. Integration with Existing Features

### Garbage Collection
- Ensure module loading and unloading properly handles garbage collection
- Track module-level references to prevent memory leaks

### String Dictionary
- Support for module-specific string dictionaries
- Optimize string usage across module boundaries

### Agent Memory Management
- Integration with agent memory for module caching
- Prioritize frequently used modules in memory

## Implementation Plan

1. **Core Improvements**:
   - Module versioning
   - Module aliases
   - Partial re-exports

2. **Advanced Features**:
   - Conditional compilation
   - Circular dependency resolution
   - Module documentation

3. **Integration**:
   - Update lexer, parser, and interpreter
   - Update AST nodes and type system
   - Integrate with existing features

4. **Testing**:
   - Unit tests for each improvement
   - Integration tests for combined features
   - Performance benchmarks

5. **Documentation**:
   - Update language reference
   - Create examples for each new feature
   - Update existing documentation

## Token Efficiency Considerations

All improvements are designed with token efficiency in mind:

1. **Concise Syntax**: Short symbols and minimal syntax for new features
2. **Reuse**: Reuse existing symbols and patterns where possible
3. **Optimization**: Optimize module loading and resolution for performance
4. **Lazy Loading**: Load module contents only when needed to reduce memory usage

## Compatibility

These improvements are designed to be backward compatible with existing code. Existing module declarations and imports will continue to work without modification.
