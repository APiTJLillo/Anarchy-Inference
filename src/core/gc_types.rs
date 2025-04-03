// src/core/gc_types.rs
// Core types for the garbage collection system

use std::collections::HashSet;
use crate::garbage_collection::managed::GcValueImpl;

/// Statistics for garbage collection
#[derive(Debug, Default, Clone)]
pub struct GcStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub cycles_detected: usize,
    pub total_memory: usize,
    pub collections_performed: usize,
    pub peak_memory: usize,
    pub last_collection_time_ms: u64,
}

/// Trait for garbage collector implementations
pub trait GarbageCollector: Send + Sync {
    /// Get statistics about the garbage collector
    fn get_stats(&self) -> GcStats;
    
    /// Perform garbage collection
    fn collect(&self);
    
    /// Get a value from the garbage collector by ID
    fn get_value(&self, id: usize) -> Option<GcValueImpl>;
    
    /// Update references for an object
    fn update_references(&self, id: usize, references: HashSet<usize>);
    
    /// Decrement reference count for an object
    fn decrement_ref_count(&self, id: usize);
}

/// Trait to add GC capabilities to the interpreter
pub trait GarbageCollected {
    /// Initialize the garbage collector
    fn init_garbage_collector(&mut self);
    
    /// Perform garbage collection
    fn collect_garbage(&mut self);
    
    /// Get statistics about the garbage collector
    fn get_gc_stats(&self) -> GcStats;
}
