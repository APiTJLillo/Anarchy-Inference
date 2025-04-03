// tests/gc_test_comprehensive.rs
// Comprehensive tests for the garbage collection implementation

use std::collections::HashMap;
use std::sync::Arc;
use anarchy_inference::memory::allocator::{Allocator, AllocatorStats};
use anarchy_inference::memory::gc::GarbageCollector;
use anarchy_inference::memory::reference::Ref;
use anarchy_inference::value::types::{Value, ValueType, Function};
use anarchy_inference::runtime::interpreter::Interpreter;
use anarchy_inference::runtime::environment::Environment;

#[test]
fn test_basic_allocation() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Get initial stats
    let initial_stats = gc.stats();
    
    // Allocate some objects
    let _obj1 = gc.allocate(42);
    let _obj2 = gc.allocate("Hello, world!".to_string());
    let _obj3 = gc.allocate(vec![1, 2, 3]);
    
    // Get updated stats
    let updated_stats = gc.stats();
    
    // Verify that allocations increased
    assert!(updated_stats.allocations > initial_stats.allocations);
    assert_eq!(updated_stats.allocations, initial_stats.allocations + 3);
}

#[test]
fn test_reference_counting() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Allocate an object
    let obj = gc.allocate(42);
    
    // Get initial stats
    let initial_stats = gc.stats();
    
    // Create a scope to test reference counting
    {
        // Clone the reference to increase reference count
        let _obj_clone = obj.clone();
        
        // Force garbage collection
        gc.collect();
        
        // Get stats after collection
        let mid_stats = gc.stats();
        
        // Verify that no deallocations occurred
        assert_eq!(mid_stats.deallocations, initial_stats.deallocations);
    }
    
    // Clone has gone out of scope, reference count decreased
    
    // Drop the original reference
    drop(obj);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.stats();
    
    // Verify that a deallocation occurred
    assert!(final_stats.deallocations > initial_stats.deallocations);
}

#[test]
fn test_cycle_detection() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create objects that reference each other in a cycle
    let mut obj1_map = HashMap::new();
    let mut obj2_map = HashMap::new();
    
    // Allocate first object
    let obj1_ref = gc.allocate(obj1_map);
    
    // Allocate second object
    let obj2_ref = gc.allocate(obj2_map);
    
    // Get references to modify the objects
    let obj1_id = obj1_ref.id();
    let obj2_id = obj2_ref.id();
    
    // Create cycle: obj1 -> obj2 -> obj1
    gc.update_references(obj1_id, vec![obj2_id]);
    gc.update_references(obj2_id, vec![obj1_id]);
    
    // Get initial stats
    let initial_stats = gc.stats();
    
    // Drop references to the objects
    drop(obj1_ref);
    drop(obj2_ref);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.stats();
    
    // Verify that cycles were detected and collected
    assert!(final_stats.cycles_detected > initial_stats.cycles_detected);
    assert!(final_stats.deallocations > initial_stats.deallocations);
}

#[test]
fn test_memory_usage_statistics() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Get initial stats
    let initial_stats = gc.stats();
    
    // Allocate many objects
    for i in 0..100 {
        let _obj = gc.allocate(i);
    }
    
    // Get updated stats
    let updated_stats = gc.stats();
    
    // Verify that memory usage increased
    assert!(updated_stats.memory_usage > initial_stats.memory_usage);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.stats();
    
    // Verify that collections were performed
    assert!(final_stats.collections > initial_stats.collections);
}

#[test]
fn test_interpreter_integration() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Get initial stats
    let initial_stats = interpreter.get_gc_stats();
    
    // Create and allocate complex values
    let obj = HashMap::new();
    let obj_ref = interpreter.allocate(obj);
    let value = Value::object(obj_ref);
    
    // Define a variable with the complex value
    interpreter.environment.define("test".to_string(), value);
    
    // Force garbage collection
    interpreter.collect_garbage();
    
    // Get final stats
    let final_stats = interpreter.get_gc_stats();
    
    // Verify that allocations occurred
    assert!(final_stats.allocations > initial_stats.allocations);
    
    // Verify that the variable still exists
    assert!(interpreter.environment.has("test"));
}

#[test]
fn test_automatic_collection_triggers() {
    // Create an interpreter with a low GC threshold
    let mut interpreter = Interpreter::new();
    interpreter.set_gc_threshold(10);
    
    // Get initial stats
    let initial_stats = interpreter.get_gc_stats();
    
    // Allocate many objects to trigger automatic collection
    for i in 0..20 {
        let obj = HashMap::new();
        let obj_ref = interpreter.allocate(obj);
        let value = Value::object(obj_ref);
        interpreter.environment.define(format!("test{}", i), value);
    }
    
    // Get final stats
    let final_stats = interpreter.get_gc_stats();
    
    // Verify that collections were automatically triggered
    assert!(final_stats.collections > initial_stats.collections);
}

#[test]
fn test_complex_object_graph() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create a complex object graph
    let mut root_map = HashMap::new();
    let mut child1_map = HashMap::new();
    let mut child2_map = HashMap::new();
    
    // Allocate child objects
    let child1_ref = gc.allocate(child1_map);
    let child2_ref = gc.allocate(child2_map);
    
    // Create root object with references to children
    root_map.insert("child1".to_string(), child1_ref.id());
    root_map.insert("child2".to_string(), child2_ref.id());
    let root_ref = gc.allocate(root_map);
    
    // Update references
    gc.update_references(root_ref.id(), vec![child1_ref.id(), child2_ref.id()]);
    
    // Get initial stats
    let initial_stats = gc.stats();
    
    // Drop reference to root (which should cause children to be collected too)
    drop(root_ref);
    drop(child1_ref);
    drop(child2_ref);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.stats();
    
    // Verify that all objects were collected
    assert!(final_stats.deallocations >= initial_stats.deallocations + 3);
}
