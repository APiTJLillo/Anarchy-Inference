# Detailed Implementation Plan for Garbage Collection in Anarchy-Inference

## 1. Introduction

This document outlines a comprehensive implementation plan for integrating garbage collection into the Anarchy-Inference language. Based on our previous attempts and the challenges encountered, this plan takes a more systematic approach to address the circular dependency issues and ensure proper integration with the existing codebase.

## 2. Analysis of Current Challenges

### 2.1 Circular Dependencies

The main challenge we've encountered is circular dependencies between modules:
- The `value.rs` module needs to reference the garbage collector to store GC-managed values
- The garbage collector needs to reference the `Value` type to manage complex values
- The interpreter needs to reference both the value system and the garbage collector

### 2.2 Integration Challenges

The existing codebase wasn't designed with garbage collection in mind:
- Values are cloned frequently without tracking references
- The ownership model doesn't account for shared references between values
- The interpreter directly manipulates values without considering memory management

### 2.3 Type System Limitations

The current type system makes it difficult to implement garbage collection:
- The `Value` enum directly contains complex data structures
- There's no separation between value types and their storage
- References between values are implicit rather than explicit

## 3. Proposed Architecture

### 3.1 Core Principles

To successfully implement garbage collection, we need to follow these principles:
1. **Separation of Concerns**: Clearly separate value types from their storage
2. **Indirection**: Use indirection for all complex values to enable tracking
3. **Explicit References**: Make references between values explicit
4. **Trait-Based Design**: Use traits to break circular dependencies

### 3.2 Module Structure

```
src/
├── memory/
│   ├── mod.rs           # Exports memory management functionality
│   ├── allocator.rs     # Memory allocation interface
│   ├── gc.rs            # Garbage collector implementation
│   └── reference.rs     # Reference counting and tracking
├── value/
│   ├── mod.rs           # Exports value types
│   ├── types.rs         # Value type definitions
│   ├── primitive.rs     # Primitive value implementations
│   └── complex.rs       # Complex value implementations
├── runtime/
│   ├── mod.rs           # Exports runtime functionality
│   ├── interpreter.rs   # Interpreter implementation
│   ├── environment.rs   # Environment implementation
│   └── evaluator.rs     # Expression evaluation
└── lib.rs               # Main library exports
```

### 3.3 Key Interfaces

#### Memory Management Interface

```rust
// memory/allocator.rs
pub trait Allocator {
    fn allocate<T: 'static>(&self, value: T) -> Ref<T>;
    fn collect(&self);
    fn stats(&self) -> AllocatorStats;
}

// memory/reference.rs
pub struct Ref<T> {
    id: usize,
    allocator: Arc<dyn Allocator>,
    _phantom: PhantomData<T>,
}
```

#### Value Type System

```rust
// value/types.rs
pub enum ValueType {
    Null,
    Boolean(bool),
    Number(i64),
    String(String),
    Object(Ref<HashMap<String, Value>>),
    Array(Ref<Vec<Value>>),
    Function(Ref<Function>),
}

pub struct Value {
    type_: ValueType,
}
```

#### Runtime Integration

```rust
// runtime/interpreter.rs
pub struct Interpreter {
    // Other fields...
    allocator: Arc<dyn Allocator>,
}

impl Interpreter {
    pub fn new() -> Self {
        let allocator = Arc::new(GarbageCollector::new());
        // Initialize other fields...
        Self { allocator, /* other fields */ }
    }
    
    pub fn allocate<T: 'static>(&self, value: T) -> Ref<T> {
        self.allocator.allocate(value)
    }
}
```

## 4. Implementation Strategy

### 4.1 Phase 1: Memory Management Foundation

1. Implement the `Allocator` trait and basic reference counting
2. Create the `Ref<T>` type for tracking references
3. Implement a simple mark-and-sweep garbage collector
4. Add memory usage statistics and monitoring

### 4.2 Phase 2: Value Type System Refactoring

1. Refactor the `Value` type to use the new memory management system
2. Separate primitive values from complex values
3. Update value operations to work with the new type system
4. Implement proper reference tracking for complex values

### 4.3 Phase 3: Interpreter Integration

1. Modify the interpreter to use the allocator for complex values
2. Update the environment to handle references properly
3. Implement automatic collection triggers
4. Add memory management hooks for language constructs

### 4.4 Phase 4: Testing and Optimization

1. Create comprehensive tests for memory management
2. Benchmark and optimize collection strategies
3. Add memory usage statistics and monitoring
4. Implement debugging tools for memory issues

## 5. Detailed Implementation Steps

### 5.1 Memory Management Foundation

#### Step 1: Define Core Interfaces

```rust
// memory/mod.rs
pub mod allocator;
pub mod gc;
pub mod reference;

pub use allocator::{Allocator, AllocatorStats};
pub use reference::Ref;
```

```rust
// memory/allocator.rs
pub trait Allocator: Send + Sync + std::fmt::Debug {
    fn allocate<T: 'static>(&self, value: T) -> Ref<T>;
    fn deallocate(&self, id: usize);
    fn collect(&self);
    fn stats(&self) -> AllocatorStats;
}

#[derive(Debug, Clone, Default)]
pub struct AllocatorStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub collections: usize,
    pub memory_usage: usize,
}
```

#### Step 2: Implement Reference Type

