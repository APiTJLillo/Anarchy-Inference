# Simplified Garbage Collection Design for Anarchy-Inference

## Overview

This document outlines a simplified approach to garbage collection for the Anarchy-Inference language. After analyzing the codebase and attempting to implement a comprehensive garbage collection system, we've identified several challenges related to Rust's lifetime and trait systems. This simplified design focuses on practical memory management that can be integrated with the existing codebase.

## Design Goals

1. **Simplicity**: Implement a straightforward reference counting system
2. **Compatibility**: Work within the constraints of the existing codebase
3. **Reliability**: Ensure memory is properly managed without leaks
4. **Performance**: Minimize overhead while providing effective memory management

## Architecture

### 1. Reference Counting

The core of this simplified garbage collection system is reference counting. Each complex value (objects, arrays, functions) will have an associated reference count that tracks how many references point to it.

```
+----------------+      +----------------+
| Value          |      | RefCount       |
|----------------|      |----------------|
| type: ValueType|----->| count: usize   |
| data: ...      |      | value: Box<T>  |
+----------------+      +----------------+
```

### 2. Value Wrapper

Instead of modifying the core Value type extensively, we'll create a wrapper that handles reference counting:

```rust
pub struct RcValue<T> {
    inner: Rc<RefCell<T>>,
}
```

This approach allows us to leverage Rust's built-in reference counting (`Rc`) and interior mutability (`RefCell`) to manage memory.

### 3. Integration Points

The simplified garbage collection system will integrate with the existing codebase at these key points:

- **Value Creation**: When complex values are created
- **Value Assignment**: When values are assigned to variables
- **Value Passing**: When values are passed as arguments
- **Scope Exit**: When a scope is exited and variables go out of scope

## Implementation Plan

### Phase 1: Core Reference Counting

1. Create a `RcValue<T>` wrapper using Rust's `Rc` and `RefCell`
2. Implement methods for creating, cloning, and accessing values
3. Add helper functions for common operations

### Phase 2: Value System Integration

1. Update the Value enum to use RcValue for complex types
2. Modify value creation functions to use reference counting
3. Ensure proper cloning behavior to increment reference counts

### Phase 3: Interpreter Integration

1. Update the interpreter to handle reference-counted values
2. Ensure proper cleanup when variables go out of scope
3. Add debugging helpers for tracking memory usage

## Benefits of This Approach

1. **Leverages Rust's Built-in Features**: Uses Rust's `Rc` and `RefCell` instead of custom implementations
2. **Minimal Changes to Existing Code**: Focuses on wrapping values rather than restructuring the codebase
3. **Incremental Implementation**: Can be implemented and tested in stages
4. **Avoids Complex Lifetime Issues**: Works within Rust's ownership system rather than fighting against it

## Limitations

1. **No Cycle Detection**: This simplified approach doesn't handle reference cycles
2. **Limited to Single-Threaded Use**: Uses `Rc` instead of `Arc`, limiting to single-threaded contexts
3. **Some Manual Management**: May require some manual reference management in certain cases

## Future Improvements

Once this simplified system is working, we can consider these improvements:

1. Add cycle detection using weak references
2. Implement thread-safe reference counting using Arc
3. Add more sophisticated memory usage tracking
4. Optimize performance for common operations
