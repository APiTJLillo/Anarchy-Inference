// src/gc.rs - Garbage Collection module for Anarchy-Inference
// Implements reference counting and memory management for the language

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::core::gc_types::{GcStats, GarbageCollector as GcTrait};
use crate::gc::managed::GcValueImpl;
use crate::core::value::GcValue;

/// GarbageCollector manages memory and tracks object references
#[derive(Debug)]
pub struct GarbageCollector {
    // Track all allocated objects with their reference counts
    objects: Mutex<HashMap<usize, GcObject>>,
    // Track objects that might form reference cycles
    potential_cycles: Mutex<HashSet<usize>>,
    // Statistics for memory management
    stats: Mutex<GcStats>,
    // Threshold for automatic collection
    collection_threshold: Mutex<usize>,
    // Flag to enable/disable automatic collection
    auto_collect_enabled: Mutex<bool>,
    // Generational collection settings
    generation_threshold: Mutex<usize>,
    // Incremental collection settings
    incremental_step_size: Mutex<usize>,
}

/// Object tracked by the garbage collector
#[derive(Debug, Clone)]
struct GcObject {
    // Unique identifier for the object
    id: usize,
    // The actual value being tracked
    value: GcValueImpl,
    // References to other objects this object holds
    references: HashSet<usize>,
    // Reference count (how many other objects point to this one)
    ref_count: usize,
    // Mark for cycle detection
    marked: bool,
    // Size of the object in bytes (approximate)
    size: usize,
    // Generation of the object (for generational collection)
    generation: usize,
    // Creation time for age-based collection
    creation_time: Instant,
}

impl GarbageCollector {
    /// Create a new garbage collector
    pub fn new() -> Self {
        GarbageCollector {
            objects: Mutex::new(HashMap::new()),
            potential_cycles: Mutex::new(HashSet::new()),
            stats: Mutex::new(GcStats::default()),
            collection_threshold: Mutex::new(1024 * 1024), // 1MB default threshold
            auto_collect_enabled: Mutex::new(true),
            generation_threshold: Mutex::new(3),           // Default to 3 generations
            incremental_step_size: Mutex::new(100),        // Process 100 objects per incremental step
        }
    }

    /// Create a new garbage collector with custom settings
    pub fn with_settings(threshold: usize, auto_collect: bool, 
                         generation_threshold: usize, incremental_step_size: usize) -> Self {
        GarbageCollector {
            objects: Mutex::new(HashMap::new()),
            potential_cycles: Mutex::new(HashSet::new()),
            stats: Mutex::new(GcStats::default()),
            collection_threshold: Mutex::new(threshold),
            auto_collect_enabled: Mutex::new(auto_collect),
            generation_threshold: Mutex::new(generation_threshold),
            incremental_step_size: Mutex::new(incremental_step_size),
        }
    }

    /// Set the collection threshold
    pub fn set_collection_threshold(&self, threshold: usize) {
        let mut collection_threshold = self.collection_threshold.lock().unwrap();
        *collection_threshold = threshold;
    }

    /// Enable or disable automatic collection
    pub fn set_auto_collect(&self, enabled: bool) {
        let mut auto_collect_enabled = self.auto_collect_enabled.lock().unwrap();
        *auto_collect_enabled = enabled;
    }

    /// Set the generation threshold
    pub fn set_generation_threshold(&self, threshold: usize) {
        let mut generation_threshold = self.generation_threshold.lock().unwrap();
        *generation_threshold = threshold;
    }

    /// Set the incremental step size
    pub fn set_incremental_step_size(&self, step_size: usize) {
        let mut incremental_step_size = self.incremental_step_size.lock().unwrap();
        *incremental_step_size = step_size;
    }

