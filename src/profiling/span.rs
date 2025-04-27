// src/profiling/span.rs - Profiling span management

use std::collections::HashMap;
use std::time::Instant;

use super::metrics::{MetricValue, SpanType};

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }
    
    /// Get a string representation of this location
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

/// A profiling span represents a single profiled operation
#[derive(Debug, Clone)]
pub struct ProfilingSpan {
    /// Name of the span (e.g., function name, expression type)
    name: String,
    /// Type of span (Function, Expression, Module, etc.)
    span_type: SpanType,
    /// Start time of the span
    start_time: Instant,
    /// End time of the span (if completed)
    end_time: Option<Instant>,
    /// Parent span ID (for nested spans)
    parent_id: Option<usize>,
    /// Metrics collected for this span
    metrics: HashMap<String, MetricValue>,
    /// Source location information
    source_location: Option<SourceLocation>,
}

impl ProfilingSpan {
    /// Create a new profiling span
    pub fn new(name: String, span_type: SpanType) -> Self {
        Self {
            name,
            span_type,
            start_time: Instant::now(),
            end_time: None,
            parent_id: None,
            metrics: HashMap::new(),
            source_location: None,
        }
    }
    
    /// Create a new profiling span with source location
    pub fn with_location(name: String, span_type: SpanType, location: SourceLocation) -> Self {
        Self {
            name,
            span_type,
            start_time: Instant::now(),
            end_time: None,
            parent_id: None,
            metrics: HashMap::new(),
            source_location: Some(location),
        }
    }
    
    /// Get the name of this span
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the type of this span
    pub fn span_type(&self) -> SpanType {
        self.span_type
    }
    
    /// Get the start time of this span
    pub fn start_time(&self) -> Instant {
        self.start_time
    }
    
    /// Get the end time of this span
    pub fn end_time(&self) -> Option<Instant> {
        self.end_time
    }
    
    /// Get the duration of this span
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end_time.map(|end_time| end_time.duration_since(self.start_time))
    }
    
    /// End this span
    pub fn end(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(Instant::now());
        }
    }
    
    /// Get the parent span ID
    pub fn parent_id(&self) -> Option<usize> {
        self.parent_id
    }
    
    /// Set the parent span ID
    pub fn set_parent_id(&mut self, parent_id: Option<usize>) {
        self.parent_id = parent_id;
    }
    
    /// Add a metric to this span
    pub fn add_metric(&mut self, name: String, value: MetricValue) {
        self.metrics.insert(name, value);
    }
    
    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<&MetricValue> {
        self.metrics.get(name)
    }
    
    /// Get all metrics
    pub fn metrics(&self) -> &HashMap<String, MetricValue> {
        &self.metrics
    }
    
    /// Get the source location
    pub fn source_location(&self) -> Option<&SourceLocation> {
        self.source_location.as_ref()
    }
    
    /// Set the source location
    pub fn set_source_location(&mut self, location: SourceLocation) {
        self.source_location = Some(location);
    }
}

/// A guard for automatically ending spans
pub struct SpanGuard<'a> {
    /// Reference to the profiler
    profiler: &'a mut super::Profiler,
    /// ID of the span being guarded
    span_id: usize,
}

impl<'a> SpanGuard<'a> {
    /// Create a new span guard
    pub fn new(profiler: &'a mut super::Profiler, span_id: usize) -> Self {
        Self { profiler, span_id }
    }
    
    /// Record a metric for the current span
    pub fn record_metric(&mut self, name: &str, value: MetricValue) -> Result<(), super::ProfilerError> {
        self.profiler.record_metric(name, value)
    }
    
    /// Get the span ID
    pub fn span_id(&self) -> usize {
        self.span_id
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        // End the span when the guard is dropped
        let _ = self.profiler.end_span();
    }
}
