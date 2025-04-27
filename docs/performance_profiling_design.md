# Performance Profiling System Design

This document outlines the design for the performance profiling system in Anarchy Inference. The system will provide comprehensive performance metrics to help developers identify bottlenecks, optimize code, and understand resource usage patterns.

## Table of Contents

1. [Overview](#overview)
2. [Design Goals](#design-goals)
3. [Architecture](#architecture)
4. [Core Components](#core-components)
5. [Profiling Metrics](#profiling-metrics)
6. [Integration Points](#integration-points)
7. [API Design](#api-design)
8. [Configuration Options](#configuration-options)
9. [Output Formats](#output-formats)
10. [Implementation Plan](#implementation-plan)

## Overview

The performance profiling system will provide detailed insights into the runtime behavior of Anarchy Inference programs. It will track execution time, memory usage, operation counts, and other metrics at various levels of granularity, from individual expressions to entire modules.

The system is designed to be:
- Lightweight when disabled (minimal overhead)
- Configurable (different levels of detail)
- Comprehensive (multiple metric types)
- Non-intrusive (minimal code changes to existing components)
- Extensible (easy to add new metrics)

## Design Goals

1. **Minimal Overhead**: The profiling system should add minimal overhead when disabled and reasonable overhead when enabled.
2. **Accurate Measurements**: Provide precise timing and memory usage measurements.
3. **Flexible Granularity**: Support profiling at different levels (expression, function, module).
4. **Comprehensive Metrics**: Track execution time, memory usage, operation counts, and other relevant metrics.
5. **Easy Integration**: Integrate seamlessly with existing code with minimal changes.
6. **Configurable**: Allow users to enable/disable specific metrics and set profiling levels.
7. **Useful Output**: Generate reports in multiple formats (text, JSON, visualization).

## Architecture

The profiling system will follow a modular architecture with the following components:

1. **Profiler Core**: Central component that manages profiling sessions and collects metrics.
2. **Metric Collectors**: Specialized components for collecting different types of metrics.
3. **Instrumentation Points**: Strategic locations in the codebase where profiling data is collected.
4. **Report Generators**: Components that format and output profiling data.
5. **Configuration System**: Manages profiling settings and options.

## Core Components

### Profiler

The `Profiler` is the central component that manages profiling sessions and coordinates metric collection:

```rust
pub struct Profiler {
    // Whether profiling is enabled
    enabled: bool,
    // Current profiling session
    current_session: Option<ProfilingSession>,
    // Configuration options
    config: ProfilerConfig,
    // Metric collectors
    time_metrics: TimeMetricCollector,
    memory_metrics: MemoryMetricCollector,
    operation_metrics: OperationMetricCollector,
    // Report generators
    report_generators: Vec<Box<dyn ReportGenerator>>,
}
```

### ProfilingSession

A `ProfilingSession` represents a single profiling run:

```rust
pub struct ProfilingSession {
    // Unique identifier for the session
    id: String,
    // Start time of the session
    start_time: Instant,
    // End time of the session (if completed)
    end_time: Option<Instant>,
    // Call stack for tracking nested operations
    call_stack: Vec<ProfilingSpan>,
    // Completed spans
    completed_spans: Vec<ProfilingSpan>,
    // Session-wide metrics
    global_metrics: HashMap<String, MetricValue>,
}
```

### ProfilingSpan

A `ProfilingSpan` represents a single profiled operation:

```rust
pub struct ProfilingSpan {
    // Name of the span (e.g., function name, expression type)
    name: String,
    // Type of span (Function, Expression, Module, etc.)
    span_type: SpanType,
    // Start time of the span
    start_time: Instant,
    // End time of the span (if completed)
    end_time: Option<Instant>,
    // Parent span ID (for nested spans)
    parent_id: Option<usize>,
    // Metrics collected for this span
    metrics: HashMap<String, MetricValue>,
    // Source location information
    source_location: Option<SourceLocation>,
}
```

### MetricCollectors

Specialized components for collecting different types of metrics:

```rust
pub trait MetricCollector {
    // Initialize the collector
    fn initialize(&mut self);
    // Start collecting metrics for a span
    fn start_span(&mut self, span: &mut ProfilingSpan);
    // End collecting metrics for a span
    fn end_span(&mut self, span: &mut ProfilingSpan);
    // Collect global metrics
    fn collect_global_metrics(&self) -> HashMap<String, MetricValue>;
    // Reset the collector
    fn reset(&mut self);
}
```

#### TimeMetricCollector

Collects execution time metrics:

```rust
pub struct TimeMetricCollector {
    // Whether time profiling is enabled
    enabled: bool,
    // Precision level for time measurements
    precision: TimePrecision,
}
```

#### MemoryMetricCollector

Collects memory usage metrics:

```rust
pub struct MemoryMetricCollector {
    // Whether memory profiling is enabled
    enabled: bool,
    // Track allocations
    track_allocations: bool,
    // Track deallocations
    track_deallocations: bool,
    // Track peak memory usage
    track_peak_memory: bool,
    // Reference to the garbage collector
    gc: Option<Arc<GarbageCollector>>,
}
```

#### OperationMetricCollector

Collects operation count metrics:

```rust
pub struct OperationMetricCollector {
    // Whether operation profiling is enabled
    enabled: bool,
    // Operation counts by type
    operation_counts: HashMap<OperationType, usize>,
}
```

### ReportGenerators

Components that format and output profiling data:

```rust
pub trait ReportGenerator {
    // Generate a report from a profiling session
    fn generate_report(&self, session: &ProfilingSession) -> Result<String, ProfilerError>;
    // Get the format of the report
    fn format(&self) -> ReportFormat;
}
```

## Profiling Metrics

The profiling system will collect the following metrics:

### Time Metrics

- **Total Execution Time**: Overall execution time of the program.
- **Function Execution Time**: Time spent in each function.
- **Expression Execution Time**: Time spent evaluating each expression.
- **Module Initialization Time**: Time spent initializing modules.
- **Garbage Collection Time**: Time spent in garbage collection.

### Memory Metrics

- **Total Memory Usage**: Overall memory usage of the program.
- **Peak Memory Usage**: Maximum memory usage during execution.
- **Allocations**: Number and size of memory allocations.
- **Deallocations**: Number and size of memory deallocations.
- **Object Counts**: Number of objects by type.
- **Reference Counts**: Distribution of reference counts.

### Operation Metrics

- **Operation Counts**: Number of operations by type (arithmetic, string, array, etc.).
- **Function Calls**: Number of function calls by function.
- **Variable Accesses**: Number of variable accesses by variable.
- **Property Accesses**: Number of property accesses by object type.
- **String Dictionary Operations**: Number of string dictionary operations.

## Integration Points

The profiling system will be integrated at the following points in the codebase:

### Interpreter

- **execute_node**: Profile execution time and operation counts for each AST node.
- **execute_nodes**: Profile execution time for blocks of nodes.
- **Binary/Unary Operations**: Profile operation counts and execution time for operations.

### Garbage Collector

- **allocate**: Profile memory allocations.
- **collect**: Profile garbage collection time and memory reclaimed.
- **increment_ref_count/decrement_ref_count**: Profile reference count changes.

### Value Management

- **Value Creation**: Profile object creation by type.
- **Property Access**: Profile property access operations.
- **Array Operations**: Profile array operations.

### Module System

- **Module Loading**: Profile module loading time.
- **Import Resolution**: Profile import resolution time.

## API Design

The profiling system will provide the following API:

### Profiler API

```rust
impl Profiler {
    // Create a new profiler with default configuration
    pub fn new() -> Self;
    
    // Create a new profiler with custom configuration
    pub fn with_config(config: ProfilerConfig) -> Self;
    
    // Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool);
    
    // Start a new profiling session
    pub fn start_session(&mut self, name: &str) -> Result<(), ProfilerError>;
    
    // End the current profiling session
    pub fn end_session(&mut self) -> Result<ProfilingSession, ProfilerError>;
    
    // Start a new profiling span
    pub fn start_span(&mut self, name: &str, span_type: SpanType) -> Result<SpanGuard, ProfilerError>;
    
    // End the current profiling span
    pub fn end_span(&mut self) -> Result<(), ProfilerError>;
    
    // Record a metric value
    pub fn record_metric(&mut self, name: &str, value: MetricValue) -> Result<(), ProfilerError>;
    
    // Generate a report for the current session
    pub fn generate_report(&self, format: ReportFormat) -> Result<String, ProfilerError>;
    
    // Reset the profiler
    pub fn reset(&mut self);
}
```

### SpanGuard API

`SpanGuard` provides a RAII-style API for automatically ending spans:

```rust
impl SpanGuard {
    // Create a new span guard
    pub fn new(profiler: &mut Profiler, span_id: usize) -> Self;
    
    // Record a metric value for the current span
    pub fn record_metric(&mut self, name: &str, value: MetricValue) -> Result<(), ProfilerError>;
}

impl Drop for SpanGuard {
    // Automatically end the span when the guard is dropped
    fn drop(&mut self) {
        self.profiler.end_span().unwrap_or_default();
    }
}
```

### Macros

The profiling system will provide macros for easy integration:

```rust
// Profile a block of code
#[macro_export]
macro_rules! profile_block {
    ($profiler:expr, $name:expr, $span_type:expr, $block:block) => {
        let _guard = $profiler.start_span($name, $span_type)?;
        let result = $block;
        result
    };
}

// Profile a function
#[macro_export]
macro_rules! profile_fn {
    ($profiler:expr, $block:block) => {
        let _guard = $profiler.start_span(function_name!(), SpanType::Function)?;
        let result = $block;
        result
    };
}
```

## Configuration Options

The profiling system will be configurable through the `ProfilerConfig` struct:

```rust
pub struct ProfilerConfig {
    // Whether profiling is enabled by default
    pub enabled: bool,
    
    // Time profiling options
    pub time_profiling: TimeProfiling,
    
    // Memory profiling options
    pub memory_profiling: MemoryProfiling,
    
    // Operation profiling options
    pub operation_profiling: OperationProfiling,
    
    // Output options
    pub output: OutputOptions,
}

pub struct TimeProfiling {
    // Whether time profiling is enabled
    pub enabled: bool,
    
    // Precision level for time measurements
    pub precision: TimePrecision,
    
    // Minimum duration to record (for filtering)
    pub min_duration: Duration,
}

pub struct MemoryProfiling {
    // Whether memory profiling is enabled
    pub enabled: bool,
    
    // Track allocations
    pub track_allocations: bool,
    
    // Track deallocations
    pub track_deallocations: bool,
    
    // Track peak memory usage
    pub track_peak_memory: bool,
}

pub struct OperationProfiling {
    // Whether operation profiling is enabled
    pub enabled: bool,
    
    // Operation types to track
    pub tracked_operations: HashSet<OperationType>,
}

pub struct OutputOptions {
    // Default report format
    pub default_format: ReportFormat,
    
    // Whether to include source locations in reports
    pub include_source_locations: bool,
    
    // Whether to include system information in reports
    pub include_system_info: bool,
    
    // Maximum depth for call stack in reports
    pub max_call_stack_depth: usize,
}
```

## Output Formats

The profiling system will support the following output formats:

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

### Visualization Format

The profiling system will also support generating data for visualization tools:

- **Flame Graphs**: For visualizing call stacks and execution time.
- **Timeline Views**: For visualizing execution over time.
- **Memory Usage Graphs**: For visualizing memory usage over time.

## Implementation Plan

The implementation will proceed in the following phases:

1. **Core Infrastructure**:
   - Implement the `Profiler` and `ProfilingSession` classes.
   - Implement the basic metric collection infrastructure.
   - Add configuration options.

2. **Time Profiling**:
   - Implement the `TimeMetricCollector`.
   - Add instrumentation points for time profiling.
   - Implement time-based reports.

3. **Memory Profiling**:
   - Implement the `MemoryMetricCollector`.
   - Integrate with the garbage collector.
   - Add instrumentation points for memory profiling.
   - Implement memory-based reports.

4. **Operation Profiling**:
   - Implement the `OperationMetricCollector`.
   - Add instrumentation points for operation profiling.
   - Implement operation-based reports.

5. **Report Generation**:
   - Implement text report generation.
   - Implement JSON report generation.
   - Implement visualization data generation.

6. **Integration and Testing**:
   - Integrate the profiling system with the interpreter.
   - Add comprehensive tests.
   - Optimize for minimal overhead.

7. **Documentation**:
   - Document the API.
   - Provide usage examples.
   - Document configuration options.