    /// Allocate a new value in the garbage collector
    pub fn allocate(&self, value: GcValueImpl) -> GcValue {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Generate a unique ID for this object
        let id = stats.allocations + stats.deallocations + 1;
        
        // Calculate approximate size of the object
        let size = self.calculate_object_size(&value);
        
        // Create the GC object
        let gc_object = GcObject {
            id,
            value: value.clone(),
            references: HashSet::new(),
            ref_count: 1, // Initial reference count is 1
            marked: false,
            size,
            generation: 0, // Start in the youngest generation
            creation_time: Instant::now(),
        };
        
        // Update statistics
        stats.allocations += 1;
        stats.total_memory += size;
        
        // Update peak memory if needed
        if stats.total_memory > stats.peak_memory {
            stats.peak_memory = stats.total_memory;
        }
        
        // Store the object
        objects.insert(id, gc_object);
        
        // Check if this object might participate in cycles
        if Self::might_form_cycle(&value) {
            let mut potential_cycles = self.potential_cycles.lock().unwrap();
            potential_cycles.insert(id);
        }
        
        // Check if we should perform automatic collection
        drop(objects); // Release lock before potential collection
        drop(stats);   // Release lock before potential collection
        self.check_auto_collect();
        
        // Create and return the GcValue
        GcValue {
            id,
            gc: Arc::new(self.clone()),
        }
    }
    
    /// Calculate the approximate size of an object in bytes
    fn calculate_object_size(&self, value: &GcValueImpl) -> usize {
        match value {
            GcValueImpl::Object(map) => {
                // Base size + size of each key-value pair
                std::mem::size_of::<GcValueImpl>() + 
                map.len() * (std::mem::size_of::<String>() + std::mem::size_of::<crate::core::value::Value>())
            },
            GcValueImpl::Array(items) => {
                // Base size + size of each element
                std::mem::size_of::<GcValueImpl>() + 
                items.len() * std::mem::size_of::<crate::core::value::Value>()
            },
            GcValueImpl::Function { name, parameters, body, closure } => {
                // Functions are more complex, use a reasonable estimate
                std::mem::size_of::<GcValueImpl>() + 
                name.len() + 
                parameters.len() * std::mem::size_of::<String>() + 
                256 // Estimate for body and closure
            },
        }
    }
    
    /// Check if automatic collection should be performed
    fn check_auto_collect(&self) {
        let auto_collect_enabled = self.auto_collect_enabled.lock().unwrap();
        if !*auto_collect_enabled {
            return;
        }
        
        let stats = self.stats.lock().unwrap();
        let threshold = self.collection_threshold.lock().unwrap();
        
        if stats.total_memory > *threshold {
            // Drop locks before collection to avoid deadlock
            drop(stats);
            drop(threshold);
            drop(auto_collect_enabled);
            
            // Perform collection
            self.collect();
        }
    }
    
    /// Check if a value might form reference cycles
    fn might_form_cycle(value: &GcValueImpl) -> bool {
        match value {
            GcValueImpl::Object(_) | GcValueImpl::Array(_) | GcValueImpl::Function { .. } => true,
            // Add other complex types that might form cycles
        }
    }
    
    /// Perform generational garbage collection
    fn collect_generational(&self, max_generation: usize) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Find objects in the specified generations with zero reference count
        let to_remove: Vec<usize> = objects.iter()
            .filter(|(_, obj)| obj.ref_count == 0 && obj.generation <= max_generation)
            .map(|(id, _)| *id)
            .collect();
        
        // Remove them
        for id in to_remove {
            if let Some(obj) = objects.remove(&id) {
                stats.deallocations += 1;
                stats.total_memory -= obj.size;
                
                // Also remove from potential cycles
                let mut potential_cycles = self.potential_cycles.lock().unwrap();
                potential_cycles.remove(&id);
            }
        }
        
