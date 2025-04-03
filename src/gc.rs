// Garbage Collection module for Anarchy-Inference
// Implements reference counting and memory management for the language

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

// Forward declaration to avoid circular imports
pub type Value = crate::value::Value;

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

/// Statistics for garbage collection
#[derive(Debug, Default, Clone)]
pub struct GcStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub cycles_detected: usize,
    pub total_memory: usize,
    pub collections_performed: usize,
}

/// Object tracked by the garbage collector
#[derive(Debug, Clone)]
struct GcObject {
    // Unique identifier for the object
    id: usize,
    // The actual value being tracked
    value: Value,
    // References to other objects this object holds
    references: HashSet<usize>,
    // Reference count (how many other objects point to this one)
    ref_count: usize,
    // Mark for cycle detection
    marked: bool,
}

/// A garbage-collected value wrapper
#[derive(Debug, Clone)]
pub struct GcValue {
    // Pointer to the actual value in the GC
    ptr: Arc<Mutex<GcValueInner>>,
    // Back-reference to the garbage collector
    gc: Arc<GarbageCollector>,
}

/// Inner structure for GcValue
#[derive(Debug)]
struct GcValueInner {
    // The actual value
    value: Value,
    // Unique identifier in the GC
    id: usize,
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
    pub fn allocate(&self, value: Value) -> GcValue {
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
            ptr: Arc::new(Mutex::new(GcValueInner {
                value,
                id,
            })),
            gc: Arc::new(self.clone()),
        }
    }
    
    /// Check if a value might form reference cycles
    fn might_form_cycle(value: &Value) -> bool {
        match value {
            Value::Object(_) | Value::Array(_) => true,
            _ => false,
        }
    }
    
    /// Update references for an object
    pub fn update_references(&self, id: usize, references: HashSet<usize>) {
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
    
    /// Increment reference count for an object
    fn increment_ref_count(&self, id: usize) {
        let mut objects = self.objects.lock().unwrap();
        if let Some(obj) = objects.get_mut(&id) {
            obj.ref_count += 1;
        }
    }
    
    /// Decrement reference count for an object
    fn decrement_ref_count(&self, id: usize) {
        let mut objects = self.objects.lock().unwrap();
        if let Some(obj) = objects.get_mut(&id) {
            obj.ref_count = obj.ref_count.saturating_sub(1);
        }
    }
    
    /// Perform garbage collection
    pub fn collect(&self) {
        // First, check for objects with zero reference count
        self.collect_unreferenced();
        
        // Then, detect and collect cycles
        self.collect_cycles();
        
        // Update statistics
        let mut stats = self.stats.lock().unwrap();
        stats.collections_performed += 1;
    }
    
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
    
    /// Get statistics about the garbage collector
    pub fn get_stats(&self) -> GcStats {
        self.stats.lock().unwrap().clone()
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

impl GcValue {
    /// Get a reference to the inner value
    pub fn get(&self) -> Value {
        let inner = self.ptr.lock().unwrap();
        inner.value.clone()
    }
    
    /// Update the inner value
    pub fn set(&self, value: Value) {
        let mut inner = self.ptr.lock().unwrap();
        
        // Update the value
        inner.value = value.clone();
        
        // Update references in the GC
        let references = Self::extract_references(&value);
        self.gc.update_references(inner.id, references);
    }
    
    /// Extract references from a value
    fn extract_references(value: &Value) -> HashSet<usize> {
        let mut references = HashSet::new();
        
        match value {
            Value::Object(map) => {
                for (_, v) in map {
                    if let Some(id) = Self::get_gc_id(v) {
                        references.insert(id);
                    }
                    
                    // Also add references from nested values
                    let nested_refs = Self::extract_references(v);
                    references.extend(nested_refs);
                }
            },
            Value::Array(items) => {
                for item in items {
                    if let Some(id) = Self::get_gc_id(item) {
                        references.insert(id);
                    }
                    
                    // Also add references from nested values
                    let nested_refs = Self::extract_references(item);
                    references.extend(nested_refs);
                }
            },
            _ => {}, // Other value types don't contain references
        }
        
        references
    }
    
    /// Get the GC ID from a value if it's a GC-managed value
    fn get_gc_id(_value: &Value) -> Option<usize> {
        // This would need to be implemented based on how GC IDs are stored in values
        None
    }
}

impl Drop for GcValue {
    fn drop(&mut self) {
        // When a GcValue is dropped, decrement the reference count
        let inner = self.ptr.lock().unwrap();
        let id = inner.id;
        
        let mut objects = self.gc.objects.lock().unwrap();
        
        if let Some(obj) = objects.get_mut(&id) {
            obj.ref_count = obj.ref_count.saturating_sub(1);
            
            // If reference count is zero, it will be collected in the next GC cycle
        }
    }
}

// Extension trait to add GC capabilities to the interpreter
pub trait GarbageCollected {
    fn init_garbage_collector(&mut self);
    fn collect_garbage(&mut self);
    fn allocate_value(&mut self, value: Value) -> GcValue;
    fn get_gc_stats(&self) -> GcStats;
}
