// Update the test file to use the correct imports and API

use anarchy_inference::core::value::Value;
use anarchy_inference::core::gc_types::GarbageCollector;
use anarchy_inference::garbage_collection::collector::GarbageCollector as GcImpl;
use anarchy_inference::garbage_collection::managed::GcValueImpl;
use anarchy_inference::interpreter::Interpreter;
use std::collections::HashMap;

#[test]
fn test_gc_basic_allocation() {
    let mut interpreter = Interpreter::new();
    
    // Create some test values and allocate them with GC
    let obj1 = GcValueImpl::Object(HashMap::new());
    let obj2 = GcValueImpl::Object(HashMap::new());
    let obj3 = GcValueImpl::Object(HashMap::new());
    
    // Allocate objects with GC
    let _gc_obj1 = interpreter.create_object();
    let _gc_obj2 = interpreter.create_object();
    let _gc_obj3 = interpreter.create_object();
    
    // Get stats after allocation
    let final_stats = interpreter.get_memory_stats();
    
    // We should have allocated at least 3 objects
    assert!(final_stats.objects_allocated >= 3);
}

#[test]
fn test_gc_memory_management() {
    let mut interpreter = Interpreter::new();
    
    // Create a scope to test object lifetime
    {
        // Create and allocate object with GC
        let _gc_obj1 = interpreter.create_object();
    }
    
    // Get stats after allocation and scope exit
    let stats = interpreter.get_memory_stats();
    
    // We should have allocated at least one object
    assert!(stats.objects_allocated > 0);
}

#[test]
fn test_gc_cycle_detection() {
    let gc = GcImpl::new();
    
    // Create objects that reference each other in a cycle
    let obj1_map = HashMap::new();
    let obj2_map = HashMap::new();
    
    // Allocate first object
    let gc_obj1 = gc.allocate(GcValueImpl::Object(obj1_map));
    
    // Allocate second object
    let gc_obj2 = gc.allocate(GcValueImpl::Object(obj2_map));
    
    // Create cycle: obj1 -> obj2 -> obj1
    let mut refs1 = std::collections::HashSet::new();
    refs1.insert(gc_obj2.id);
    gc.update_references(gc_obj1.id, refs1);
    
    let mut refs2 = std::collections::HashSet::new();
    refs2.insert(gc_obj1.id);
    gc.update_references(gc_obj2.id, refs2);
    
    // Get initial stats
    let initial_stats = gc.get_stats();
    
    // Drop references to the objects
    drop(gc_obj1);
    drop(gc_obj2);
    
    // Force garbage collection
    gc.collect();
    
    // Get final stats
    let final_stats = gc.get_stats();
    
    // We should have detected and collected the cycle
    assert!(final_stats.cycles_detected > initial_stats.cycles_detected);
}
