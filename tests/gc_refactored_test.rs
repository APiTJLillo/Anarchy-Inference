// Update the test file to use the correct imports

use anarchy_inference::core::value::{Value, ValueType};
use anarchy_inference::core::gc_types::GarbageCollected;
use anarchy_inference::gc::collector::GarbageCollector;
use anarchy_inference::gc::managed::GcValueImpl;
use anarchy_inference::interpreter::evaluator::Interpreter;
use std::collections::HashMap;

#[test]
fn test_gc_basic_allocation() {
    let mut interpreter = Interpreter::new();
    
    // Get initial stats
    let initial_stats = interpreter.get_gc_stats();
    
    // Create some test values and allocate them with GC
    let obj1 = GcValueImpl::Object(vec![
        ("name".to_string(), Value::Primitive(ValueType::String("Object 1".to_string()))),
        ("value".to_string(), Value::Primitive(ValueType::Number(42))),
    ].into_iter().collect());
    
    let obj2 = GcValueImpl::Object(vec![
        ("name".to_string(), Value::Primitive(ValueType::String("Object 2".to_string()))),
        ("value".to_string(), Value::Primitive(ValueType::Number(84))),
    ].into_iter().collect());
    
    let obj3 = GcValueImpl::Object(vec![
        ("name".to_string(), Value::Primitive(ValueType::String("Object 3".to_string()))),
        ("value".to_string(), Value::Primitive(ValueType::Number(126))),
    ].into_iter().collect());
    
    // Allocate objects with GC
    let _gc_obj1 = interpreter.allocate_complex_value(obj1);
    let _gc_obj2 = interpreter.allocate_complex_value(obj2);
    let _gc_obj3 = interpreter.allocate_complex_value(obj3);
    
    // Force garbage collection
    interpreter.collect_garbage();
    
    // Get final stats
    let final_stats = interpreter.get_gc_stats();
    
    // We should have allocated at least 3 objects
    assert!(final_stats.allocations >= initial_stats.allocations + 3);
}

#[test]
fn test_gc_memory_management() {
    let mut interpreter = Interpreter::new();
    
    // Get initial stats
    let initial_stats = interpreter.get_gc_stats();
    
    // Create a scope to test object lifetime
    {
        // Create some test values and allocate them with GC
        let obj1 = GcValueImpl::Object(vec![
            ("name".to_string(), Value::Primitive(ValueType::String("Temporary Object".to_string()))),
            ("value".to_string(), Value::Primitive(ValueType::Number(42))),
        ].into_iter().collect());
        
        // Allocate object with GC
        let _gc_obj1 = interpreter.allocate_complex_value(obj1);
        
        // Object should be allocated but not yet collected
        let mid_stats = interpreter.get_gc_stats();
        assert!(mid_stats.allocations > initial_stats.allocations);
    }
    
    // Force garbage collection after object goes out of scope
    interpreter.collect_garbage();
    
    // Get final stats
    let final_stats = interpreter.get_gc_stats();
    
    // We should have deallocated at least one object
    assert!(final_stats.deallocations > initial_stats.deallocations);
}

#[test]
fn test_gc_cycle_detection() {
    let gc = GarbageCollector::new();
    
    // Create objects that reference each other in a cycle
    let mut obj1_map = HashMap::new();
    let mut obj2_map = HashMap::new();
    
    // Allocate first object
    let obj1 = GcValueImpl::Object(obj1_map.clone());
    let gc_obj1 = gc.allocate(obj1);
    
    // Allocate second object
    let obj2 = GcValueImpl::Object(obj2_map.clone());
    let gc_obj2 = gc.allocate(obj2);
    
    // Create cycle: obj1 -> obj2 -> obj1
    obj1_map.insert("next".to_string(), Value::GcManaged(gc_obj2.clone()));
    obj2_map.insert("next".to_string(), Value::GcManaged(gc_obj1.clone()));
    
    // Update the objects with their new references
    if let Some(GcValueImpl::Object(map)) = gc.get_value(gc_obj1.id) {
        gc.update_references(gc_obj1.id, gc.extract_references(&GcValueImpl::Object(map)));
    }
    
    if let Some(GcValueImpl::Object(map)) = gc.get_value(gc_obj2.id) {
        gc.update_references(gc_obj2.id, gc.extract_references(&GcValueImpl::Object(map)));
    }
    
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
