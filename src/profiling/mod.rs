// src/profiling/mod.rs - Performance Profiling module for Anarchy Inference

//! Performance Profiling System for Anarchy Inference
//! 
//! This module provides a comprehensive performance profiling system for
//! tracking execution time, memory usage, and operation counts in Anarchy Inference.

mod config;
mod metrics;
mod report;
mod session;
mod span;
mod collectors;

pub use config::{ProfilerConfig, TimeProfiling, MemoryProfiling, OperationProfiling, OutputOptions};
pub use metrics::{MetricValue, MetricType, OperationType, TimePrecision, SpanType};
pub use report::{ReportGenerator, ReportFormat, TextReportGenerator, JsonReportGenerator};
pub use session::ProfilingSession;
pub use span::{ProfilingSpan, SourceLocation, SpanGuard};
pub use collectors::{MetricCollector, TimeMetricCollector, MemoryMetricCollector, OperationMetricCollector};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::error::LangError;

/// Error type for profiling operations
#[derive(Debug, Clone)]
pub enum ProfilerError {
    /// No active session
    NoActiveSession,
    /// No active span
    NoActiveSpan,
    /// Invalid span ID
    InvalidSpanId,
    /// Profiling disabled
    ProfilingDisabled,
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Report generation error
    ReportGenerationError(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for ProfilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveSession => write!(f, "No active profiling session"),
            Self::NoActiveSpan => write!(f, "No active profiling span"),
            Self::InvalidSpanId => write!(f, "Invalid span ID"),
            Self::ProfilingDisabled => write!(f, "Profiling is disabled"),
            Self::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::ReportGenerationError(msg) => write!(f, "Report generation error: {}", msg),
            Self::Other(msg) => write!(f, "Profiler error: {}", msg),
        }
    }
}

impl std::error::Error for ProfilerError {}

impl From<ProfilerError> for LangError {
    fn from(err: ProfilerError) -> Self {
        LangError::runtime_error(&format!("Profiler error: {}", err))
    }
}

/// The main profiler struct that manages profiling sessions and metrics
#[derive(Debug)]
pub struct Profiler {
    /// Whether profiling is enabled
    enabled: bool,
    /// Current profiling session
    current_session: Option<Arc<Mutex<ProfilingSession>>>,
    /// Configuration options
    config: ProfilerConfig,
    /// Time metric collector
    time_metrics: TimeMetricCollector,
    /// Memory metric collector
    memory_metrics: MemoryMetricCollector,
    /// Operation metric collector
    operation_metrics: OperationMetricCollector,
    /// Report generators
    report_generators: HashMap<ReportFormat, Box<dyn ReportGenerator>>,
}

impl Profiler {
    /// Create a new profiler with default configuration
    pub fn new() -> Self {
        let config = ProfilerConfig::default();
        Self::with_config(config)
    }
    
    /// Create a new profiler with custom configuration
    pub fn with_config(config: ProfilerConfig) -> Self {
        let time_metrics = TimeMetricCollector::new(config.time_profiling.clone());
        let memory_metrics = MemoryMetricCollector::new(config.memory_profiling.clone());
        let operation_metrics = OperationMetricCollector::new(config.operation_profiling.clone());
        
        let mut report_generators = HashMap::new();
        report_generators.insert(ReportFormat::Text, Box::new(TextReportGenerator::new()) as Box<dyn ReportGenerator>);
        report_generators.insert(ReportFormat::Json, Box::new(JsonReportGenerator::new()) as Box<dyn ReportGenerator>);
        
        Self {
            enabled: config.enabled,
            current_session: None,
            config,
            time_metrics,
            memory_metrics,
            operation_metrics,
            report_generators,
        }
    }
    
    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Start a new profiling session
    pub fn start_session(&mut self, name: &str) -> Result<(), ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        // End any existing session
        if self.current_session.is_some() {
            self.end_session()?;
        }
        
        // Initialize metric collectors
        self.time_metrics.initialize();
        self.memory_metrics.initialize();
        self.operation_metrics.initialize();
        
        // Create a new session
        let session = ProfilingSession::new(name.to_string());
        self.current_session = Some(Arc::new(Mutex::new(session)));
        
