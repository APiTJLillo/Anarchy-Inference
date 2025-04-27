// src/profiling/report.rs - Report generation for the Performance Profiling system

use std::collections::HashMap;
use std::fmt::Write;
use std::time::Duration;

use super::metrics::{MetricValue, SpanType};
use super::session::ProfilingSession;

/// Format for profiling reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReportFormat {
    /// Human-readable text format
    Text,
    /// Machine-readable JSON format
    Json,
    /// CSV format for data analysis
    Csv,
}

/// Trait for report generators
pub trait ReportGenerator: std::fmt::Debug {
    /// Generate a report from a profiling session
    fn generate_report(&self, session: &ProfilingSession) -> Result<String, String>;
    
    /// Get the format of this report generator
    fn format(&self) -> ReportFormat;
}

/// Text report generator
#[derive(Debug)]
pub struct TextReportGenerator {
    /// Include source locations in the report
    include_source_locations: bool,
    /// Include system information in the report
    include_system_info: bool,
    /// Maximum depth for call stack in the report
    max_call_stack_depth: usize,
}

impl TextReportGenerator {
    /// Create a new text report generator
    pub fn new() -> Self {
        Self {
            include_source_locations: true,
            include_system_info: true,
            max_call_stack_depth: 10,
        }
    }
    
    /// Create a new text report generator with custom settings
    pub fn with_settings(
        include_source_locations: bool,
        include_system_info: bool,
        max_call_stack_depth: usize,
    ) -> Self {
        Self {
            include_source_locations,
            include_system_info,
            max_call_stack_depth,
        }
    }
}

