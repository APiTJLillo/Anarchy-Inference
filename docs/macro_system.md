# Macro System Documentation

## Overview

The Anarchy Inference macro system provides powerful compile-time metaprogramming capabilities that enable code generation, abstraction, and token optimization. Macros allow you to define reusable code patterns that are expanded during compilation, reducing token usage and enabling domain-specific abstractions.

## Types of Macros

Anarchy Inference supports two types of macros:

### 1. Declarative Macros

Declarative macros use pattern matching to transform code. They are defined using a pattern and a template, where the pattern is matched against the input code and the template is used to generate the output code.

**Syntax:**
```
ℳ name(pattern_variables) ⟼ template_expression
```

**Example:**
```
ℳ unless(condition, body) ⟼ if (!condition) { body }

// Usage
unless(x == 0, {
    ⌽ "x is not zero";
})

// Expands to
if (!(x == 0)) {
    ⌽ "x is not zero";
}
```

### 2. Procedural Macros

Procedural macros are more powerful and allow arbitrary code transformations. They are defined as functions that take AST nodes as input and return new AST nodes as output.

**Syntax:**
```
ℳƒ name(pattern_variables) ⟼ {
    // Macro implementation
    ⟼ generated_expression
}
```

**Example:**
```
ℳƒ debug_print(expr) ⟼ {
    ⟼ {
        let temp = expr;
        ⌽ ":debug_message {expr} = {temp}";
        temp
    }
}

// Usage
debug_print(calculate_value())

// Expands to
{
    let temp = calculate_value();
    ⌽ ":debug_message calculate_value() = {temp}";
    temp
}
```

## Macro Expansion Process

Macros are expanded during the parsing phase, before execution:

1. The parser identifies macro definitions and registers them
2. When a macro invocation is encountered, the parser checks if a macro with that name exists
3. If found, the macro is expanded by matching the pattern against the arguments
4. The expansion process is recursive, allowing macros to expand to code that contains other macro invocations
5. The expanded code replaces the original macro invocation in the AST

## Hygiene

The macro system implements hygiene mechanisms to prevent variable capture and other issues:

1. Variables defined in macros are automatically renamed to avoid conflicts with variables in the surrounding scope
2. Variables from the surrounding scope can be explicitly captured by the macro
3. Each macro expansion maintains its own scope for variables

**Example:**
```
ℳ with_temp_var(expr) ⟼ {
    let temp = 42;
    expr
}

let temp = 10;
with_temp_var(⌽ temp)  // Prints 10, not 42, due to hygiene
```

## Integration with Language Features

### Module System

Macros can be defined inside modules and exported/imported like other items:

```
λ macros {
    ⊢ ℳ unless(condition, body) ⟼ if (!condition) { body }
}

⟑ macros::{unless}

unless(x == 0, {
    ⌽ "x is not zero";
})
```

### String Dictionary

Macros can generate string dictionary references for token optimization:

```
ℳ log_error(message) ⟼ ⌽ ":error_prefix {message}"

log_error("File not found")  // Expands to ⌽ ":error_prefix File not found"
```

### Conditional Compilation

Macros can be combined with conditional compilation for platform-specific code:

```
ℳ platform_specific(web_code, desktop_code) ⟼ {
    #[if(feature="web")]
    {
        web_code
    }
    #[if(!feature="web")]
    {
        desktop_code
    }
}

platform_specific(
    ⌽ "Running on web",
    ⌽ "Running on desktop"
)
```

## Best Practices

### 1. Keep Macros Simple

Macros should be focused on a single task and should be as simple as possible. Complex macros are harder to debug and maintain.

### 2. Use Descriptive Names

Macro names should clearly indicate what the macro does. This makes code more readable and helps other developers understand your code.

### 3. Document Macros

Always include documentation for your macros, explaining what they do, what arguments they expect, and what code they generate.

### 4. Be Careful with Side Effects

Macros that have side effects can lead to unexpected behavior. Try to make macros pure functions that only transform code.

### 5. Test Macro Expansions

Always test your macros to ensure they expand to the expected code. This can be done by printing the expanded code during development.

## Common Patterns

### 1. Control Flow Macros

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

### 2. Code Generation Macros

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

### 3. Error Handling Macros

```
ℳ try_or_default(expr, default) ⟼ {
    try {
        expr
    } catch (err) {
        default
    }
}

try_or_default(parse_json(input), {})
```

### 4. Logging Macros

```
ℳ log_with_location(level, message) ⟼ {
    ⌽ ":{level}_prefix [{__FILE__}:{__LINE__}] {message}"
}

log_with_location("error", "Something went wrong")
```

## Limitations and Considerations

1. **Recursion Limit**: There is a limit to how deeply macros can be recursively expanded (default is 100 levels)
2. **Compile-Time Only**: Macros are expanded at compile time and cannot depend on runtime values
3. **Debugging**: Debugging macro-generated code can be challenging; use the debug tools to inspect macro expansions
4. **Performance**: Complex macros can increase compilation time, but they don't affect runtime performance

## Advanced Features

### 1. Macro Hygiene Control

You can explicitly control hygiene by marking variables for capture:

```
ℳƒ capture_var(var_name) ⟼ {
    // Mark var_name for capture from the surrounding scope
    capture(var_name);
    ⟼ var_name
}
```

### 2. AST Manipulation

Procedural macros can directly manipulate the AST:

```
ℳƒ optimize_math(expr) ⟼ {
    // Analyze and optimize mathematical expressions
    if (is_constant_expression(expr)) {
        ⟼ evaluate_constant(expr)
    } else {
        ⟼ expr
    }
}
```

### 3. Custom Syntax Extensions

Macros can be used to implement custom syntax extensions:

```
ℳ sql(query) ⟼ {
    parse_sql_query(query)
}

let results = sql("SELECT * FROM users WHERE age > 18")
```

## Conclusion

The Anarchy Inference macro system provides powerful metaprogramming capabilities that enable code generation, abstraction, and token optimization. By using macros effectively, you can write more concise, maintainable, and efficient code while reducing token usage.
