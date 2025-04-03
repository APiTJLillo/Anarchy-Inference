// Update the collector.rs file to implement the GarbageCollector trait

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::core::gc_types::{GcStats, GarbageCollector as GcTrait};
use crate::core::value::GcValue;
use crate::gc::managed::GcValueImpl;

/// GarbageCollector manages memory and tracks object references
#[derive(Debug)]
pub struct GarbageCollector {
    // Track all allocated objects with their reference counts
    objects: Mutex<HashMap<usize, GcObject>>,
    // Track objects that might form reference cycles
    potential_cycles: Mutex<HashSet<usize>>,
    // Statistics for memory management
    stats: Mutex<GcStats>,
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
}

impl GarbageCollector {
    /// Create a new garbage collector
    pub fn new() -> Self {
        GarbageCollector {
            objects: Mutex::new(HashMap::new()),
            potential_cycles: Mutex::new(HashSet::new()),
            stats: Mutex::new(GcStats::default()),
        }
    }

    /// Allocate a new value in the garbage collector
    pub fn allocate(&self, value: GcValueImpl) -> GcValue {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Generate a unique ID for this object
        let id = stats.allocations + stats.deallocations + 1;
        
        // Create the GC object
        let gc_object = GcObject {
            id,
            value: value.clone(),
            references: HashSet::new(),
            ref_count: 1, // Initial reference count is 1
            marked: false,
        };
        
        // Update statistics
        stats.allocations += 1;
        stats.total_memory += std::mem::size_of::<GcObject>();
        
        // Store the object
        objects.insert(id, gc_object);
        
        // Check if this object might participate in cycles
        if Self::might_form_cycle(&value) {
            let mut potential_cycles = self.potential_cycles.lock().unwrap();
            potential_cycles.insert(id);
        }
        
        // Create and return the GcValue
        GcValue {
            id,
            gc: Arc::new(self.clone()),
        }
    }
    
    /// Check if a value might form reference cycles
    fn might_form_cycle(value: &GcValueImpl) -> bool {
        match value {
            GcValueImpl::Object(_) | GcValueImpl::Array(_) | GcValueImpl::Function { .. } => true,
            _ => false,
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
            if let Some(_obj) = objects.remove(&id) {
                stats.deallocations += 1;
                stats.cycles_detected += 1;
                stats.total_memory -= std::mem::size_of::<GcObject>();
                
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
        
        // Find objects with zero reference count
        let to_remove: Vec<usize> = objects.iter()
            .filter(|(_, obj)| obj.ref_count == 0)
            .map(|(id, _)| *id)
            .collect();
        
        // Remove them
        for id in to_remove {
            if let Some(_obj) = objects.remove(&id) {
                stats.deallocations += 1;
                stats.total_memory -= std::mem::size_of::<GcObject>();
                
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
}

impl Clone for GarbageCollector {
    fn clone(&self) -> Self {
        // Create a new GC with the same state
        let objects = self.objects.lock().unwrap().clone();
        let potential_cycles = self.potential_cycles.lock().unwrap().clone();
        let stats = self.stats.lock().unwrap().clone();
        
        let new_gc = GarbageCollector {
            objects: Mutex::new(objects),
            potential_cycles: Mutex::new(potential_cycles),
            stats: Mutex::new(stats),
        };
        
        new_gc
    }
}
