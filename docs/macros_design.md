# Macro System Design for Anarchy Inference

## Overview

This document outlines the design for a macro system in Anarchy Inference. Macros provide a powerful way to extend the language by allowing code generation at compile time, reducing token usage, and enabling domain-specific abstractions.

## Goals

1. **Token Efficiency**: Reduce token usage by allowing complex operations to be expressed concisely
2. **Abstraction**: Enable higher-level abstractions without runtime overhead
3. **Code Generation**: Support compile-time code generation and transformation
4. **Safety**: Provide hygiene mechanisms to prevent variable capture and other macro-related issues
5. **Integration**: Seamlessly integrate with existing language features including modules and string dictionaries

## Macro Types

The macro system will support two primary types of macros:

### 1. Declarative Macros (Pattern-Based)

Declarative macros use pattern matching to transform code. They are defined using a pattern and a template, where the pattern is matched against the input code and the template is used to generate the output code.

Syntax:
```
ℳ name(pattern) ⟼ template
```

Example:
```
ℳ unless(condition, body) ⟼ if (!condition) { body }
```

### 2. Procedural Macros (Function-Based)

Procedural macros are more powerful and allow arbitrary code transformations. They are defined as functions that take AST nodes as input and return new AST nodes as output.

Syntax:
```
ℳƒ name(params) ⟼ {
    // Macro implementation
    ⟼ generated_ast
}
```

Example:
```
ℳƒ debug_print(expr) ⟼ {
    ⟼ {
        let temp = expr;
        ⌽ ":debug_message {expr} = {temp}";
        temp
    }
}
```

## Macro Expansion Process

The macro expansion process will occur during parsing, before type checking and execution:

1. **Lexing**: The lexer identifies macro definitions and invocations
2. **Parsing**: The parser creates AST nodes for macro definitions and invocations
3. **Expansion**: Macros are expanded recursively until no more expansions are possible
4. **Validation**: The expanded code is validated for correctness
5. **Execution**: The expanded code is executed normally

## AST Extensions

The following extensions to the AST will be required:

```rust
// In NodeType enum
MacroDefinition {
    name: String,
    pattern: Box<ASTNode>,
    template: Box<ASTNode>,
    is_procedural: bool,
},

MacroInvocation {
    name: String,
    arguments: Vec<ASTNode>,
},

MacroExpansion {
    original: Box<ASTNode>,
    expanded: Box<ASTNode>,
},
```

## Lexer Extensions

The lexer will be extended to recognize the macro definition symbol (ℳ) and procedural macro symbol (ℳƒ):

```rust
// In Token enum
MacroKeyword,         // ℳ
ProceduralMacroKeyword, // ℳƒ
```

## Parser Extensions

The parser will be extended to parse macro definitions and invocations:

```rust
impl Parser {
    // Parse a macro definition
    fn parse_macro_definition(&mut self) -> Result<ASTNode, LangError> {
        // ...
    }
    
    // Parse a macro invocation
    fn parse_macro_invocation(&mut self) -> Result<ASTNode, LangError> {
        // ...
    }
}
```

## Macro Expansion

A new `MacroExpander` struct will be responsible for expanding macros:

```rust
pub struct MacroExpander {
    // Map of macro names to their definitions
    macros: HashMap<String, ASTNode>,
}

impl MacroExpander {
    // Create a new macro expander
    pub fn new() -> Self {
        // ...
    }
    
    // Register a macro definition
    pub fn register_macro(&mut self, name: String, definition: ASTNode) {
        // ...
    }
    
    // Expand a macro invocation
    pub fn expand_macro(&self, invocation: &ASTNode) -> Result<ASTNode, LangError> {
        // ...
    }
    
    // Expand all macros in an AST
    pub fn expand_all(&self, ast: &ASTNode) -> Result<ASTNode, LangError> {
        // ...
    }
}
```

## Hygiene Mechanism

To prevent variable capture and other issues, the macro system will implement a hygiene mechanism:

1. **Symbol Renaming**: Variables defined in macros will be automatically renamed to avoid conflicts
2. **Context Tracking**: Each macro expansion will track its context to ensure proper variable resolution
3. **Explicit Capture**: Macros can explicitly capture variables from the surrounding scope

## Integration with Modules

Macros will be integrated with the module system:

1. **Macro Export**: Macros can be exported from modules
2. **Macro Import**: Macros can be imported from other modules
3. **Macro Visibility**: Macros can be public or private

Example:
```
λ macros {
    ⊢ ℳ unless(condition, body) ⟼ if (!condition) { body }
}

⟑ macros::{unless}

unless(x == 0, {
    ⌽ "x is not zero";
})
```

## Integration with String Dictionary

Macros will be integrated with the string dictionary system:

1. **String Interpolation**: Macros can generate string dictionary references
2. **Token Optimization**: Macros can optimize token usage by using string dictionary references

Example:
```
ℳ log_error(message) ⟼ {
    ⌽ ":error_prefix {message}";
}
```

## Error Handling

The macro system will provide detailed error messages for macro-related issues:

1. **Definition Errors**: Errors in macro definitions
2. **Expansion Errors**: Errors during macro expansion
3. **Hygiene Errors**: Errors related to variable capture
4. **Recursion Errors**: Errors due to infinite recursion

## Implementation Plan

1. **AST Extensions**: Add new AST nodes for macro definitions and invocations
2. **Lexer Extensions**: Add new tokens for macro-related symbols
3. **Parser Extensions**: Add parsing support for macro definitions and invocations
4. **Macro Expander**: Implement the macro expansion logic
5. **Hygiene Mechanism**: Implement the hygiene mechanism
6. **Integration**: Integrate with modules and string dictionary
7. **Error Handling**: Implement detailed error messages

## Examples

### Example 1: Simple Declarative Macro

```
ℳ repeat(count, body) ⟼ {
    ι i = 0;
    while (i < count) {
        body;
        i = i + 1;
    }
}

repeat(5, {
    ⌽ "Hello, world!";
})
```

Expands to:
```
{
    ι i = 0;
    while (i < 5) {
        ⌽ "Hello, world!";
        i = i + 1;
    }
}
```

### Example 2: Procedural Macro

```
ℳƒ derive_getters(struct_name, fields) ⟼ {
    ι result = [];
    for (ι i = 0; i < fields.length; i++) {
        ι field = fields[i];
        result.push({
            ƒ get_{field}() ⟼ this.{field}
        });
    }
    ⟼ result;
}

derive_getters(Person, [name, age, email])
```

Expands to:
```
ƒ get_name() ⟼ this.name
ƒ get_age() ⟼ this.age
ƒ get_email() ⟼ this.email
```

### Example 3: Macro with String Dictionary Integration

```
ℳ api_endpoint(method, path, handler) ⟼ {
    register_endpoint(method, path, (req, res) => {
        try {
            handler(req, res);
        } catch (err) {
            res.status(500).send(":api_error {err.message}");
        }
    });
}

api_endpoint("GET", "/users", (req, res) => {
    res.json(get_users());
})
```

## Conclusion

The proposed macro system for Anarchy Inference provides a powerful way to extend the language while maintaining token efficiency. By supporting both declarative and procedural macros, it enables a wide range of use cases from simple pattern-based transformations to complex code generation. The integration with modules and string dictionaries ensures that macros work seamlessly with other language features.
