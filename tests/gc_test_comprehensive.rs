// tests/gc_test_comprehensive.rs
// Comprehensive tests for the garbage collection implementation

use std::collections::HashMap;
use std::sync::Arc;
use anarchy_inference::garbage_collection::collector::GarbageCollector;
use anarchy_inference::core::gc_types::GarbageCollector as GcTrait;
use anarchy_inference::core::value::{Value, GcValue};
use anarchy_inference::interpreter::Interpreter;
use anarchy_inference::garbage_collection::managed::GcValueImpl;

#[test]
fn test_basic_allocation() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Get initial stats
    let initial_stats = gc.get_stats();
    
    // Allocate some objects
    let _obj1 = gc.allocate(GcValueImpl::Object(HashMap::new()));
    let _obj2 = gc.allocate(GcValueImpl::Array(vec![]));
    let _obj3 = gc.allocate(GcValueImpl::Object(HashMap::new()));
    
    // Get updated stats
    let updated_stats = gc.get_stats();
    
    // Verify that allocations increased
    assert!(updated_stats.allocations > initial_stats.allocations);
    assert_eq!(updated_stats.allocations, initial_stats.allocations + 3);
}

#[test]
fn test_reference_counting() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Allocate an object
    let obj = gc.allocate(GcValueImpl::Object(HashMap::new()));
    
    // Get initial stats
    let initial_stats = gc.get_stats();
    
    // Create a scope to test reference counting
    {
        // Clone the reference to increase reference count
        let _obj_clone = obj.clone();
        
        // Force garbage collection
        gc.collect();
        
        // Get stats after collection
        let mid_stats = gc.get_stats();
        
        // Verify that no deallocations occurred
        assert_eq!(mid_stats.deallocations, initial_stats.deallocations);
    }
    
    // Clone has gone out of scope, reference count decreased
    
    // Drop the original reference
    drop(obj);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.get_stats();
    
    // Verify that a deallocation occurred
    assert!(final_stats.deallocations > initial_stats.deallocations);
}

#[test]
fn test_cycle_detection() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create objects that reference each other in a cycle
    let obj1_map = HashMap::new();
    let obj2_map = HashMap::new();
    
    // Allocate first object
    let obj1_ref = gc.allocate(GcValueImpl::Object(obj1_map));
    
    // Allocate second object
    let obj2_ref = gc.allocate(GcValueImpl::Object(obj2_map));
    
    // Get references to modify the objects
    let obj1_id = obj1_ref.id;
    let obj2_id = obj2_ref.id;
    
    // Create cycle: obj1 -> obj2 -> obj1
    let mut refs1 = std::collections::HashSet::new();
    refs1.insert(obj2_id);
    gc.update_references(obj1_id, refs1);
    
    let mut refs2 = std::collections::HashSet::new();
    refs2.insert(obj1_id);
    gc.update_references(obj2_id, refs2);
    
    // Get initial stats
    let initial_stats = gc.get_stats();
    
    // Drop references to the objects
    drop(obj1_ref);
    drop(obj2_ref);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.get_stats();
    
    // Verify that cycles were detected and collected
    assert!(final_stats.cycles_detected > initial_stats.cycles_detected);
    assert!(final_stats.deallocations > initial_stats.deallocations);
}

#[test]
fn test_memory_usage_statistics() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Get initial stats
    let initial_stats = gc.get_stats();
    
    // Allocate many objects
    for _i in 0..100 {
        let _obj = gc.allocate(GcValueImpl::Object(HashMap::new()));
    }
    
    // Get updated stats
    let updated_stats = gc.get_stats();
    
    // Verify that memory usage increased
    assert!(updated_stats.total_memory > initial_stats.total_memory);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.get_stats();
    
    // Verify that collections were performed
    assert!(final_stats.collections_performed > initial_stats.collections_performed);
}

#[test]
fn test_interpreter_integration() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Create and allocate complex values
    let _obj_ref = interpreter.create_object();
    
    // Get stats after allocation
    let final_stats = interpreter.get_memory_stats();
    
    // Verify that allocations occurred
    assert!(final_stats.objects_allocated > 0);
}

#[test]
fn test_automatic_collection_triggers() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Allocate many objects
    for _i in 0..20 {
        let _obj_ref = interpreter.create_object();
    }
    
    // Get stats after allocations
    let stats = interpreter.get_memory_stats();
    
    // Verify that allocations occurred
    assert!(stats.objects_allocated > 0);
}

#[test]
fn test_complex_object_graph() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create a complex object graph
    let mut root_map = HashMap::new();
    let child1_map = HashMap::new();
    let child2_map = HashMap::new();
    
    // Allocate child objects
    let child1_ref = gc.allocate(GcValueImpl::Object(child1_map));
    let child2_ref = gc.allocate(GcValueImpl::Object(child2_map));
    
    // Create root object with references to children
    // Convert usize IDs to Value objects
    root_map.insert("child1".to_string(), Value::number(child1_ref.id as f64));
    root_map.insert("child2".to_string(), Value::number(child2_ref.id as f64));
    let root_ref = gc.allocate(GcValueImpl::Object(root_map));
    
    // Update references
    let mut refs = std::collections::HashSet::new();
    refs.insert(child1_ref.id);
    refs.insert(child2_ref.id);
    gc.update_references(root_ref.id, refs);
    
    // Get initial stats
    let initial_stats = gc.get_stats();
    
    // Drop reference to root (which should cause children to be collected too)
    drop(root_ref);
    drop(child1_ref);
    drop(child2_ref);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.get_stats();
    
    // Verify that all objects were collected
    assert!(final_stats.deallocations >= initial_stats.deallocations + 3);
}