```rust
// memory/reference.rs
use std::marker::PhantomData;
use std::sync::Arc;
use crate::memory::allocator::Allocator;

#[derive(Debug)]
pub struct Ref<T> {
    pub(crate) id: usize,
    pub(crate) allocator: Arc<dyn Allocator>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Ref<T> {
    pub fn new(id: usize, allocator: Arc<dyn Allocator>) -> Self {
        Self {
            id,
            allocator,
            _phantom: PhantomData,
        }
    }
    
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        // Increment reference count in allocator
        Self {
            id: self.id,
            allocator: self.allocator.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for Ref<T> {
    fn drop(&mut self) {
        // Decrement reference count in allocator
        self.allocator.deallocate(self.id);
    }
}
```

#### Step 3: Implement Garbage Collector

```rust
// memory/gc.rs
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::memory::allocator::{Allocator, AllocatorStats};
use crate::memory::reference::Ref;

struct GcObject {
    id: usize,
    value: Box<dyn Any + Send + Sync>,
    references: HashSet<usize>,
    ref_count: usize,
    marked: bool,
}

#[derive(Debug)]
pub struct GarbageCollector {
    objects: Mutex<HashMap<usize, GcObject>>,
    potential_cycles: Mutex<HashSet<usize>>,
    stats: Mutex<AllocatorStats>,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            objects: Mutex::new(HashMap::new()),
            potential_cycles: Mutex::new(HashSet::new()),
            stats: Mutex::new(AllocatorStats::default()),
        }
    }
    
    // Implementation details...
}

impl Allocator for GarbageCollector {
    fn allocate<T: 'static>(&self, value: T) -> Ref<T> {
        // Implementation details...
    }
    
    fn deallocate(&self, id: usize) {
        // Implementation details...
    }
    
    fn collect(&self) {
        // Implementation details...
    }
    
    fn stats(&self) -> AllocatorStats {
        // Implementation details...
    }
}
```

### 5.2 Value Type System Refactoring

#### Step 1: Define Value Types

```rust
// value/types.rs
use std::collections::HashMap;
use std::sync::Arc;
use crate::memory::reference::Ref;

#[derive(Debug, Clone)]
pub enum ValueType {
    Null,
    Boolean(bool),
    Number(i64),
    String(String),
    Object(Ref<HashMap<String, Value>>),
    Array(Ref<Vec<Value>>),
    Function(Ref<Function>),
}

#[derive(Debug, Clone)]
pub struct Value {
    type_: ValueType,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Box<ASTNode>,
    pub closure: Arc<Environment>,
}
```

#### Step 2: Implement Value Operations

```rust
// value/complex.rs
use std::collections::HashMap;
use crate::memory::reference::Ref;
use crate::value::types::{Value, ValueType};

impl Value {
    pub fn new_object(obj: Ref<HashMap<String, Value>>) -> Self {
        Self {
            type_: ValueType::Object(obj),
        }
    }
    
    pub fn new_array(arr: Ref<Vec<Value>>) -> Self {
        Self {
            type_: ValueType::Array(arr),
        }
    }
    
    pub fn new_function(func: Ref<Function>) -> Self {
        Self {
            type_: ValueType::Function(func),
        }
    }
    
    // Other methods...
}
```

### 5.3 Interpreter Integration

#### Step 1: Update Interpreter

```rust
// runtime/interpreter.rs
use std::sync::Arc;
use crate::memory::allocator::Allocator;
use crate::memory::gc::GarbageCollector;
use crate::memory::reference::Ref;
use crate::value::types::{Value, ValueType, Function};

pub struct Interpreter {
    // Other fields...
    allocator: Arc<dyn Allocator>,
}

impl Interpreter {
    pub fn new() -> Self {
        let allocator = Arc::new(GarbageCollector::new());
        // Initialize other fields...
        Self {
            allocator,
            // Other fields...
        }
    }
    
    pub fn allocate<T: 'static>(&self, value: T) -> Ref<T> {
        self.allocator.allocate(value)
    }
    
    pub fn collect_garbage(&self) {
        self.allocator.collect();
    }
    
    // Other methods...
}
```

#### Step 2: Update Environment

```rust
// runtime/environment.rs
use std::collections::HashMap;
use std::sync::Arc;
use crate::value::types::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Arc<Environment>>,
}

impl Environment {
    // Implementation details...
}
```

## 6. Testing Strategy

### 6.1 Unit Tests

1. Test reference counting
2. Test garbage collection of unreferenced objects
3. Test cycle detection and collection
4. Test memory statistics

### 6.2 Integration Tests

1. Test interpreter with garbage collection
2. Test complex data structures with references
3. Test memory usage patterns
4. Test collection triggers

### 6.3 Benchmarks

1. Measure allocation performance
2. Measure collection performance
3. Compare memory usage with and without garbage collection
4. Measure impact on interpreter performance

## 7. Timeline and Milestones

1. **Week 1**: Implement memory management foundation
2. **Week 2**: Refactor value type system
3. **Week 3**: Integrate with interpreter
4. **Week 4**: Testing and optimization

## 8. Conclusion

This detailed implementation plan provides a comprehensive approach to integrating garbage collection into the Anarchy-Inference language. By following this plan, we can avoid the circular dependency issues encountered in previous attempts and ensure proper integration with the existing codebase.

The key to success is the clear separation of concerns between memory management, value types, and the interpreter, along with the use of traits and indirection to break circular dependencies. This approach will result in a more maintainable and extensible codebase that properly handles memory management for complex data structures.
