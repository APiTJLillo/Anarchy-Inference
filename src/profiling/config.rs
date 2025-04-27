// src/profiling/config.rs - Configuration for the Performance Profiling system

use std::collections::HashSet;
use std::time::Duration;

use super::metrics::{OperationType, TimePrecision};

/// Configuration for the profiler
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Whether profiling is enabled by default
    pub enabled: bool,
    
    /// Time profiling options
    pub time_profiling: TimeProfiling,
    
    /// Memory profiling options
    pub memory_profiling: MemoryProfiling,
    
    /// Operation profiling options
    pub operation_profiling: OperationProfiling,
    
    /// Output options
    pub output: OutputOptions,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            time_profiling: TimeProfiling::default(),
            memory_profiling: MemoryProfiling::default(),
            operation_profiling: OperationProfiling::default(),
            output: OutputOptions::default(),
        }
    }
}

/// Configuration for time profiling
#[derive(Debug, Clone)]
pub struct TimeProfiling {
    /// Whether time profiling is enabled
    pub enabled: bool,
    
    /// Precision level for time measurements
    pub precision: TimePrecision,
    
    /// Minimum duration to record (for filtering)
    pub min_duration: Duration,
}

impl Default for TimeProfiling {
    fn default() -> Self {
        Self {
            enabled: true,
            precision: TimePrecision::Microsecond,
            min_duration: Duration::from_micros(10), // 10 microseconds
        }
    }
}

/// Configuration for memory profiling
#[derive(Debug, Clone)]
pub struct MemoryProfiling {
    /// Whether memory profiling is enabled
    pub enabled: bool,
    
    /// Track allocations
    pub track_allocations: bool,
    
    /// Track deallocations
    pub track_deallocations: bool,
    
    /// Track peak memory usage
    pub track_peak_memory: bool,
}

impl Default for MemoryProfiling {
    fn default() -> Self {
        Self {
            enabled: true,
            track_allocations: true,
            track_deallocations: true,
            track_peak_memory: true,
        }
    }
}

/// Configuration for operation profiling
#[derive(Debug, Clone)]
pub struct OperationProfiling {
    /// Whether operation profiling is enabled
    pub enabled: bool,
    
    /// Operation types to track
    pub tracked_operations: HashSet<OperationType>,
}

impl Default for OperationProfiling {
    fn default() -> Self {
        let mut tracked_operations = HashSet::new();
        tracked_operations.insert(OperationType::Arithmetic);
        tracked_operations.insert(OperationType::String);
        tracked_operations.insert(OperationType::Array);
        tracked_operations.insert(OperationType::Object);
        tracked_operations.insert(OperationType::Function);
        tracked_operations.insert(OperationType::Variable);
        tracked_operations.insert(OperationType::Property);
        tracked_operations.insert(OperationType::StringDictionary);
        
        Self {
            enabled: true,
            tracked_operations,
        }
    }
}

/// Configuration for output options
#[derive(Debug, Clone)]
pub struct OutputOptions {
    /// Default report format
    pub default_format: super::report::ReportFormat,
    
    /// Whether to include source locations in reports
    pub include_source_locations: bool,
    
    /// Whether to include system information in reports
    pub include_system_info: bool,
    
    /// Maximum depth for call stack in reports
    pub max_call_stack_depth: usize,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            default_format: super::report::ReportFormat::Text,
            include_source_locations: true,
            include_system_info: true,
            max_call_stack_depth: 10,
        }
    }
}
