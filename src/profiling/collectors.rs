// src/profiling/collectors.rs - Metric collectors for the Performance Profiling system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::config::{TimeProfiling, MemoryProfiling, OperationProfiling};
use super::metrics::{MetricValue, OperationType, TimePrecision};
use super::span::ProfilingSpan;
use crate::gc::GarbageCollector;

/// Trait for metric collectors
pub trait MetricCollector: std::fmt::Debug {
    /// Initialize the collector
    fn initialize(&mut self);
    
    /// Start collecting metrics for a span
    fn start_span(&mut self, span: &ProfilingSpan);
    
    /// End collecting metrics for a span
    fn end_span(&mut self, span: &ProfilingSpan);
    
    /// Collect global metrics
    fn collect_global_metrics(&self) -> HashMap<String, MetricValue>;
    
    /// Reset the collector
    fn reset(&mut self);
}

/// Collector for time metrics
#[derive(Debug)]
pub struct TimeMetricCollector {
    /// Whether time profiling is enabled
    enabled: bool,
    
    /// Precision level for time measurements
    precision: TimePrecision,
    
    /// Minimum duration to record (for filtering)
    min_duration: std::time::Duration,
    
    /// Start times for spans
    span_start_times: HashMap<String, Instant>,
    
    /// Total time spent in each span type
    time_by_span_type: HashMap<super::metrics::SpanType, std::time::Duration>,
    
    /// Configuration
    config: TimeProfiling,
}

impl TimeMetricCollector {
    /// Create a new time metric collector
    pub fn new(config: TimeProfiling) -> Self {
        Self {
            enabled: config.enabled,
            precision: config.precision,
            min_duration: config.min_duration,
            span_start_times: HashMap::new(),
            time_by_span_type: HashMap::new(),
            config,
        }
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: TimeProfiling) {
        self.enabled = config.enabled;
        self.precision = config.precision;
        self.min_duration = config.min_duration;
        self.config = config;
    }
}

impl MetricCollector for TimeMetricCollector {
    fn initialize(&mut self) {
        self.span_start_times.clear();
        self.time_by_span_type.clear();
    }
    
    fn start_span(&mut self, span: &ProfilingSpan) {
        if !self.enabled {
            return;
        }
        
        // Store the start time
        self.span_start_times.insert(format!("{}", span.name()), Instant::now());
    }
    
    fn end_span(&mut self, span: &ProfilingSpan) {
        if !self.enabled {
            return;
        }
        
        // Get the start time
        let start_time = match self.span_start_times.remove(&format!("{}", span.name())) {
            Some(time) => time,
            None => return,
        };
        
        // Calculate the duration
        let duration = Instant::now().duration_since(start_time);
        
        // Skip if duration is too short
        if duration < self.min_duration {
            return;
        }
        
        // Update time by span type
        let span_type = span.span_type();
        let total = self.time_by_span_type.entry(span_type).or_insert_with(std::time::Duration::default);
        *total += duration;
    }
    
    fn collect_global_metrics(&self) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        if !self.enabled {
            return metrics;
        }
        
        // Add time by span type
        for (span_type, duration) in &self.time_by_span_type {
            let key = format!("time_by_span_type_{:?}", span_type);
            metrics.insert(key, MetricValue::from_duration(*duration));
        }
        
        metrics
    }
    
    fn reset(&mut self) {
        self.span_start_times.clear();
        self.time_by_span_type.clear();
    }
}

/// Collector for memory metrics
#[derive(Debug)]
pub struct MemoryMetricCollector {
    /// Whether memory profiling is enabled
    enabled: bool,
    
    /// Track allocations
    track_allocations: bool,
    
    /// Track deallocations
    track_deallocations: bool,
    
    /// Track peak memory usage
    track_peak_memory: bool,
    
    /// Reference to the garbage collector
    gc: Option<Arc<GarbageCollector>>,
    
    /// Total allocations
    total_allocations: usize,
    
