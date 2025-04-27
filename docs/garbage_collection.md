# Garbage Collection in Anarchy Inference

This document provides a comprehensive guide to the garbage collection system in Anarchy Inference, including its architecture, usage patterns, and best practices.

## Overview

Anarchy Inference uses a hybrid garbage collection approach that combines reference counting with cycle detection. This approach provides efficient memory management with minimal overhead while still handling complex reference patterns including circular references.

## Architecture

The garbage collection system consists of several key components:

### GarbageCollector

The central component responsible for tracking objects, managing reference counts, and performing collection operations. It provides:

- Object allocation and tracking
- Reference count management
- Cycle detection and collection
- Memory usage statistics
- Automatic collection based on configurable thresholds
- Generational collection for improved performance
- Incremental collection to reduce pause times

### GcValue

A wrapper for values managed by the garbage collector, providing:

- Reference to the actual value
- Connection to the garbage collector
- Reference tracking
- Automatic cleanup on drop

### GcValueImpl

Concrete implementations of complex values that need garbage collection:

- Objects (key-value maps)
- Arrays (ordered collections)
- Functions (with closures)
- Other complex types

## Memory Management Strategies

### Reference Counting

The primary memory management strategy:

- Each object has a reference count tracking how many references point to it
- When count reaches zero, object can be collected
- Efficient for most usage patterns
- Immediate collection of unreferenced objects

### Cycle Detection

Supplementary strategy to handle reference cycles:

- Mark-and-sweep algorithm to detect unreachable cycles
- Runs periodically or when memory usage exceeds threshold
- Only processes objects that might form cycles
- Uses graph traversal to identify unreachable objects

## Usage

### Basic Usage

The garbage collector is automatically initialized when creating an interpreter:

```rust
let mut interpreter = Interpreter::new();
```

Complex values (objects, arrays, functions) are automatically allocated in the garbage collector:

```rust
// In Anarchy Inference code:
ι obj = { name: "Example", value: 42 };
ι arr = [1, 2, 3, 4, 5];
ƒ example() { ⟼ "Hello, world!"; }
```

### Manual Collection

You can manually trigger garbage collection:

```rust
interpreter.collect_garbage();
```

### Configuration

You can configure the garbage collector through the interpreter:

```rust
// Get the garbage collector
let gc = interpreter.get_garbage_collector();

// Configure collection threshold (in bytes)
gc.set_collection_threshold(2 * 1024 * 1024); // 2MB

// Enable/disable automatic collection
gc.set_auto_collect(true);

// Configure generational collection
gc.set_generation_threshold(3);

// Configure incremental collection
gc.set_incremental_step_size(100);
```

### Statistics

You can get statistics about the garbage collector:

```rust
let stats = interpreter.get_gc_stats();
println!("Allocations: {}", stats.allocations);
println!("Deallocations: {}", stats.deallocations);
println!("Total memory: {} bytes", stats.total_memory);
println!("Peak memory: {} bytes", stats.peak_memory);
println!("Collections performed: {}", stats.collections_performed);
println!("Cycles detected: {}", stats.cycles_detected);
println!("Last collection time: {} ms", stats.last_collection_time_ms);
```

## Collection Policies

### Automatic Collection

By default, garbage collection is triggered automatically when memory usage exceeds a threshold (default: 1MB). This behavior can be configured or disabled.

### Generational Collection

Objects are organized into generations:

- New objects start in generation 0
- Surviving objects are promoted to higher generations
- Collection can target specific generations
- Younger generations are collected more frequently

### Incremental Collection

Collection can be performed incrementally:

- Process a limited number of objects per step
- Spread collection cost over time
- Reduce pause times
- Useful for real-time applications

## Best Practices

### Memory Management

1. **Avoid creating unnecessary references**: Each reference increases the reference count and delays collection.

2. **Break reference cycles when possible**: Although the cycle detector will handle them, it's more efficient to break cycles manually.

3. **Use local variables**: Variables that go out of scope automatically have their reference counts decremented.

4. **Consider object pooling**: For frequently allocated objects, consider using an object pool to reduce allocation overhead.

### Performance Tuning

1. **Adjust collection threshold**: Increase for applications with large memory requirements, decrease for memory-constrained environments.

2. **Configure generational collection**: Adjust generation threshold based on object lifetime patterns.

3. **Use incremental collection**: For real-time applications, use incremental collection to reduce pause times.

4. **Monitor memory usage**: Regularly check memory statistics to identify potential issues.

## Implementation Details

### Reference Counting

Each object has a reference count that tracks how many references point to it. When a reference is created, the count is incremented. When a reference is dropped, the count is decremented. When the count reaches zero, the object can be collected.

### Cycle Detection

The cycle detector uses a mark-and-sweep algorithm:

1. **Mark phase**: Start from all root objects (those with non-zero reference counts) and mark all reachable objects.
2. **Sweep phase**: Collect all unmarked objects that might form cycles.

This approach ensures that circular references don't cause memory leaks.

### Memory Tracking

The garbage collector tracks memory usage for each object and maintains statistics about total memory usage, peak memory usage, and collection performance.

### Thread Safety

The garbage collector uses mutexes to ensure thread safety, allowing it to be used in multi-threaded environments.

## Conclusion

The garbage collection system in Anarchy Inference provides efficient memory management with minimal overhead. By combining reference counting with cycle detection, it offers immediate collection of most objects while still handling complex reference patterns. The configurable collection policies allow it to be tuned for different application requirements, from memory-constrained environments to real-time applications.
