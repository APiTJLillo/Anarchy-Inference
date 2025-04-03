// Update the collector.rs file to implement the GarbageCollector trait

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::core::gc_types::{GcStats, GarbageCollector as GcTrait};
use crate::core::value::GcValue;
use crate::garbage_collection::managed::GcValueImpl;

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
}

// Custom Clone implementation for GarbageCollector since Mutex doesn't implement Clone
impl Clone for GarbageCollector {
    fn clone(&self) -> Self {
        // Create a new instance with empty collections
        let new_gc = GarbageCollector {
            objects: Mutex::new(HashMap::new()),
            potential_cycles: Mutex::new(HashSet::new()),
            stats: Mutex::new(self.stats.lock().unwrap().clone()),
            collection_threshold: Mutex::new(*self.collection_threshold.lock().unwrap()),
            auto_collect_enabled: Mutex::new(*self.auto_collect_enabled.lock().unwrap()),
        };
        
        // Clone the objects map - need to drop locks before returning
        {
            let objects = self.objects.lock().unwrap();
            let mut new_objects = new_gc.objects.lock().unwrap();
            for (id, obj) in objects.iter() {
                new_objects.insert(*id, obj.clone());
            }
            // new_objects is dropped here when it goes out of scope
        }
        
        // Clone the potential cycles set - need to drop locks before returning
        {
            let potential_cycles = self.potential_cycles.lock().unwrap();
            let mut new_potential_cycles = new_gc.potential_cycles.lock().unwrap();
            for id in potential_cycles.iter() {
                new_potential_cycles.insert(*id);
            }
            // new_potential_cycles is dropped here when it goes out of scope
        }
        
        new_gc
    }
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
        }
    }

    /// Create a new garbage collector with custom settings
    pub fn with_settings(threshold: usize, auto_collect: bool) -> Self {
        GarbageCollector {
            objects: Mutex::new(HashMap::new()),
            potential_cycles: Mutex::new(HashSet::new()),
            stats: Mutex::new(GcStats::default()),
            collection_threshold: Mutex::new(threshold),
            auto_collect_enabled: Mutex::new(auto_collect),
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
        };
        
        // Update statistics
        stats.allocations += 1;
        stats.total_memory += size;
        
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
            GcValueImpl::Function { .. } => {
                // Functions are more complex, use a reasonable estimate
                std::mem::size_of::<GcValueImpl>() + 256
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
        // First, check for objects with zero reference count
        self.collect_unreferenced();
        
        // Then, detect and collect cycles
        self.collect_cycles();
        
        // Update statistics
        let mut stats = self.stats.lock().unwrap();
        stats.collections_performed += 1;
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
            self.decrement_ref_count(old_ref);
        }
        
        // Add new references
        for new_ref in references {
            self.increment_ref_count(new_ref);
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
        let mut potential_cycles = self.potential_cycles.lock().unwrap();
        
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
                potential_cycles.remove(&id);
            }
        }
    }
    
    /// Detect and collect reference cycles
    fn collect_cycles(&self) {
        // Mark all reachable objects
        self.mark_reachable_objects();
        
        // Sweep all unmarked objects (these are in cycles)
        self.sweep_unmarked_objects();
    }
}
