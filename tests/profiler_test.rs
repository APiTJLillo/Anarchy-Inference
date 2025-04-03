#[cfg(test)]
mod profiler_tests {
    use std::time::Duration;
    use std::thread;
    use crate::core::profiler::Profiler;

    #[test]
    fn test_profiler_basic() {
        let profiler = Profiler::new();
        
        // Test enabling/disabling
        assert!(profiler.is_enabled());
        profiler.set_enabled(false);
        assert!(!profiler.is_enabled());
        profiler.set_enabled(true);
        assert!(profiler.is_enabled());
        
        // Test reset
        profiler.reset();
        
        // Test total elapsed
        let elapsed = profiler.total_elapsed();
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn test_profiler_spans() {
        let profiler = Profiler::new();
        
        // Start a span
        let guard = profiler.start_span("test_span", 1000);
        
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        
        // End the span
        drop(guard);
        
        // Get stats for the span
        let stats = profiler.get_span_stats("test_span");
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "test_span");
        assert!(stats[0].duration.as_nanos() > 0);
        
        // Test nested spans
        let outer_guard = profiler.start_span("outer_span", 2000);
        thread::sleep(Duration::from_millis(5));
        
        let inner_guard = profiler.start_span("inner_span", 2100);
        thread::sleep(Duration::from_millis(5));
        drop(inner_guard);
        
        thread::sleep(Duration::from_millis(5));
        drop(outer_guard);
        
        // Get stats for all spans
        let all_stats = profiler.get_stats();
        assert!(all_stats.contains_key("test_span"));
        assert!(all_stats.contains_key("outer_span"));
        assert!(all_stats.contains_key("inner_span"));
        
        // Test report generation
        let report = profiler.generate_report();
        assert!(report.contains("Performance Profiling Report"));
        assert!(report.contains("test_span"));
        assert!(report.contains("outer_span"));
        assert!(report.contains("inner_span"));
    }

    #[test]
    fn test_profiler_memory_tracking() {
        let profiler = Profiler::new();
        
        // Start a span with initial memory
        profiler.start_span("memory_test", 1000);
        
        // End the span with increased memory
        profiler.end_span("memory_test", 1500);
        
        // Get stats for the span
        let stats = profiler.get_span_stats("memory_test").unwrap();
        assert_eq!(stats[0].memory_delta, 500);
        
        // Start a span with initial memory
        profiler.start_span("memory_decrease", 2000);
        
        // End the span with decreased memory
        profiler.end_span("memory_decrease", 1800);
        
        // Get stats for the span
        let stats = profiler.get_span_stats("memory_decrease").unwrap();
        assert_eq!(stats[0].memory_delta, -200);
    }

    #[test]
    fn test_profiler_macro() {
        struct MemoryProvider {
            memory: usize,
        }
        
        impl MemoryProvider {
            fn current_memory(&self) -> usize {
                self.memory
            }
        }
        
        let profiler = Profiler::new();
        let memory_provider = MemoryProvider { memory: 1000 };
        
        // Use the macro to profile a block
        let result = profile_span!(profiler, "macro_test", memory_provider, {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        // Check the result
        assert_eq!(result, 42);
        
        // Get stats for the span
        let stats = profiler.get_span_stats("macro_test");
        assert!(stats.is_some());
    }
}