    /// Total allocation size in bytes
    total_allocation_bytes: usize,
    
    /// Total deallocations
    total_deallocations: usize,
    
    /// Total deallocation size in bytes
    total_deallocation_bytes: usize,
    
    /// Peak memory usage in bytes
    peak_memory_bytes: usize,
    
    /// Current memory usage in bytes
    current_memory_bytes: usize,
    
    /// Number of garbage collections
    gc_collections: usize,
    
    /// Bytes reclaimed by garbage collection
    gc_reclaimed_bytes: usize,
    
    /// Configuration
    config: MemoryProfiling,
}

impl MemoryMetricCollector {
    /// Create a new memory metric collector
    pub fn new(config: MemoryProfiling) -> Self {
        Self {
            enabled: config.enabled,
            track_allocations: config.track_allocations,
            track_deallocations: config.track_deallocations,
            track_peak_memory: config.track_peak_memory,
            gc: None,
            total_allocations: 0,
            total_allocation_bytes: 0,
            total_deallocations: 0,
            total_deallocation_bytes: 0,
            peak_memory_bytes: 0,
            current_memory_bytes: 0,
            gc_collections: 0,
            gc_reclaimed_bytes: 0,
            config,
        }
    }
    
    /// Set the garbage collector
    pub fn set_garbage_collector(&mut self, gc: Arc<GarbageCollector>) {
        self.gc = Some(gc);
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: MemoryProfiling) {
        self.enabled = config.enabled;
        self.track_allocations = config.track_allocations;
        self.track_deallocations = config.track_deallocations;
        self.track_peak_memory = config.track_peak_memory;
        self.config = config;
    }
    
    /// Record an allocation
    pub fn record_allocation(&mut self, size: usize) {
        if !self.enabled || !self.track_allocations {
            return;
        }
        
        self.total_allocations += 1;
        self.total_allocation_bytes += size;
        self.current_memory_bytes += size;
        
        if self.track_peak_memory && self.current_memory_bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = self.current_memory_bytes;
        }
    }
    
    /// Record a deallocation
    pub fn record_deallocation(&mut self, size: usize) {
        if !self.enabled || !self.track_deallocations {
            return;
        }
        
        self.total_deallocations += 1;
        self.total_deallocation_bytes += size;
        self.current_memory_bytes = self.current_memory_bytes.saturating_sub(size);
    }
    
    /// Record a garbage collection
    pub fn record_gc_collection(&mut self, reclaimed_bytes: usize) {
        if !self.enabled {
            return;
        }
        
        self.gc_collections += 1;
        self.gc_reclaimed_bytes += reclaimed_bytes;
    }
    
    /// Update memory metrics from the garbage collector
    pub fn update_from_gc(&mut self) {
        if !self.enabled || self.gc.is_none() {
            return;
        }
        
        if let Some(gc) = &self.gc {
            let stats = gc.get_stats();
            
            if self.track_allocations {
                self.total_allocations = stats.allocations;
                self.total_allocation_bytes = stats.total_memory;
            }
            
            if self.track_deallocations {
                self.total_deallocations = stats.deallocations;
            }
            
            if self.track_peak_memory {
                self.peak_memory_bytes = stats.peak_memory;
            }
            
            self.current_memory_bytes = stats.total_memory;
            self.gc_collections = stats.collections_performed;
        }
    }
}

impl MetricCollector for MemoryMetricCollector {
    fn initialize(&mut self) {
        self.total_allocations = 0;
        self.total_allocation_bytes = 0;
        self.total_deallocations = 0;
        self.total_deallocation_bytes = 0;
        self.peak_memory_bytes = 0;
        self.current_memory_bytes = 0;
        self.gc_collections = 0;
        self.gc_reclaimed_bytes = 0;
    }
    
    fn start_span(&mut self, _span: &ProfilingSpan) {
        if !self.enabled {
            return;
        }
        
        // Update memory metrics from the garbage collector
        self.update_from_gc();
    }
    
