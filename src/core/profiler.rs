// src/core/profiler.rs
// Performance profiling system for Anarchy-Inference

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::cell::Cell;

/// A performance profiler for tracking execution time and memory usage
#[derive(Debug)]
pub struct Profiler {
    /// Tracks active spans
    active_spans: Mutex<HashMap<String, SpanData>>,
    /// Completed span statistics
    completed_spans: Mutex<HashMap<String, Vec<SpanStats>>>,
    /// Global start time - using Cell for interior mutability
    start_time: Cell<Instant>,
    /// Whether profiling is enabled
    enabled: Mutex<bool>,
}

/// Data for an active profiling span
#[derive(Debug)]
struct SpanData {
    /// When the span started
    start_time: Instant,
    /// Parent span name, if any
    parent: Option<String>,
    /// Memory usage at start
    start_memory: usize,
}

/// Statistics for a completed profiling span
#[derive(Debug, Clone)]
pub struct SpanStats {
    /// Name of the span
    pub name: String,
    /// Duration of the span
    pub duration: Duration,
    /// Memory usage delta (end - start)
    pub memory_delta: isize,
    /// Parent span name, if any
    pub parent: Option<String>,
    /// When the span was recorded relative to profiler start
    pub timestamp: Duration,
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            active_spans: Mutex::new(HashMap::new()),
            completed_spans: Mutex::new(HashMap::new()),
            start_time: Cell::new(Instant::now()),
            enabled: Mutex::new(true),
        }
    }
    
    /// Start a profiling span
    pub fn start_span(&self, name: &str, current_memory: usize) -> Option<SpanGuard> {
        let enabled = self.enabled.lock().unwrap();
        if !*enabled {
            return None;
        }
        
        let mut active_spans = self.active_spans.lock().unwrap();
        
        // Find the current active parent span, if any
        let parent = active_spans.keys().next().cloned();
        
        // Record the start of this span
        active_spans.insert(name.to_string(), SpanData {
            start_time: Instant::now(),
            parent,
            start_memory: current_memory,
        });
        
        // Return a guard that will end the span when dropped
        Some(SpanGuard {
            profiler: Arc::new(self.clone()),
            name: name.to_string(),
        })
    }
    
    /// End a profiling span
    pub fn end_span(&self, name: &str, current_memory: usize) {
        let enabled = self.enabled.lock().unwrap();
        if !*enabled {
            return;
        }
        
        let mut active_spans = self.active_spans.lock().unwrap();
        let mut completed_spans = self.completed_spans.lock().unwrap();
        
        // Find and remove the span
        if let Some(span_data) = active_spans.remove(name) {
            // Calculate duration and memory delta
            let duration = span_data.start_time.elapsed();
            let memory_delta = current_memory as isize - span_data.start_memory as isize;
            
            // Record the completed span
            let stats = SpanStats {
                name: name.to_string(),
                duration,
                memory_delta,
                parent: span_data.parent,
                timestamp: span_data.start_time.duration_since(self.start_time.get()),
            };
            
            // Add to the completed spans
            completed_spans.entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(stats);
        }
    }
    
    /// Enable or disable profiling
    pub fn set_enabled(&self, enabled: bool) {
        let mut enabled_lock = self.enabled.lock().unwrap();
        *enabled_lock = enabled;
    }
    
    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        let enabled = self.enabled.lock().unwrap();
        *enabled
    }
    
    /// Reset the profiler
    pub fn reset(&self) {
        let mut active_spans = self.active_spans.lock().unwrap();
        let mut completed_spans = self.completed_spans.lock().unwrap();
        
        active_spans.clear();
        completed_spans.clear();
        
        // Reset the start time using Cell's set method - safe interior mutability
        self.start_time.set(Instant::now());
    }
    
    /// Get statistics for all completed spans
    pub fn get_stats(&self) -> HashMap<String, Vec<SpanStats>> {
        let completed_spans = self.completed_spans.lock().unwrap();
        completed_spans.clone()
    }
    
    /// Get statistics for a specific span
    pub fn get_span_stats(&self, name: &str) -> Option<Vec<SpanStats>> {
        let completed_spans = self.completed_spans.lock().unwrap();
        completed_spans.get(name).cloned()
    }
    
    /// Get the total elapsed time since the profiler was created
    pub fn total_elapsed(&self) -> Duration {
        self.start_time.get().elapsed()
    }
    
    /// Generate a report of all profiling data
    pub fn generate_report(&self) -> String {
        let completed_spans = self.completed_spans.lock().unwrap();
        let mut report = String::new();
        
        report.push_str("Performance Profiling Report\n");
        report.push_str("===========================\n\n");
        report.push_str(&format!("Total elapsed time: {:?}\n\n", self.total_elapsed()));
        
        // Calculate aggregate statistics for each span
        for (name, spans) in completed_spans.iter() {
            if spans.is_empty() {
                continue;
            }
            
            let count = spans.len();
            let total_duration: Duration = spans.iter().map(|s| s.duration).sum();
            let avg_duration = total_duration / count as u32;
            let max_duration = spans.iter().map(|s| s.duration).max().unwrap_or_default();
            let min_duration = spans.iter().map(|s| s.duration).min().unwrap_or_default();
            
            let total_memory_delta: isize = spans.iter().map(|s| s.memory_delta).sum();
            let avg_memory_delta = total_memory_delta / count as isize;
            
            report.push_str(&format!("Span: {}\n", name));
            report.push_str(&format!("  Count: {}\n", count));
            report.push_str(&format!("  Total Duration: {:?}\n", total_duration));
            report.push_str(&format!("  Average Duration: {:?}\n", avg_duration));
            report.push_str(&format!("  Min Duration: {:?}\n", min_duration));
            report.push_str(&format!("  Max Duration: {:?}\n", max_duration));
            report.push_str(&format!("  Total Memory Delta: {} bytes\n", total_memory_delta));
            report.push_str(&format!("  Average Memory Delta: {} bytes\n\n", avg_memory_delta));
        }
        
        report
    }
}

