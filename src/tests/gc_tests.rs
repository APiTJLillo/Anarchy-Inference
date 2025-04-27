// src/tests/gc_tests.rs - Tests for the garbage collection system
// This file contains tests for the garbage collection functionality

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::{ASTNode, NodeType};
use crate::core::value::GcValue;
use crate::gc::managed::GcValueImpl;
use crate::gc::{GarbageCollector, GarbageCollected};
use crate::interpreter::Interpreter;
use crate::value::Value;

#[test]
fn test_gc_basic_allocation() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create a simple object
    let mut object = HashMap::new();
    object.insert("name".to_string(), Value::String("test".to_string()));
    object.insert("value".to_string(), Value::Number(42.0));
    
    let object_value = GcValueImpl::Object(object);
    
    // Allocate the object
    let gc_value = gc.allocate(object_value);
    
    // Check that the object was allocated
    assert!(gc.get_value(gc_value.id).is_some());
    
    // Check the stats
    let stats = gc.get_stats();
    assert_eq!(stats.allocations, 1);
    assert_eq!(stats.deallocations, 0);
    assert!(stats.total_memory > 0);
}

#[test]
fn test_gc_reference_counting() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create a simple object
    let object_value = GcValueImpl::new_object();
    
    // Allocate the object
    let gc_value = gc.allocate(object_value);
    
    // Check initial reference count (should be 1)
    let stats_before = gc.get_stats();
    
    // Increment reference count
    gc.increment_ref_count(gc_value.id);
    
    // Decrement reference count back to 1
    gc.decrement_ref_count(gc_value.id);
    
    // Object should still exist
    assert!(gc.get_value(gc_value.id).is_some());
    
    // Decrement reference count to 0
    gc.decrement_ref_count(gc_value.id);
    
    // Force collection
    gc.collect();
    
    // Object should be collected
    assert!(gc.get_value(gc_value.id).is_none());
    
    // Check the stats
    let stats_after = gc.get_stats();
    assert_eq!(stats_after.deallocations, 1);
    assert_eq!(stats_after.collections_performed, 1);
}

#[test]
fn test_gc_cycle_detection() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Create two objects that reference each other
    let object1_value = GcValueImpl::new_object();
    let object2_value = GcValueImpl::new_object();
    
    let gc_value1 = gc.allocate(object1_value);
    let gc_value2 = gc.allocate(object2_value);
    
    // Create circular references
    let mut refs1 = std::collections::HashSet::new();
    refs1.insert(gc_value2.id);
    
    let mut refs2 = std::collections::HashSet::new();
    refs2.insert(gc_value1.id);
    
    gc.update_references(gc_value1.id, refs1);
    gc.update_references(gc_value2.id, refs2);
    
    // Both objects should exist
    assert!(gc.get_value(gc_value1.id).is_some());
    assert!(gc.get_value(gc_value2.id).is_some());
    
    // Decrement reference counts to 0 (external references)
    gc.decrement_ref_count(gc_value1.id);
    gc.decrement_ref_count(gc_value2.id);
    
    // Force collection
    gc.collect();
    
    // Both objects should be collected (cycle detection)
    assert!(gc.get_value(gc_value1.id).is_none());
    assert!(gc.get_value(gc_value2.id).is_none());
    
    // Check the stats
    let stats = gc.get_stats();
    assert_eq!(stats.deallocations, 2);
    assert!(stats.cycles_detected > 0);
}

