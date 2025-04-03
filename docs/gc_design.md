# Garbage Collection Design Document for Anarchy-Inference

## 1. Introduction

This document outlines a comprehensive design for implementing garbage collection in the Anarchy-Inference language. The goal is to provide automatic memory management for complex data structures while avoiding circular dependencies in the codebase.

## 2. Current Challenges

The current implementation faces several challenges:

1. **Circular Dependencies**: The `interpreter.rs`, `value.rs`, and `gc.rs` modules all need to reference each other, creating circular dependencies.
2. **Type Integration**: The garbage collector needs to work with the `Value` type, but `Value` also needs to reference garbage-collected values.
3. **Module Structure**: The current module structure doesn't cleanly separate concerns, making it difficult to add cross-cutting features like garbage collection.

## 3. Proposed Architecture

### 3.1 Module Restructuring

To resolve circular dependencies, we propose the following module structure:

```
src/
├── core/
│   ├── mod.rs           # Exports core types
│   ├── value.rs         # Core value type definitions
│   └── gc_types.rs      # GC-related type definitions
├── gc/
│   ├── mod.rs           # Exports GC functionality
│   ├── collector.rs     # GarbageCollector implementation
│   └── managed.rs       # GC-managed value wrapper
├── interpreter/
│   ├── mod.rs           # Exports interpreter functionality
│   ├── environment.rs   # Environment implementation
│   └── evaluator.rs     # AST evaluation logic
└── lib.rs               # Main library exports
```

### 3.2 Core Types

The `core` module will contain shared type definitions used by both the garbage collector and interpreter:

```rust
// core/value.rs
pub enum ValueType {
    Null,
    Number(i64),
    String(String),
    Boolean(bool),
    Object,
    Array,
    Function,
    // Other primitive types...
}

// Forward declaration for GcValue
pub struct GcValue(pub(crate) usize);

// The main Value type
pub enum Value {
    Primitive(ValueType),
    GcManaged(GcValue),
}
```

### 3.3 Garbage Collector Design

The garbage collector will be implemented using a reference counting approach with cycle detection:

```rust
// gc/collector.rs
pub struct GarbageCollector {
    objects: Mutex<HashMap<usize, GcObject>>,
    potential_cycles: Mutex<HashSet<usize>>,
    stats: Mutex<GcStats>,
}

struct GcObject {
    id: usize,
    value: ValueImpl,
    references: HashSet<usize>,
    ref_count: usize,
    marked: bool,
}

// Concrete implementations of complex values
enum ValueImpl {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<ASTNode>,
        closure: Arc<Environment>,
    },
    // Other complex types...
}
```

### 3.4 GC-Managed Values

GC-managed values will be wrapped in a reference-counted struct:

```rust
// gc/managed.rs
pub struct GcValue {
    ptr: Arc<Mutex<GcValueInner>>,
    gc: Arc<GarbageCollector>,
}

struct GcValueInner {
    id: usize,
    value_type: ValueImpl,
}
```

### 3.5 Interpreter Integration

The interpreter will use the garbage collector through a trait:

```rust
// gc/mod.rs
pub trait GarbageCollected {
    fn init_garbage_collector(&mut self);
    fn collect_garbage(&mut self);
    fn allocate_value(&mut self, value: ValueImpl) -> GcValue;
    fn get_gc_stats(&self) -> GcStats;
}

// interpreter/mod.rs
pub struct Interpreter {
    // Other fields...
    gc: Arc<GarbageCollector>,
    allocation_count: usize,
    gc_threshold: usize,
}

impl GarbageCollected for Interpreter {
    // Implementation...
}
```

## 4. Implementation Strategy

### 4.1 Phase 1: Core Types

1. Create the `core` module with basic type definitions
2. Define the `Value` enum with primitive types and a placeholder for GC-managed values
3. Define interfaces for garbage collection

### 4.2 Phase 2: Garbage Collector

1. Implement the `GarbageCollector` struct with reference counting
2. Implement the `GcValue` wrapper for managed values
3. Add cycle detection and collection

### 4.3 Phase 3: Interpreter Integration

1. Modify the interpreter to use the garbage collector
2. Update value handling to use GC for complex types
3. Implement automatic collection triggers

### 4.4 Phase 4: Testing and Optimization

1. Create comprehensive tests for memory management
2. Benchmark and optimize collection strategies
3. Add memory usage statistics and monitoring

## 5. Memory Management Policies

### 5.1 Allocation

- Simple values (numbers, booleans, etc.) are stored directly
- Complex values (objects, arrays, functions) are managed by the GC
- Values are allocated on first creation and when copied

### 5.2 Collection Triggers

- Automatic collection after N allocations (configurable threshold)
- Manual collection when requested by the program
- Collection when memory usage exceeds a threshold

### 5.3 Collection Strategy

1. Mark all objects with zero reference count for collection
2. Detect and collect reference cycles using mark-and-sweep
3. Update statistics after collection

## 6. Performance Considerations

### 6.1 Optimization Opportunities

- Generational collection for long-lived objects
- Incremental collection to reduce pause times
- Thread-local allocation for better concurrency

### 6.2 Memory Overhead

- Each GC-managed object requires additional metadata
- Reference counting adds overhead for updates
- Cycle detection requires additional traversal

## 7. Future Enhancements

- Weak references for caching and event handling
- Finalization hooks for resource cleanup
- Custom memory policies for different object types

## 8. Conclusion

This design provides a comprehensive approach to implementing garbage collection in the Anarchy-Inference language while avoiding circular dependencies. By restructuring the modules and carefully designing the type system, we can achieve automatic memory management without sacrificing performance or code maintainability.