// Implement Clone for Profiler
impl Clone for Profiler {
    fn clone(&self) -> Self {
        // Create a new profiler with the same settings
        let new_profiler = Profiler {
            active_spans: Mutex::new(HashMap::new()),
            completed_spans: Mutex::new(HashMap::new()),
            start_time: Cell::new(self.start_time.get()),
            enabled: Mutex::new(*self.enabled.lock().unwrap()),
        };
        
        // Copy active spans
        {
            let active_spans = self.active_spans.lock().unwrap();
            let mut new_active_spans = new_profiler.active_spans.lock().unwrap();
            for (name, span_data) in active_spans.iter() {
                new_active_spans.insert(name.clone(), SpanData {
                    start_time: span_data.start_time,
                    parent: span_data.parent.clone(),
                    start_memory: span_data.start_memory,
                });
            }
        }
        
        // Copy completed spans
        {
            let completed_spans = self.completed_spans.lock().unwrap();
            let mut new_completed_spans = new_profiler.completed_spans.lock().unwrap();
            for (name, spans) in completed_spans.iter() {
                new_completed_spans.insert(name.clone(), spans.clone());
            }
        }
        
        new_profiler
    }
}

/// Guard object that ends a span when dropped
pub struct SpanGuard {
    /// Reference to the profiler
    profiler: Arc<Profiler>,
    /// Name of the span
    name: String,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        // End the span when the guard is dropped
        // We use 0 for current memory since we don't have access to it here
        // The actual implementation would need to get the current memory usage
        self.profiler.end_span(&self.name, 0);
    }
}

/// Convenience macro for profiling a block of code
#[macro_export]
macro_rules! profile_span {
    ($profiler:expr, $name:expr, $memory_provider:expr, $body:block) => {{
        let _guard = $profiler.start_span($name, $memory_provider.current_memory());
        let result = $body;
        result
    }};
}