    fn end_span(&mut self, _span: &ProfilingSpan) {
        if !self.enabled {
            return;
        }
        
        // Update memory metrics from the garbage collector
        self.update_from_gc();
    }
    
    fn collect_global_metrics(&self) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        if !self.enabled {
            return metrics;
        }
        
        // Add memory metrics
        if self.track_allocations {
            metrics.insert("mem_total_allocations".to_string(), MetricValue::Count(self.total_allocations));
            metrics.insert("mem_total_allocation_bytes".to_string(), MetricValue::Memory(self.total_allocation_bytes));
        }
        
        if self.track_deallocations {
            metrics.insert("mem_total_deallocations".to_string(), MetricValue::Count(self.total_deallocations));
            metrics.insert("mem_total_deallocation_bytes".to_string(), MetricValue::Memory(self.total_deallocation_bytes));
        }
        
        if self.track_peak_memory {
            metrics.insert("mem_peak_memory_bytes".to_string(), MetricValue::Memory(self.peak_memory_bytes));
        }
        
        metrics.insert("mem_current_memory_bytes".to_string(), MetricValue::Memory(self.current_memory_bytes));
        metrics.insert("mem_gc_collections".to_string(), MetricValue::Count(self.gc_collections));
        metrics.insert("mem_gc_reclaimed_bytes".to_string(), MetricValue::Memory(self.gc_reclaimed_bytes));
        
        metrics
    }
    
    fn reset(&mut self) {
        self.total_allocations = 0;
        self.total_allocation_bytes = 0;
        self.total_deallocations = 0;
        self.total_deallocation_bytes = 0;
        self.peak_memory_bytes = 0;
        self.current_memory_bytes = 0;
        self.gc_collections = 0;
        self.gc_reclaimed_bytes = 0;
    }
}

/// Collector for operation metrics
#[derive(Debug)]
pub struct OperationMetricCollector {
    /// Whether operation profiling is enabled
    enabled: bool,
    
    /// Operation types to track
    tracked_operations: std::collections::HashSet<OperationType>,
    
    /// Operation counts by type
    operation_counts: HashMap<OperationType, usize>,
    
    /// Configuration
    config: OperationProfiling,
}

impl OperationMetricCollector {
    /// Create a new operation metric collector
    pub fn new(config: OperationProfiling) -> Self {
        Self {
            enabled: config.enabled,
            tracked_operations: config.tracked_operations.clone(),
            operation_counts: HashMap::new(),
            config,
        }
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: OperationProfiling) {
        self.enabled = config.enabled;
        self.tracked_operations = config.tracked_operations.clone();
        self.config = config;
    }
    
    /// Record an operation
    pub fn record_operation(&mut self, operation_type: OperationType) {
        if !self.enabled || !self.tracked_operations.contains(&operation_type) {
            return;
        }
        
        let count = self.operation_counts.entry(operation_type).or_insert(0);
        *count += 1;
    }
    
    /// Record multiple operations
    pub fn record_operations(&mut self, operation_type: OperationType, count: usize) {
        if !self.enabled || !self.tracked_operations.contains(&operation_type) {
            return;
        }
        
        let op_count = self.operation_counts.entry(operation_type).or_insert(0);
        *op_count += count;
    }
}

impl MetricCollector for OperationMetricCollector {
    fn initialize(&mut self) {
        self.operation_counts.clear();
    }
    
    fn start_span(&mut self, _span: &ProfilingSpan) {
        // No action needed at span start for operation metrics
    }
    
    fn end_span(&mut self, _span: &ProfilingSpan) {
        // No action needed at span end for operation metrics
    }
    
    fn collect_global_metrics(&self) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        if !self.enabled {
            return metrics;
        }
        
        // Add operation counts
        for (op_type, count) in &self.operation_counts {
            let key = format!("op_count_{}", op_type);
            metrics.insert(key, MetricValue::Count(*count));
        }
        
        metrics
    }
    
    fn reset(&mut self) {
        self.operation_counts.clear();
    }
}
