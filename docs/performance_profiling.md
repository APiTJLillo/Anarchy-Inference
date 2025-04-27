# Performance Profiling in Anarchy Inference

This document provides a comprehensive guide to the performance profiling system in Anarchy Inference. The profiling system allows developers to measure execution time, memory usage, and operation counts to identify bottlenecks and optimize their code.

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Profiling Features](#profiling-features)
4. [API Reference](#api-reference)
5. [Configuration Options](#configuration-options)
6. [Report Formats](#report-formats)
7. [Integration with Interpreter](#integration-with-interpreter)
8. [Best Practices](#best-practices)
9. [Examples](#examples)

## Overview

The Anarchy Inference performance profiling system provides detailed insights into the runtime behavior of programs. It tracks execution time, memory usage, and operation counts at various levels of granularity, from individual expressions to entire modules.

Key features include:
- Minimal overhead when disabled
- Configurable profiling levels
- Comprehensive metrics collection
- Multiple report formats
- Integration with the interpreter
- Support for custom metrics

## Getting Started

### Basic Usage

To start profiling your Anarchy Inference code, follow these steps:

1. Create a profiler instance:
```rust
let mut profiler = Profiler::new();
profiler.set_enabled(true);
```

2. Start a profiling session:
```rust
profiler.start_session("my_session").unwrap();
```

3. Profile code execution:
```rust
// Using the span API directly
let span_guard = profiler.start_span("my_operation", SpanType::Function).unwrap();
// ... code to profile ...
drop(span_guard); // Automatically ends the span

// Or using the macro
profile_block!(profiler, "my_operation", SpanType::Function, {
    // ... code to profile ...
});
```

4. End the session and generate a report:
```rust
profiler.end_session().unwrap();
let report = profiler.generate_report(ReportFormat::Text).unwrap();
println!("{}", report);
```

### Integration with Interpreter

The profiling system is integrated with the Anarchy Inference interpreter, making it easy to profile code execution:

```rust
// Create an interpreter
let mut interpreter = Interpreter::new();

// Enable profiling
interpreter.enable_profiling();

// Start a profiling session
interpreter.start_profiling_session("my_session").unwrap();

// Execute code with profiling
interpreter.profile_execute_node(&ast_node).unwrap();

// End the session and generate a report
interpreter.end_profiling_session().unwrap();
let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
println!("{}", report);
```

## Profiling Features

### Time Profiling

The profiling system tracks execution time at different levels:
- Total execution time of the program
- Time spent in each function
- Time spent evaluating each expression
- Time spent in garbage collection

Example report output:
```
Top Functions by Execution Time:
1. calculate_fibonacci: 0.567s (45.9%)
2. process_data: 0.345s (28.0%)
3. parse_input: 0.123s (10.0%)
```

### Memory Profiling

Memory profiling tracks:
- Total memory usage
- Peak memory usage
- Number and size of allocations
- Number and size of deallocations
- Garbage collection statistics

Example report output:
```
Memory Usage:
- Total Allocations: 1024 (512KB)
- Peak Memory Usage: 768KB
- GC Collections: 5 (reclaimed 256KB)
```

### Operation Profiling

Operation profiling counts:
- Arithmetic operations
- String operations
- Array operations
- Object operations
- Function calls
- Variable accesses
- Property accesses
- String dictionary operations

Example report output:
```
Operation Counts:
- Arithmetic: 5432
- String: 789
- Array: 123
- Object: 456
```

## API Reference

### Profiler

The main class that manages profiling sessions and coordinates metric collection.

```rust
// Create a new profiler with default configuration
let mut profiler = Profiler::new();

// Create a new profiler with custom configuration
let config = ProfilerConfig { /* ... */ };
let mut profiler = Profiler::with_config(config);

// Enable or disable profiling
profiler.set_enabled(true);

// Start a new profiling session
profiler.start_session("my_session").unwrap();

// Start a new profiling span
let span_guard = profiler.start_span("my_span", SpanType::Function).unwrap();

// Record a metric
profiler.record_metric("my_metric", MetricValue::Count(42)).unwrap();

// End the current profiling session
let session = profiler.end_session().unwrap();

// Generate a report
let report = profiler.generate_report(ReportFormat::Text).unwrap();
```

### ProfilingSession

Represents a single profiling run.

```rust
// Get the name of the session
session.name();

// Get the duration of the session
session.duration();

// Get the top spans by duration
let top_spans = session.top_spans_by_duration(10);

// Get the time spent in each span type
let time_by_type = session.time_by_span_type();
```

### ProfilingSpan

Represents a single profiled operation.

```rust
// Get the name of the span
span.name();

// Get the type of the span
span.span_type();

// Get the duration of the span
span.duration();

// Get a metric by name
span.get_metric("execution_time");
```

### SpanGuard

Provides a RAII-style API for automatically ending spans.

```rust
// Create a span guard
let span_guard = profiler.start_span("my_span", SpanType::Function).unwrap();

// Record a metric for the current span
span_guard.record_metric("my_metric", MetricValue::Count(42)).unwrap();

// The span is automatically ended when the guard is dropped
drop(span_guard);
```

### Macros

The profiling system provides macros for easy integration:

```rust
// Profile a block of code
profile_block!(profiler, "my_block", SpanType::Block, {
    // ... code to profile ...
});

// Profile a function
profile_fn!(profiler, {
    // ... function body ...
});
```

## Configuration Options

The profiling system is highly configurable through the `ProfilerConfig` struct:

```rust
let mut config = ProfilerConfig::default();

// Enable profiling
config.enabled = true;

// Configure time profiling
config.time_profiling.enabled = true;
config.time_profiling.precision = TimePrecision::Microsecond;
config.time_profiling.min_duration = Duration::from_micros(10);

// Configure memory profiling
config.memory_profiling.enabled = true;
config.memory_profiling.track_allocations = true;
config.memory_profiling.track_peak_memory = true;

// Configure operation profiling
config.operation_profiling.enabled = true;
let mut tracked_ops = HashSet::new();
tracked_ops.insert(OperationType::Arithmetic);
tracked_ops.insert(OperationType::String);
config.operation_profiling.tracked_operations = tracked_ops;

// Configure output options
config.output.default_format = ReportFormat::Text;
config.output.include_source_locations = true;
config.output.max_call_stack_depth = 10;

// Create a profiler with this configuration
let profiler = Profiler::with_config(config);
```

## Report Formats

The profiling system supports multiple report formats:

### Text Format

A human-readable text format:

```
=== Anarchy Inference Profiling Report ===
Session: main
Duration: 1.234s

Top Functions by Execution Time:
1. calculate_fibonacci: 0.567s (45.9%)
2. process_data: 0.345s (28.0%)
3. parse_input: 0.123s (10.0%)

Memory Usage:
- Total Allocations: 1024 (512KB)
- Peak Memory Usage: 768KB
- GC Collections: 5 (reclaimed 256KB)

Operation Counts:
- Arithmetic: 5432
- String: 789
- Array: 123
- Object: 456

Call Graph:
main (1.234s)
  ├─ parse_input (0.123s)
  ├─ process_data (0.345s)
  │   └─ calculate_fibonacci (0.567s)
  └─ output_results (0.199s)
```

### JSON Format

A machine-readable JSON format:

```json
{
  "session": {
    "name": "main",
    "duration_ms": 1234,
    "start_time": "2025-04-26T22:15:00Z",
    "end_time": "2025-04-26T22:15:01.234Z"
  },
  "time_metrics": {
    "functions": [
      {"name": "calculate_fibonacci", "duration_ms": 567, "percentage": 45.9},
      {"name": "process_data", "duration_ms": 345, "percentage": 28.0},
      {"name": "parse_input", "duration_ms": 123, "percentage": 10.0}
    ]
  },
  "memory_metrics": {
    "total_allocations": 1024,
    "total_allocation_bytes": 524288,
    "peak_memory_bytes": 786432,
    "gc_collections": 5,
    "gc_reclaimed_bytes": 262144
  },
  "operation_metrics": {
    "arithmetic": 5432,
    "string": 789,
    "array": 123,
    "object": 456
  },
  "call_graph": {
    "name": "main",
    "duration_ms": 1234,
    "children": [
      {"name": "parse_input", "duration_ms": 123, "children": []},
      {
        "name": "process_data",
        "duration_ms": 345,
        "children": [
          {"name": "calculate_fibonacci", "duration_ms": 567, "children": []}
        ]
      },
      {"name": "output_results", "duration_ms": 199, "children": []}
    ]
  }
}
```

## Integration with Interpreter

The profiling system is integrated with the Anarchy Inference interpreter through the `ProfilingInterpreter` trait:

```rust
// Enable profiling
interpreter.enable_profiling();

// Disable profiling
interpreter.disable_profiling();

// Start a profiling session
interpreter.start_profiling_session("my_session").unwrap();

// Execute code with profiling
interpreter.profile_execute_node(&ast_node).unwrap();

// End the profiling session
interpreter.end_profiling_session().unwrap();

// Generate a profiling report
let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
```

## Best Practices

### Minimizing Overhead

To minimize profiling overhead:
- Disable profiling when not needed
- Use appropriate precision levels for time measurements
- Set minimum duration thresholds to filter out short operations
- Only track the operation types you're interested in

### Effective Profiling

For effective profiling:
- Start with high-level profiling to identify bottlenecks
- Drill down into specific functions or operations
- Compare different implementations
- Profile with realistic workloads
- Look for patterns in the data

### Interpreting Results

When interpreting profiling results:
- Focus on the most time-consuming operations
- Look for unexpected memory usage
- Check for excessive operation counts
- Examine the call graph for deep call stacks
- Compare results across different runs

## Examples

### Basic Profiling

```rust
// Create an interpreter
let mut interpreter = Interpreter::new();

// Enable profiling
interpreter.enable_profiling();

// Start a profiling session
interpreter.start_profiling_session("fibonacci").unwrap();

// Execute the Fibonacci function
let ast = parse("function fibonacci(n) { if (n <= 1) return n; return fibonacci(n-1) + fibonacci(n-2); } fibonacci(10);");
interpreter.profile_execute_nodes(&ast).unwrap();

// End the profiling session
interpreter.end_profiling_session().unwrap();

// Generate a report
let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
println!("{}", report);
```

### Custom Metrics

```rust
// Create a profiler
let mut profiler = Profiler::new();
profiler.set_enabled(true);

// Start a session
profiler.start_session("custom_metrics").unwrap();

// Start a span
let mut span_guard = profiler.start_span("my_operation", SpanType::Function).unwrap();

// Record custom metrics
span_guard.record_metric("items_processed", MetricValue::Count(1000)).unwrap();
span_guard.record_metric("average_size", MetricValue::Number(42.5)).unwrap();
span_guard.record_metric("success_rate", MetricValue::Percentage(99.9)).unwrap();

// End the span
drop(span_guard);

// End the session
profiler.end_session().unwrap();

// Generate a report
let report = profiler.generate_report(ReportFormat::Text).unwrap();
println!("{}", report);
```

### Memory Profiling

```rust
// Create an interpreter with memory profiling enabled
let mut interpreter = Interpreter::new();
let mut config = ProfilerConfig::default();
config.enabled = true;
config.memory_profiling.enabled = true;
config.memory_profiling.track_allocations = true;
config.memory_profiling.track_peak_memory = true;

let mut profiler = Profiler::with_config(config);
interpreter.set_profiler(profiler);

// Start a profiling session
interpreter.start_profiling_session("memory_test").unwrap();

// Create and manipulate objects
let ast = parse("var arr = []; for (var i = 0; i < 1000; i++) { arr.push({id: i, data: 'item ' + i}); }");
interpreter.profile_execute_nodes(&ast).unwrap();

// End the profiling session
interpreter.end_profiling_session().unwrap();

// Generate a report
let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
println!("{}", report);
```

### Operation Profiling

```rust
// Create an interpreter with operation profiling enabled
let mut interpreter = Interpreter::new();
let mut config = ProfilerConfig::default();
config.enabled = true;
config.operation_profiling.enabled = true;

let mut profiler = Profiler::with_config(config);
interpreter.set_profiler(profiler);

// Start a profiling session
interpreter.start_profiling_session("operation_test").unwrap();

// Execute code with various operations
let ast = parse("var sum = 0; for (var i = 0; i < 1000; i++) { sum += i * i; }");
interpreter.profile_execute_nodes(&ast).unwrap();

// End the profiling session
interpreter.end_profiling_session().unwrap();

// Generate a report
let report = interpreter.generate_profiling_report(ReportFormat::Text).unwrap();
println!("{}", report);
```

### JSON Report Generation

```rust
// Create an interpreter
let mut interpreter = Interpreter::new();
interpreter.enable_profiling();

// Start a profiling session
interpreter.start_profiling_session("json_report").unwrap();

// Execute some code
let ast = parse("function test() { return 42; } test();");
interpreter.profile_execute_nodes(&ast).unwrap();

// End the profiling session
interpreter.end_profiling_session().unwrap();

// Generate a JSON report
let report = interpreter.generate_profiling_report(ReportFormat::Json).unwrap();

// Save the report to a file
std::fs::write("profile.json", report).unwrap();
```