#[test]
fn test_gc_with_interpreter() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Create a simple program that creates objects
    let program = r#"
    ι obj1 = { name: "Object 1", value: 42 };
    ι obj2 = { name: "Object 2", value: 84 };
    ι obj3 = { name: "Object 3", value: 126 };
    
    // Create a reference cycle
    obj1.ref = obj2;
    obj2.ref = obj1;
    
    // Return obj3 (should be kept)
    obj3
    "#;
    
    // Parse and execute the program
    let lexer = crate::lexer::Lexer::new(program.to_string());
    let mut parser = crate::parser::Parser::from_lexer(lexer).unwrap();
    let nodes = parser.parse().unwrap();
    
    // Execute the program
    let result = interpreter.execute_nodes(&nodes).unwrap();
    
    // Get GC stats before collection
    let stats_before = interpreter.get_gc_stats();
    
    // Force garbage collection
    interpreter.collect_garbage();
    
    // Get GC stats after collection
    let stats_after = interpreter.get_gc_stats();
    
    // Check that some objects were collected
    assert!(stats_after.deallocations > 0);
    
    // Check that the result is still valid
    match result {
        Value::GcManaged(gc_value) => {
            assert!(interpreter.get_gc_stats().total_memory > 0);
        },
        _ => panic!("Expected GcManaged value"),
    }
}

#[test]
fn test_gc_with_complex_structures() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Create a program with nested objects and arrays
    let program = r#"
    // Create a complex object structure
    ι data = {
        users: [
            { name: "Alice", age: 30, roles: ["admin", "user"] },
            { name: "Bob", age: 25, roles: ["user"] },
            { name: "Charlie", age: 35, roles: ["user", "moderator"] }
        ],
        settings: {
            theme: "dark",
            notifications: true,
            preferences: {
                language: "en",
                timezone: "UTC"
            }
        }
    };
    
    // Create some functions
    ƒ getUserNames(data) {
        ι names = [];
        ι i = 0;
        ι len = data.users.length;
        
        while (i < len) {
            names.push(data.users[i].name);
            i = i + 1;
        }
        
        ⟼ names;
    }
    
    // Return the data
    data
    "#;
    
    // Parse and execute the program
    let lexer = crate::lexer::Lexer::new(program.to_string());
    let mut parser = crate::parser::Parser::from_lexer(lexer).unwrap();
    let nodes = parser.parse().unwrap();
    
    // Execute the program
    let result = interpreter.execute_nodes(&nodes).unwrap();
    
    // Get GC stats
    let stats = interpreter.get_gc_stats();
    
    // Check that objects were allocated
    assert!(stats.allocations > 0);
    assert!(stats.total_memory > 0);
    
    // Force garbage collection
    interpreter.collect_garbage();
    
    // Check that the result is still valid
    match result {
        Value::GcManaged(gc_value) => {
            // Try to access a property to verify the object is still valid
            let property = result.get_property("users").unwrap();
            assert!(matches!(property, Value::GcManaged(_)));
        },
        _ => panic!("Expected GcManaged value"),
    }
}

#[test]
fn test_gc_generational_collection() {
    // Create a garbage collector with custom settings
    let gc = GarbageCollector::with_settings(1024 * 1024, true, 3, 100);
    
    // Create objects in different generations
    let mut objects = Vec::new();
    
    // Create 100 objects
    for i in 0..100 {
        let object_value = GcValueImpl::new_object();
        let gc_value = gc.allocate(object_value);
        objects.push(gc_value);
    }
    
    // Collect generation 0
    gc.collect_generation(0);
    
    // Check that objects are still there (they have references)
    for gc_value in &objects {
        assert!(gc.get_value(gc_value.id).is_some());
    }
    
    // Drop references to half the objects
    for i in 0..50 {
        gc.decrement_ref_count(objects[i].id);
    }
    
    // Collect generation 0 again
    gc.collect_generation(0);
    
    // Check that unreferenced objects in generation 0 are gone
    for i in 0..50 {
        assert!(gc.get_value(objects[i].id).is_none());
    }
    
    // Check that referenced objects are still there
    for i in 50..100 {
        assert!(gc.get_value(objects[i].id).is_some());
    }
    
    // Check the stats
    let stats = gc.get_stats();
    assert_eq!(stats.deallocations, 50);
}