        Ok(())
    }
    
    /// End the current profiling session
    pub fn end_session(&mut self) -> Result<Arc<Mutex<ProfilingSession>>, ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        let session = self.current_session.take()
            .ok_or(ProfilerError::NoActiveSession)?;
        
        // End the session
        {
            let mut session_guard = session.lock().unwrap();
            session_guard.end();
            
            // Collect global metrics
            let time_metrics = self.time_metrics.collect_global_metrics();
            let memory_metrics = self.memory_metrics.collect_global_metrics();
            let operation_metrics = self.operation_metrics.collect_global_metrics();
            
            // Add global metrics to the session
            for (name, value) in time_metrics {
                session_guard.add_global_metric(name, value);
            }
            
            for (name, value) in memory_metrics {
                session_guard.add_global_metric(name, value);
            }
            
            for (name, value) in operation_metrics {
                session_guard.add_global_metric(name, value);
            }
        }
        
        // Reset metric collectors
        self.time_metrics.reset();
        self.memory_metrics.reset();
        self.operation_metrics.reset();
        
        Ok(session)
    }
    
    /// Start a new profiling span
    pub fn start_span(&mut self, name: &str, span_type: SpanType) -> Result<SpanGuard, ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        // Get the current session
        let session = self.current_session.as_ref()
            .ok_or(ProfilerError::NoActiveSession)?;
        
        // Create a new span
        let span_id = {
            let mut session_guard = session.lock().unwrap();
            let span = ProfilingSpan::new(name.to_string(), span_type);
            
            // Start metric collection for this span
            self.time_metrics.start_span(&span);
            self.memory_metrics.start_span(&span);
            self.operation_metrics.start_span(&span);
            
            // Add the span to the session
            session_guard.start_span(span)
        };
        
        // Create a span guard
        Ok(SpanGuard::new(self, span_id))
    }
    
    /// End the current profiling span
    pub fn end_span(&mut self) -> Result<(), ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        // Get the current session
        let session = self.current_session.as_ref()
            .ok_or(ProfilerError::NoActiveSession)?;
        
        // End the current span
        let span = {
            let mut session_guard = session.lock().unwrap();
            session_guard.end_current_span()
                .ok_or(ProfilerError::NoActiveSpan)?
        };
        
        // End metric collection for this span
        self.time_metrics.end_span(&span);
        self.memory_metrics.end_span(&span);
        self.operation_metrics.end_span(&span);
        
        Ok(())
    }
    
    /// Record a metric value for the current span
    pub fn record_metric(&mut self, name: &str, value: MetricValue) -> Result<(), ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        // Get the current session
        let session = self.current_session.as_ref()
            .ok_or(ProfilerError::NoActiveSession)?;
        
        // Record the metric
        let mut session_guard = session.lock().unwrap();
        session_guard.record_metric(name, value)
            .map_err(|e| ProfilerError::Other(e.to_string()))?;
        
        Ok(())
    }
    
    /// Record a global metric value
    pub fn record_global_metric(&mut self, name: &str, value: MetricValue) -> Result<(), ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        // Get the current session
        let session = self.current_session.as_ref()
            .ok_or(ProfilerError::NoActiveSession)?;
        
        // Record the global metric
        let mut session_guard = session.lock().unwrap();
        session_guard.add_global_metric(name.to_string(), value);
        
        Ok(())
    }
    
    /// Generate a report for the current session
    pub fn generate_report(&self, format: ReportFormat) -> Result<String, ProfilerError> {
        if !self.enabled {
            return Err(ProfilerError::ProfilingDisabled);
        }
        
        // Get the current session
        let session = self.current_session.as_ref()
            .ok_or(ProfilerError::NoActiveSession)?;
        
        // Get the report generator
        let generator = self.report_generators.get(&format)
            .ok_or_else(|| ProfilerError::Other(format!("No report generator for format {:?}", format)))?;
        
        // Generate the report
        let session_guard = session.lock().unwrap();
        generator.generate_report(&session_guard)
            .map_err(|e| ProfilerError::ReportGenerationError(e.to_string()))
    }
    
    /// Reset the profiler
    pub fn reset(&mut self) {
        // End any existing session
        if self.current_session.is_some() {
            let _ = self.end_session();
        }
        
        // Reset metric collectors
        self.time_metrics.reset();
        self.memory_metrics.reset();
        self.operation_metrics.reset();
    }
    
    /// Get the current profiling session
    pub fn current_session(&self) -> Option<Arc<Mutex<ProfilingSession>>> {
        self.current_session.clone()
    }
    
    /// Get the profiler configuration
    pub fn config(&self) -> &ProfilerConfig {
        &self.config
    }
    
    /// Update the profiler configuration
    pub fn update_config(&mut self, config: ProfilerConfig) {
        self.config = config;
        self.time_metrics.update_config(self.config.time_profiling.clone());
        self.memory_metrics.update_config(self.config.memory_profiling.clone());
        self.operation_metrics.update_config(self.config.operation_profiling.clone());
    }
    
    /// Get a reference to the time metric collector
    pub fn time_metrics(&self) -> &TimeMetricCollector {
        &self.time_metrics
    }
    
    /// Get a reference to the memory metric collector
    pub fn memory_metrics(&self) -> &MemoryMetricCollector {
        &self.memory_metrics
    }
    
    /// Get a reference to the operation metric collector
    pub fn operation_metrics(&self) -> &OperationMetricCollector {
        &self.operation_metrics
    }
}

/// Macro to profile a block of code
#[macro_export]
macro_rules! profile_block {
    ($profiler:expr, $name:expr, $span_type:expr, $block:block) => {{
        let guard = $profiler.start_span($name, $span_type);
        let result = $block;
        drop(guard);
        result
    }};
}

/// Macro to profile a function
#[macro_export]
macro_rules! profile_fn {
    ($profiler:expr, $block:block) => {{
        let guard = $profiler.start_span(function_name!(), SpanType::Function);
        let result = $block;
        drop(guard);
        result
    }};
}

/// Macro to get the current function name
#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