impl ReportGenerator for TextReportGenerator {
    fn generate_report(&self, session: &ProfilingSession) -> Result<String, String> {
        let mut output = String::new();
        
        // Header
        writeln!(output, "=== Anarchy Inference Profiling Report ===").map_err(|e| e.to_string())?;
        writeln!(output, "Session: {}", session.name()).map_err(|e| e.to_string())?;
        writeln!(output, "Duration: {:.3}s", session.duration().as_secs_f64()).map_err(|e| e.to_string())?;
        writeln!(output).map_err(|e| e.to_string())?;
        
        // System information
        if self.include_system_info {
            writeln!(output, "System Information:").map_err(|e| e.to_string())?;
            // Add system information here
            writeln!(output).map_err(|e| e.to_string())?;
        }
        
        // Top functions by execution time
        writeln!(output, "Top Functions by Execution Time:").map_err(|e| e.to_string())?;
        let top_spans = session.top_spans_by_duration(10);
        let total_duration = session.duration();
        
        for (i, span) in top_spans.iter().enumerate() {
            if let Some(duration) = span.duration() {
                let percentage = duration.as_secs_f64() / total_duration.as_secs_f64() * 100.0;
                writeln!(
                    output,
                    "{}. {}: {:.3}s ({:.1}%)",
                    i + 1,
                    span.name(),
                    duration.as_secs_f64(),
                    percentage
                ).map_err(|e| e.to_string())?;
                
                if self.include_source_locations {
                    if let Some(location) = span.source_location() {
                        writeln!(output, "   Location: {}", location.to_string()).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        writeln!(output).map_err(|e| e.to_string())?;
        
        // Memory usage
        writeln!(output, "Memory Usage:").map_err(|e| e.to_string())?;
        if let Some(total_allocations) = session.get_global_metric("total_allocations") {
            writeln!(output, "- Total Allocations: {}", total_allocations).map_err(|e| e.to_string())?;
        }
        if let Some(total_allocation_bytes) = session.get_global_metric("total_allocation_bytes") {
            writeln!(output, "- Total Allocation Size: {}", total_allocation_bytes).map_err(|e| e.to_string())?;
        }
        if let Some(peak_memory) = session.get_global_metric("peak_memory") {
            writeln!(output, "- Peak Memory Usage: {}", peak_memory).map_err(|e| e.to_string())?;
        }
        if let Some(gc_collections) = session.get_global_metric("gc_collections") {
            writeln!(output, "- GC Collections: {}", gc_collections).map_err(|e| e.to_string())?;
        }
        if let Some(gc_reclaimed) = session.get_global_metric("gc_reclaimed_bytes") {
            writeln!(output, "- GC Reclaimed: {}", gc_reclaimed).map_err(|e| e.to_string())?;
        }
        writeln!(output).map_err(|e| e.to_string())?;
        
        // Operation counts
        writeln!(output, "Operation Counts:").map_err(|e| e.to_string())?;
        for (name, value) in session.global_metrics() {
            if name.starts_with("op_count_") {
                let op_name = name.strip_prefix("op_count_").unwrap_or(name);
                writeln!(output, "- {}: {}", op_name, value).map_err(|e| e.to_string())?;
            }
        }
        writeln!(output).map_err(|e| e.to_string())?;
        
        // Call graph
        writeln!(output, "Call Graph:").map_err(|e| e.to_string())?;
        let call_tree = session.build_call_tree();
        
        // Find root spans (those with no parent)
        let root_spans: Vec<_> = call_tree.iter()
            .filter(|(span_id, _)| {
                if let Some(span) = session.get_span(*span_id) {
                    span.parent_id().is_none()
                } else {
                    false
                }
            })
            .collect();
        
        // Print the call tree
        for (span_id, _) in root_spans {
            self.print_call_tree(
                session,
                *span_id,
                &call_tree,
                &mut output,
                0,
                self.max_call_stack_depth,
            )?;
        }
        
        Ok(output)
    }
    
    fn format(&self) -> ReportFormat {
        ReportFormat::Text
    }
}

impl TextReportGenerator {
    /// Print a call tree recursively
    fn print_call_tree(
        &self,
        session: &ProfilingSession,
        span_id: usize,
        call_tree: &[(usize, Vec<usize>)],
        output: &mut String,
        depth: usize,
        max_depth: usize,
    ) -> Result<(), String> {
        if depth > max_depth {
            return Ok(());
        }
        
        // Get the span
        let span = match session.get_span(span_id) {
            Some(span) => span,
            None => return Ok(()),
        };
        
        // Get the duration
        let duration_str = match span.duration() {
            Some(duration) => format!(" ({:.3}s)", duration.as_secs_f64()),
            None => String::new(),
        };
        
        // Print the span
        let indent = "  ".repeat(depth);
        let prefix = if depth == 0 {
            ""
        } else if depth > 0 {
            "├─ "
        } else {
            "└─ "
        };
        
        writeln!(output, "{}{}{}{}", indent, prefix, span.name(), duration_str).map_err(|e| e.to_string())?;
        
        // Print source location if enabled
        if self.include_source_locations {
            if let Some(location) = span.source_location() {
                writeln!(output, "{}   Location: {}", indent, location.to_string()).map_err(|e| e.to_string())?;
            }
        }
        
        // Find children of this span
        let children = call_tree.iter()
            .find(|(id, _)| *id == span_id)
            .map(|(_, children)| children)
            .unwrap_or(&vec![]);
        
        // Print children
        for (i, &child_id) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            let child_prefix = if is_last { "└─ " } else { "├─ " };
            
            // Get the child span
            let child_span = match session.get_span(child_id) {
                Some(span) => span,
                None => continue,
            };
            
            // Get the child duration
            let child_duration_str = match child_span.duration() {
                Some(duration) => format!(" ({:.3}s)", duration.as_secs_f64()),
                None => String::new(),
            };
            
            // Print the child span
            let child_indent = indent.clone() + if is_last { "   " } else { "│  " };
            writeln!(output, "{}{}{}{}", child_indent, child_prefix, child_span.name(), child_duration_str)
                .map_err(|e| e.to_string())?;
            
            // Print child source location if enabled
            if self.include_source_locations {
                if let Some(location) = child_span.source_location() {
                    writeln!(output, "{}   Location: {}", child_indent, location.to_string())
                        .map_err(|e| e.to_string())?;
                }
            }
            
            // Recursively print grandchildren
            self.print_call_tree(
                session,
                child_id,
                call_tree,
                output,
                depth + 1,
                max_depth,
            )?;
        }
        
        Ok(())
    }
}

/// JSON report generator
#[derive(Debug)]
pub struct JsonReportGenerator {
    /// Include source locations in the report
    include_source_locations: bool,
    /// Include system information in the report
    include_system_info: bool,
    /// Maximum depth for call stack in the report
    max_call_stack_depth: usize,
}

impl JsonReportGenerator {
    /// Create a new JSON report generator
    pub fn new() -> Self {
        Self {
            include_source_locations: true,
            include_system_info: true,
            max_call_stack_depth: 10,
        }
    }
    
    /// Create a new JSON report generator with custom settings
    pub fn with_settings(
        include_source_locations: bool,
        include_system_info: bool,
        max_call_stack_depth: usize,
    ) -> Self {
        Self {
            include_source_locations,
            include_system_info,
            max_call_stack_depth,
        }
    }
}

impl ReportGenerator for JsonReportGenerator {
    fn generate_report(&self, session: &ProfilingSession) -> Result<String, String> {
        let mut output = String::new();
        
        // Start JSON object
        writeln!(output, "{{").map_err(|e| e.to_string())?;
        
        // Session information
        writeln!(output, "  \"session\": {{").map_err(|e| e.to_string())?;
        writeln!(output, "    \"name\": \"{}\",", session.name()).map_err(|e| e.to_string())?;
        writeln!(output, "    \"duration_ms\": {},", session.duration().as_millis()).map_err(|e| e.to_string())?;
        writeln!(output, "    \"start_time\": \"{:?}\",", session.start_time()).map_err(|e| e.to_string())?;
        if let Some(end_time) = session.end_time() {
            writeln!(output, "    \"end_time\": \"{:?}\"", end_time).map_err(|e| e.to_string())?;
        } else {
            writeln!(output, "    \"end_time\": null").map_err(|e| e.to_string())?;
        }
        writeln!(output, "  }},").map_err(|e| e.to_string())?;
        
        // System information
        if self.include_system_info {
            writeln!(output, "  \"system_info\": {{").map_err(|e| e.to_string())?;
            // Add system information here
            writeln!(output, "  }},").map_err(|e| e.to_string())?;
        }
        
        // Time metrics
        writeln!(output, "  \"time_metrics\": {{").map_err(|e| e.to_string())?;
        
        // Top functions by execution time
        writeln!(output, "    \"functions\": [").map_err(|e| e.to_string())?;
        let top_spans = session.top_spans_by_duration(10);
        let total_duration = session.duration();
        
        for (i, span) in top_spans.iter().enumerate() {
            if let Some(duration) = span.duration() {
                let percentage = duration.as_secs_f64() / total_duration.as_secs_f64() * 100.0;
                
                writeln!(output, "      {{").map_err(|e| e.to_string())?;
                writeln!(output, "        \"name\": \"{}\",", span.name()).map_err(|e| e.to_string())?;
                writeln!(output, "        \"duration_ms\": {},", duration.as_millis()).map_err(|e| e.to_string())?;
                writeln!(output, "        \"percentage\": {:.1}", percentage).map_err(|e| e.to_string())?;
                
                if self.include_source_locations {
                    if let Some(location) = span.source_location() {
                        writeln!(output, ",").map_err(|e| e.to_string())?;
                        writeln!(output, "        \"location\": {{").map_err(|e| e.to_string())?;
                        writeln!(output, "          \"file\": \"{}\",", location.file).map_err(|e| e.to_string())?;
                        writeln!(output, "          \"line\": {},", location.line).map_err(|e| e.to_string())?;
                        writeln!(output, "          \"column\": {}", location.column).map_err(|e| e.to_string())?;
                        writeln!(output, "        }}").map_err(|e| e.to_string())?;
                    }
                }
                
                if i < top_spans.len() - 1 {
                    writeln!(output, "      }},").map_err(|e| e.to_string())?;
                } else {
                    writeln!(output, "      }}").map_err(|e| e.to_string())?;
                }
            }
        }
        
        writeln!(output, "    ]").map_err(|e| e.to_string())?;
        writeln!(output, "  }},").map_err(|e| e.to_string())?;
        
        // Memory metrics
        writeln!(output, "  \"memory_metrics\": {{").map_err(|e| e.to_string())?;
        
        let mut first = true;
        for (name, value) in session.global_metrics() {
            if name.starts_with("mem_") {
                if !first {
                    writeln!(output, ",").map_err(|e| e.to_string())?;
                }
                first = false;
                
                let metric_name = name.strip_prefix("mem_").unwrap_or(name);
                match value {
                    MetricValue::Memory(bytes) => {
                        write!(output, "    \"{}\": {}", metric_name, bytes).map_err(|e| e.to_string())?;
                    },
                    MetricValue::Count(count) => {
                        write!(output, "    \"{}\": {}", metric_name, count).map_err(|e| e.to_string())?;
                    },
                    _ => {
                        write!(output, "    \"{}\": \"{}\"", metric_name, value).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        
        writeln!(output).map_err(|e| e.to_string())?;
        writeln!(output, "  }},").map_err(|e| e.to_string())?;
        
        // Operation metrics
        writeln!(output, "  \"operation_metrics\": {{").map_err(|e| e.to_string())?;
        
        let mut first = true;
        for (name, value) in session.global_metrics() {
            if name.starts_with("op_count_") {
                if !first {
                    writeln!(output, ",").map_err(|e| e.to_string())?;
                }
                first = false;
                
                let op_name = name.strip_prefix("op_count_").unwrap_or(name);
                match value {
                    MetricValue::Count(count) => {
                        write!(output, "    \"{}\": {}", op_name, count).map_err(|e| e.to_string())?;
                    },
                    _ => {
                        write!(output, "    \"{}\": \"{}\"", op_name, value).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        
        writeln!(output).map_err(|e| e.to_string())?;
        writeln!(output, "  }},").map_err(|e| e.to_string())?;
        
        // Call graph
        writeln!(output, "  \"call_graph\": {{").map_err(|e| e.to_string())?;
        
        // Find root spans (those with no parent)
        let root_spans: Vec<_> = session.spans().iter()
            .enumerate()
            .filter(|(_, span)| span.parent_id().is_none())
            .collect();
        
        if let Some((root_id, root_span)) = root_spans.first() {
            self.generate_call_graph_json(
                session,
                *root_id,
                &mut output,
                0,
                self.max_call_stack_depth,
                false,
            )?;
        }
        
        writeln!(output, "  }}").map_err(|e| e.to_string())?;
        
        // End JSON object
        writeln!(output, "}}").map_err(|e| e.to_string())?;
        
        Ok(output)
    }
    
    fn format(&self) -> ReportFormat {
        ReportFormat::Json
    }
}

impl JsonReportGenerator {
    /// Generate a JSON call graph recursively
    fn generate_call_graph_json(
        &self,
        session: &ProfilingSession,
        span_id: usize,
        output: &mut String,
        depth: usize,
        max_depth: usize,
        is_array_item: bool,
    ) -> Result<(), String> {
        if depth > max_depth {
            return Ok(());
        }
        
        // Get the span
        let span = match session.get_span(span_id) {
            Some(span) => span,
            None => return Ok(()),
        };
        
        // Start span object
        if is_array_item {
            writeln!(output, "    {}", "  ".repeat(depth) + "{").map_err(|e| e.to_string())?;
        } else {
            writeln!(output, "    {}", "  ".repeat(depth) + "\"name\": \"" + span.name() + "\",").map_err(|e| e.to_string())?;
            
            if let Some(duration) = span.duration() {
                writeln!(output, "    {}", "  ".repeat(depth) + "\"duration_ms\": " + &duration.as_millis().to_string() + ",").map_err(|e| e.to_string())?;
            }
            
            if self.include_source_locations {
                if let Some(location) = span.source_location() {
                    writeln!(output, "    {}", "  ".repeat(depth) + "\"location\": {").map_err(|e| e.to_string())?;
                    writeln!(output, "    {}", "  ".repeat(depth + 1) + "\"file\": \"" + &location.file + "\",").map_err(|e| e.to_string())?;
                    writeln!(output, "    {}", "  ".repeat(depth + 1) + "\"line\": " + &location.line.to_string() + ",").map_err(|e| e.to_string())?;
                    writeln!(output, "    {}", "  ".repeat(depth + 1) + "\"column\": " + &location.column.to_string()).map_err(|e| e.to_string())?;
                    writeln!(output, "    {}", "  ".repeat(depth) + "},").map_err(|e| e.to_string())?;
                }
            }
        }
        
        // Find children of this span
        let children: Vec<_> = session.spans().iter()
            .enumerate()
            .filter(|(_, child)| child.parent_id() == Some(span_id))
            .collect();
        
        if !children.is_empty() {
            writeln!(output, "    {}", "  ".repeat(depth) + "\"children\": [").map_err(|e| e.to_string())?;
            
            for (i, (child_id, _)) in children.iter().enumerate() {
                self.generate_call_graph_json(
                    session,
                    *child_id,
                    output,
                    depth + 1,
                    max_depth,
                    true,
                )?;
                
                if i < children.len() - 1 {
                    writeln!(output, "    {}", "  ".repeat(depth + 1) + "},").map_err(|e| e.to_string())?;
                } else {
                    writeln!(output, "    {}", "  ".repeat(depth + 1) + "}").map_err(|e| e.to_string())?;
                }
            }
            
            writeln!(output, "    {}", "  ".repeat(depth) + "]").map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }
}