#[test]
fn test_gc_incremental_collection() {
    // Create a garbage collector with custom settings
    let gc = GarbageCollector::with_settings(1024 * 1024, true, 3, 10);
    
    // Create 100 objects
    let mut objects = Vec::new();
    
    for i in 0..100 {
        let object_value = GcValueImpl::new_object();
        let gc_value = gc.allocate(object_value);
        objects.push(gc_value);
    }
    
    // Drop references to all objects
    for gc_value in &objects {
        gc.decrement_ref_count(gc_value.id);
    }
    
    // Perform incremental collection (should collect 10 objects)
    gc.collect_incremental_step();
    
    // Check the stats
    let stats = gc.get_stats();
    assert_eq!(stats.deallocations, 10);
    
    // Perform more incremental collections until all objects are collected
    for _ in 0..9 {
        gc.collect_incremental_step();
    }
    
    // Check that all objects are gone
    for gc_value in &objects {
        assert!(gc.get_value(gc_value.id).is_none());
    }
    
    // Check the stats
    let stats = gc.get_stats();
    assert_eq!(stats.deallocations, 100);
}

#[test]
fn test_gc_memory_tracking() {
    // Create a garbage collector
    let gc = GarbageCollector::new();
    
    // Get initial memory usage
    let initial_memory = gc.memory_usage();
    
    // Create a large object
    let mut large_array = Vec::new();
    for i in 0..1000 {
        large_array.push(Value::Number(i as f64));
    }
    
    let array_value = GcValueImpl::Array(large_array);
    let gc_value = gc.allocate(array_value);
    
    // Check that memory usage increased
    let memory_after_allocation = gc.memory_usage();
    assert!(memory_after_allocation > initial_memory);
    
    // Drop reference and collect
    gc.decrement_ref_count(gc_value.id);
    gc.collect();
    
    // Check that memory usage decreased
    let memory_after_collection = gc.memory_usage();
    assert!(memory_after_collection < memory_after_allocation);
    
    // Check the stats
    let stats = gc.get_stats();
    assert_eq!(stats.deallocations, 1);
    assert!(stats.peak_memory >= memory_after_allocation);
}

#[test]
fn test_gc_with_functions_and_closures() {
    // Create an interpreter
    let mut interpreter = Interpreter::new();
    
    // Create a program with functions and closures
    let program = r#"
    // Create a function that returns a closure
    ƒ createCounter(start) {
        ι count = start;
        
        // Return a closure that captures count
        ƒ counter() {
            count = count + 1;
            ⟼ count;
        }
        
        ⟼ counter;
    }
    
    // Create two counters
    ι counter1 = createCounter(0);
    ι counter2 = createCounter(10);
    
    // Use the counters
    ι result1 = counter1();  // 1
    ι result2 = counter1();  // 2
    ι result3 = counter2();  // 11
    
    // Return the results
    [result1, result2, result3]
    "#;
    
    // Parse and execute the program
    let lexer = crate::lexer::Lexer::new(program.to_string());
    let mut parser = crate::parser::Parser::from_lexer(lexer).unwrap();
    let nodes = parser.parse().unwrap();
    
    // Execute the program
    let result = interpreter.execute_nodes(&nodes).unwrap();
    
    // Get GC stats before collection
    let stats_before = interpreter.get_gc_stats();
    
    // Force garbage collection
    interpreter.collect_garbage();
    
    // Get GC stats after collection
    let stats_after = interpreter.get_gc_stats();
    
    // Check that the result is still valid
    match result {
        Value::GcManaged(gc_value) => {
            // Try to access elements to verify the array is still valid
            let element0 = result.get_element(0).unwrap();
            let element1 = result.get_element(1).unwrap();
            let element2 = result.get_element(2).unwrap();
            
            match (element0, element1, element2) {
                (Value::Number(n1), Value::Number(n2), Value::Number(n3)) => {
                    assert_eq!(n1, 1.0);
                    assert_eq!(n2, 2.0);
                    assert_eq!(n3, 11.0);
                },
                _ => panic!("Expected Number values"),
            }
        },
        _ => panic!("Expected GcManaged value"),
    }
}