        // Promote surviving objects to the next generation
        for (_, obj) in objects.iter_mut() {
            if obj.generation <= max_generation {
                obj.generation += 1;
            }
        }
    }
    
    /// Perform incremental garbage collection
    fn collect_incremental(&self, step_size: usize) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Get a subset of objects to process in this step
        let to_process: Vec<usize> = objects.iter()
            .filter(|(_, obj)| obj.ref_count == 0)
            .map(|(id, _)| *id)
            .take(step_size)
            .collect();
        
        // Remove them
        for id in to_process {
            if let Some(obj) = objects.remove(&id) {
                stats.deallocations += 1;
                stats.total_memory -= obj.size;
                
                // Also remove from potential cycles
                let mut potential_cycles = self.potential_cycles.lock().unwrap();
                potential_cycles.remove(&id);
            }
        }
    }
    
    /// Mark all reachable objects
    fn mark_reachable_objects(&self) {
        let mut objects = self.objects.lock().unwrap();
        
        // Reset all marks
        for (_, obj) in objects.iter_mut() {
            obj.marked = false;
        }
        
        // Start marking from all root objects (ref_count > 0)
        let roots: Vec<usize> = objects.iter()
            .filter(|(_, obj)| obj.ref_count > 0)
            .map(|(id, _)| *id)
            .collect();
        
        // Mark all objects reachable from roots
        for root in roots {
            self.mark_object(root, &mut objects);
        }
    }
    
    /// Mark an object and all objects reachable from it
    fn mark_object(&self, id: usize, objects: &mut HashMap<usize, GcObject>) {
        if let Some(obj) = objects.get_mut(&id) {
            if obj.marked {
                return; // Already marked
            }
            
            // Mark this object
            obj.marked = true;
            
            // Mark all referenced objects
            let references = obj.references.clone();
            for ref_id in references {
                self.mark_object(ref_id, objects);
            }
        }
    }
    
    /// Sweep all unmarked objects
    fn sweep_unmarked_objects(&self) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        let mut potential_cycles = self.potential_cycles.lock().unwrap();
        
        // Find unmarked objects
        let to_remove: Vec<usize> = objects.iter()
            .filter(|(_, obj)| !obj.marked)
            .map(|(id, _)| *id)
            .collect();
        
        // Remove them
        for id in to_remove {
            if let Some(obj) = objects.remove(&id) {
                stats.deallocations += 1;
                stats.cycles_detected += 1;
                stats.total_memory -= obj.size;
                
                // Also remove from potential cycles
                potential_cycles.remove(&id);
            }
        }
    }
    
    /// Increment reference count for an object
    pub fn increment_ref_count(&self, id: usize) {
        let mut objects = self.objects.lock().unwrap();
        if let Some(obj) = objects.get_mut(&id) {
            obj.ref_count += 1;
        }
    }
}

// Implement the GarbageCollector trait
impl GcTrait for GarbageCollector {
    fn get_stats(&self) -> GcStats {
        self.stats.lock().unwrap().clone()
    }
    
    fn collect(&self) {
        let start_time = Instant::now();
        
        // First, check for objects with zero reference count
        self.collect_unreferenced();
        
        // Then, detect and collect cycles
        self.collect_cycles();
        
        // Update statistics
        let mut stats = self.stats.lock().unwrap();
        stats.collections_performed += 1;
        stats.last_collection_time_ms = start_time.elapsed().as_millis() as u64;
    }
    
    fn get_value(&self, id: usize) -> Option<GcValueImpl> {
        let objects = self.objects.lock().unwrap();
        objects.get(&id).map(|obj| obj.value.clone())
    }
    
    fn update_references(&self, id: usize, references: HashSet<usize>) {
        let mut objects = self.objects.lock().unwrap();
        
        // First collect the old references and new references
        let old_refs = if let Some(obj) = objects.get(&id) {
            obj.references.clone()
        } else {
            HashSet::new()
        };
        
        // Now update the object's reference list
        if let Some(obj) = objects.get_mut(&id) {
            obj.references = references.clone();
        }
        
        // Drop the lock before processing references to avoid multiple mutable borrows
        drop(objects);
        
        // Now handle the reference counts separately
        // Remove old references
        for old_ref in old_refs {
            if !references.contains(&old_ref) {
                self.decrement_ref_count(old_ref);
            }
        }
        
        // Add new references
        for new_ref in references {
            if !old_refs.contains(&new_ref) {
                self.increment_ref_count(new_ref);
            }
        }
    }
    
