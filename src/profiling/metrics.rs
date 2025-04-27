// src/profiling/metrics.rs - Metric types for the Performance Profiling system

use std::fmt;
use std::time::Duration;

/// Types of metrics that can be collected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Time-based metric
    Time,
    /// Memory-based metric
    Memory,
    /// Count-based metric
    Count,
    /// Percentage-based metric
    Percentage,
    /// Custom metric
    Custom,
}

/// Value of a metric
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Time duration in nanoseconds
    Time(u64),
    /// Memory size in bytes
    Memory(usize),
    /// Count value
    Count(usize),
    /// Percentage value (0-100)
    Percentage(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Number value
    Number(f64),
}

impl MetricValue {
    /// Get the type of this metric value
    pub fn get_type(&self) -> MetricType {
        match self {
            Self::Time(_) => MetricType::Time,
            Self::Memory(_) => MetricType::Memory,
            Self::Count(_) => MetricType::Count,
            Self::Percentage(_) => MetricType::Percentage,
            _ => MetricType::Custom,
        }
    }
    
    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::Time(ns) => {
                if *ns < 1_000 {
                    format!("{}ns", ns)
                } else if *ns < 1_000_000 {
                    format!("{:.2}µs", *ns as f64 / 1_000.0)
                } else if *ns < 1_000_000_000 {
                    format!("{:.2}ms", *ns as f64 / 1_000_000.0)
                } else {
                    format!("{:.2}s", *ns as f64 / 1_000_000_000.0)
                }
            },
            Self::Memory(bytes) => {
                if *bytes < 1_024 {
                    format!("{}B", bytes)
                } else if *bytes < 1_048_576 {
                    format!("{:.2}KB", *bytes as f64 / 1_024.0)
                } else if *bytes < 1_073_741_824 {
                    format!("{:.2}MB", *bytes as f64 / 1_048_576.0)
                } else {
                    format!("{:.2}GB", *bytes as f64 / 1_073_741_824.0)
                }
            },
            Self::Count(count) => format!("{}", count),
            Self::Percentage(pct) => format!("{:.2}%", pct),
            Self::String(s) => s.clone(),
            Self::Boolean(b) => b.to_string(),
            Self::Number(n) => format!("{}", n),
        }
    }
    
    /// Create a time metric from a Duration
    pub fn from_duration(duration: Duration) -> Self {
        Self::Time(duration.as_nanos() as u64)
    }
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Types of operations that can be profiled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Arithmetic operations
    Arithmetic,
    /// String operations
    String,
    /// Array operations
    Array,
    /// Object operations
    Object,
    /// Function operations
    Function,
    /// Variable operations
    Variable,
    /// Property operations
    Property,
    /// String dictionary operations
    StringDictionary,
    /// Other operations
    Other,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arithmetic => write!(f, "Arithmetic"),
            Self::String => write!(f, "String"),
            Self::Array => write!(f, "Array"),
            Self::Object => write!(f, "Object"),
            Self::Function => write!(f, "Function"),
            Self::Variable => write!(f, "Variable"),
            Self::Property => write!(f, "Property"),
            Self::StringDictionary => write!(f, "StringDictionary"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Precision level for time measurements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePrecision {
    /// Nanosecond precision
    Nanosecond,
    /// Microsecond precision
    Microsecond,
    /// Millisecond precision
    Millisecond,
    /// Second precision
    Second,
}

impl TimePrecision {
    /// Convert a duration to the appropriate precision
    pub fn convert_duration(&self, duration: Duration) -> u64 {
        match self {
            Self::Nanosecond => duration.as_nanos() as u64,
            Self::Microsecond => duration.as_micros() as u64,
            Self::Millisecond => duration.as_millis() as u64,
            Self::Second => duration.as_secs(),
        }
    }
    
    /// Get the unit name for this precision
    pub fn unit_name(&self) -> &'static str {
        match self {
            Self::Nanosecond => "ns",
            Self::Microsecond => "µs",
            Self::Millisecond => "ms",
            Self::Second => "s",
        }
    }
}

/// Types of profiling spans
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpanType {
    /// Function span
    Function,
    /// Expression span
    Expression,
    /// Module span
    Module,
    /// Block span
    Block,
    /// Custom span
    Custom,
}

impl fmt::Display for SpanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => write!(f, "Function"),
            Self::Expression => write!(f, "Expression"),
            Self::Module => write!(f, "Module"),
            Self::Block => write!(f, "Block"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}
