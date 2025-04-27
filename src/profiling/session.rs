// src/profiling/session.rs - Profiling session management

use std::collections::HashMap;
use std::time::Instant;

use super::metrics::MetricValue;
use super::span::ProfilingSpan;

/// A profiling session represents a single profiling run
#[derive(Debug)]
pub struct ProfilingSession {
    /// Name of the session
    name: String,
    /// Start time of the session
    start_time: Instant,
    /// End time of the session (if completed)
    end_time: Option<Instant>,
    /// Call stack for tracking nested operations
    call_stack: Vec<usize>,
    /// All spans in this session
    spans: Vec<ProfilingSpan>,
    /// Session-wide metrics
    global_metrics: HashMap<String, MetricValue>,
}

impl ProfilingSession {
    /// Create a new profiling session
    pub fn new(name: String) -> Self {
        Self {
            name,
            start_time: Instant::now(),
            end_time: None,
            call_stack: Vec::new(),
            spans: Vec::new(),
            global_metrics: HashMap::new(),
        }
    }
    
    /// Get the name of this session
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the start time of this session
    pub fn start_time(&self) -> Instant {
        self.start_time
    }
    
    /// Get the end time of this session
    pub fn end_time(&self) -> Option<Instant> {
        self.end_time
    }
    
    /// Get the duration of this session
    pub fn duration(&self) -> std::time::Duration {
        match self.end_time {
            Some(end_time) => end_time.duration_since(self.start_time),
            None => Instant::now().duration_since(self.start_time),
        }
    }
    
    /// End this session
    pub fn end(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(Instant::now());
        }
    }
    
    /// Start a new span
    pub fn start_span(&mut self, mut span: ProfilingSpan) -> usize {
        // Set the parent span if there is one
        if let Some(&parent_id) = self.call_stack.last() {
            span.set_parent_id(Some(parent_id));
        }
        
        // Add the span to the list
        let span_id = self.spans.len();
        self.spans.push(span);
        
        // Add to the call stack
        self.call_stack.push(span_id);
        
        span_id
    }
    
    /// End the current span
    pub fn end_current_span(&mut self) -> Option<ProfilingSpan> {
        // Pop the current span from the call stack
        let span_id = self.call_stack.pop()?;
        
        // End the span
        let span = &mut self.spans[span_id];
        span.end();
        
        Some(span.clone())
    }
    
    /// Get a span by ID
    pub fn get_span(&self, span_id: usize) -> Option<&ProfilingSpan> {
        self.spans.get(span_id)
    }
    
    /// Get a mutable reference to a span by ID
    pub fn get_span_mut(&mut self, span_id: usize) -> Option<&mut ProfilingSpan> {
        self.spans.get_mut(span_id)
    }
    
    /// Get all spans in this session
    pub fn spans(&self) -> &[ProfilingSpan] {
        &self.spans
    }
    
    /// Get the current span
    pub fn current_span(&self) -> Option<&ProfilingSpan> {
        self.call_stack.last().and_then(|&span_id| self.spans.get(span_id))
    }
    
    /// Get a mutable reference to the current span
    pub fn current_span_mut(&mut self) -> Option<&mut ProfilingSpan> {
        let span_id = *self.call_stack.last()?;
        self.spans.get_mut(span_id)
    }
    
    /// Record a metric for the current span
    pub fn record_metric(&mut self, name: &str, value: MetricValue) -> Result<(), String> {
        let span = self.current_span_mut()
            .ok_or_else(|| "No active span".to_string())?;
        
        span.add_metric(name.to_string(), value);
        
        Ok(())
    }
    
    /// Add a global metric
    pub fn add_global_metric(&mut self, name: String, value: MetricValue) {
        self.global_metrics.insert(name, value);
    }
    
    /// Get all global metrics
    pub fn global_metrics(&self) -> &HashMap<String, MetricValue> {
        &self.global_metrics
    }
    
    /// Get a global metric by name
    pub fn get_global_metric(&self, name: &str) -> Option<&MetricValue> {
        self.global_metrics.get(name)
    }
    
    /// Get the call stack
    pub fn call_stack(&self) -> &[usize] {
        &self.call_stack
    }
    
    /// Get the call stack as spans
    pub fn call_stack_spans(&self) -> Vec<&ProfilingSpan> {
        self.call_stack.iter()
            .filter_map(|&span_id| self.spans.get(span_id))
            .collect()
    }
    
    /// Build a call tree from the spans
    pub fn build_call_tree(&self) -> Vec<(usize, Vec<usize>)> {
        let mut tree = Vec::new();
        
        // Build a map of parent -> children
        let mut children_map: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
        
        for (span_id, span) in self.spans.iter().enumerate() {
            let parent_id = span.parent_id();
            children_map.entry(parent_id).or_default().push(span_id);
        }
        
        // Start with root spans (no parent)
        let root_spans = children_map.get(&None).cloned().unwrap_or_default();
        
        // Build the tree recursively
        for &root_span_id in &root_spans {
            self.build_call_tree_recursive(root_span_id, &children_map, &mut tree);
        }
        
        tree
    }
    
    /// Recursively build a call tree
    fn build_call_tree_recursive(
        &self,
        span_id: usize,
        children_map: &HashMap<Option<usize>, Vec<usize>>,
        tree: &mut Vec<(usize, Vec<usize>)>
    ) {
        let children = children_map.get(&Some(span_id)).cloned().unwrap_or_default();
        tree.push((span_id, children.clone()));
        
        for &child_id in &children {
            self.build_call_tree_recursive(child_id, children_map, tree);
        }
    }
    
    /// Get the top spans by duration
    pub fn top_spans_by_duration(&self, limit: usize) -> Vec<&ProfilingSpan> {
        let mut spans: Vec<&ProfilingSpan> = self.spans.iter().collect();
        
        // Sort by duration (descending)
        spans.sort_by(|a, b| {
            let a_duration = a.duration().unwrap_or_default();
            let b_duration = b.duration().unwrap_or_default();
            b_duration.cmp(&a_duration)
        });
        
        // Take the top N spans
        spans.into_iter().take(limit).collect()
    }
    
    /// Get the total time spent in each span type
    pub fn time_by_span_type(&self) -> HashMap<super::metrics::SpanType, std::time::Duration> {
        let mut result = HashMap::new();
        
        for span in &self.spans {
            if let Some(duration) = span.duration() {
                let span_type = span.span_type();
                let total = result.entry(span_type).or_insert_with(std::time::Duration::default);
                *total += duration;
            }
        }
        
        result
    }
}