    fn decrement_ref_count(&self, id: usize) {
        let mut objects = self.objects.lock().unwrap();
        if let Some(obj) = objects.get_mut(&id) {
            obj.ref_count = obj.ref_count.saturating_sub(1);
        }
    }
}

// Additional methods not part of the trait
impl GarbageCollector {
    /// Collect objects with zero reference count
    fn collect_unreferenced(&self) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Find objects with zero reference count
        let to_remove: Vec<usize> = objects.iter()
            .filter(|(_, obj)| obj.ref_count == 0)
            .map(|(id, _)| *id)
            .collect();
        
        // Remove them
        for id in to_remove {
            if let Some(obj) = objects.remove(&id) {
                stats.deallocations += 1;
                stats.total_memory -= obj.size;
                
                // Also remove from potential cycles
                let mut potential_cycles = self.potential_cycles.lock().unwrap();
                potential_cycles.remove(&id);
            }
        }
    }
    
    /// Detect and collect reference cycles
    fn collect_cycles(&self) {
        let potential_cycles = self.potential_cycles.lock().unwrap();
        
        if potential_cycles.is_empty() {
            return;
        }
        
        // Mark phase
        self.mark_reachable_objects();
        
        // Sweep phase
        self.sweep_unmarked_objects();
    }
    
    /// Force a full garbage collection
    pub fn force_collect(&self) {
        self.collect();
    }
    
    /// Get the current memory usage
    pub fn memory_usage(&self) -> usize {
        let stats = self.stats.lock().unwrap();
        stats.total_memory
    }
    
    /// Get the current collection threshold
    pub fn get_collection_threshold(&self) -> usize {
        let threshold = self.collection_threshold.lock().unwrap();
        *threshold
    }
    
    /// Check if automatic collection is enabled
    pub fn is_auto_collect_enabled(&self) -> bool {
        let enabled = self.auto_collect_enabled.lock().unwrap();
        *enabled
    }
    
    /// Get the current generation threshold
    pub fn get_generation_threshold(&self) -> usize {
        let threshold = self.generation_threshold.lock().unwrap();
        *threshold
    }
    
    /// Get the current incremental step size
    pub fn get_incremental_step_size(&self) -> usize {
        let step_size = self.incremental_step_size.lock().unwrap();
        *step_size
    }
    
    /// Perform a generational collection
    pub fn collect_generation(&self, generation: usize) {
        self.collect_generational(generation);
    }
    
    /// Perform an incremental collection step
    pub fn collect_incremental_step(&self) {
        let step_size = self.get_incremental_step_size();
        self.collect_incremental(step_size);
    }
}

impl Clone for GarbageCollector {
    fn clone(&self) -> Self {
        // Create a new GC with the same state
        let objects = self.objects.lock().unwrap().clone();
        let potential_cycles = self.potential_cycles.lock().unwrap().clone();
        let stats = self.stats.lock().unwrap().clone();
        let threshold = self.collection_threshold.lock().unwrap().clone();
        let auto_collect = self.auto_collect_enabled.lock().unwrap().clone();
        let generation_threshold = self.generation_threshold.lock().unwrap().clone();
        let incremental_step_size = self.incremental_step_size.lock().unwrap().clone();
        
        let new_gc = GarbageCollector {
            objects: Mutex::new(objects),
            potential_cycles: Mutex::new(potential_cycles),
            stats: Mutex::new(stats),
            collection_threshold: Mutex::new(threshold),
            auto_collect_enabled: Mutex::new(auto_collect),
            generation_threshold: Mutex::new(generation_threshold),
            incremental_step_size: Mutex::new(incremental_step_size),
        };
        
        new_gc
    }
}

// Extension trait to add GC capabilities to the interpreter
pub trait GarbageCollected {
    fn init_garbage_collector(&mut self);
    fn collect_garbage(&mut self);
    fn allocate_value(&mut self, value: GcValueImpl) -> GcValue;
    fn get_gc_stats(&self) -> GcStats;
}
