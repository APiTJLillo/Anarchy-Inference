#[cfg(test)]
mod tests {
    use crate::value::{Value, ValueType};
    use crate::rc_value::{RcComplexValue, ComplexValue};
    use crate::interpreter::Interpreter;
    use crate::ast::{ASTNode, NodeType};
    use std::collections::HashMap;
    use crate::error::SourceLocation;

    #[test]
    fn test_rc_value_creation() {
        // Test creating a reference-counted complex value
        let complex = ComplexValue::new_object();
        let rc_complex = RcComplexValue::new(complex);
        
        // Check initial reference count
        assert_eq!(rc_complex.ref_count(), 1);
        
        // Clone the reference and check count
        let rc_complex2 = rc_complex.clone();
        assert_eq!(rc_complex.ref_count(), 2);
        assert_eq!(rc_complex2.ref_count(), 2);
        
        // Drop one reference and check count
        drop(rc_complex2);
        assert_eq!(rc_complex.ref_count(), 1);
    }
    
    #[test]
    fn test_value_with_rc() {
        // Test creating values with reference counting
        let null_val = Value::null();
        let num_val = Value::number(42.0);
        let bool_val = Value::boolean(true);
        let str_val = Value::string("hello".to_string());
        let obj_val = Value::empty_object();
        
        // Check types
        assert_eq!(null_val.get_type(), ValueType::Null);
        assert_eq!(num_val.get_type(), ValueType::Number);
        assert_eq!(bool_val.get_type(), ValueType::Boolean);
        assert_eq!(str_val.get_type(), ValueType::String);
        assert_eq!(obj_val.get_type(), ValueType::Object);
        
        // Check reference counts
        assert_eq!(obj_val.ref_count(), 1);
        
        // Clone object and check reference count
        let obj_val2 = obj_val.clone();
        assert_eq!(obj_val.ref_count(), 2);
        assert_eq!(obj_val2.ref_count(), 2);
        
        // Drop one reference and check count
        drop(obj_val2);
        assert_eq!(obj_val.ref_count(), 1);
    }
    
    #[test]
    fn test_object_properties() {
        // Test object property access with reference counting
        let obj_val = Value::empty_object();
        
        // Set properties
        obj_val.set_property("name".to_string(), Value::string("test".to_string())).unwrap();
        obj_val.set_property("value".to_string(), Value::number(123.0)).unwrap();
        
        // Get properties
        let name = obj_val.get_property("name").unwrap();
        let value = obj_val.get_property("value").unwrap();
        
        // Check property values
        if let Value::String(s) = &name {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string value");
        }
        
        if let Value::Number(n) = &value {
            assert_eq!(*n, 123.0);
        } else {
            panic!("Expected number value");
        }
    }
    
    #[test]
    fn test_array_elements() {
        // Test array element access with reference counting
        let elements = vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ];
        let arr_val = Value::array(elements);
        
        // Get elements
        let elem0 = arr_val.get_element(0).unwrap();
        let elem1 = arr_val.get_element(1).unwrap();
        let elem2 = arr_val.get_element(2).unwrap();
        
        // Check element values
        if let Value::Number(n) = &elem0 {
            assert_eq!(*n, 1.0);
        } else {
            panic!("Expected number value");
        }
        
        if let Value::Number(n) = &elem1 {
            assert_eq!(*n, 2.0);
        } else {
            panic!("Expected number value");
        }
        
        if let Value::Number(n) = &elem2 {
            assert_eq!(*n, 3.0);
        } else {
            panic!("Expected number value");
        }
        
        // Set element
        arr_val.set_element(1, Value::number(42.0)).unwrap();
        
        // Check updated element
        let updated_elem = arr_val.get_element(1).unwrap();
        if let Value::Number(n) = &updated_elem {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number value");
        }
    }
    
    #[test]
    fn test_interpreter_memory_tracking() {
        // Test memory tracking in the interpreter
        let mut interpreter = Interpreter::new();
        
        // Create objects and check memory stats
        let obj1 = interpreter.create_object();
        let obj2 = interpreter.create_object();
        let obj3 = interpreter.create_object();
        
        let stats = interpreter.get_memory_stats();
        assert_eq!(stats.objects_allocated, 3);
        assert_eq!(stats.total_complex_values, 3);
        
        // Create arrays and check memory stats
        let arr1 = interpreter.create_array(vec![Value::number(1.0), Value::number(2.0)]);
        let arr2 = interpreter.create_array(vec![Value::number(3.0), Value::number(4.0)]);
        
        let stats = interpreter.get_memory_stats();
        assert_eq!(stats.arrays_allocated, 2);
        assert_eq!(stats.total_complex_values, 5);
        
        // Create functions and check memory stats
        let func1 = interpreter.create_function(
            "test".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Box::new(ASTNode {
                node_type: NodeType::Number(42),
                location: SourceLocation::default(),
            })
        );
        
        let stats = interpreter.get_memory_stats();
        assert_eq!(stats.functions_allocated, 1);
        assert_eq!(stats.total_complex_values, 6);
    }
    
    #[test]
    fn test_complex_object_graph() {
        // Test complex object graph with reference counting
        let mut interpreter = Interpreter::new();
        
        // Create parent object
        let parent = interpreter.create_object();
        
        // Create child objects
        let child1 = interpreter.create_object();
        let child2 = interpreter.create_object();
        
        // Set up parent-child relationships
        parent.set_property("child1".to_string(), child1.clone()).unwrap();
        parent.set_property("child2".to_string(), child2.clone()).unwrap();
        
        // Set up circular reference (child1 references parent)
        child1.set_property("parent".to_string(), parent.clone()).unwrap();
        
        // Check reference counts
        assert!(parent.ref_count() > 1); // Referenced by: original var, child1
        assert!(child1.ref_count() > 1); // Referenced by: original var, parent
        assert!(child2.ref_count() > 1); // Referenced by: original var, parent
        
        // Break circular reference
        child1.set_property("parent".to_string(), Value::null()).unwrap();
        
        // Check memory stats
        let stats = interpreter.get_memory_stats();
        assert_eq!(stats.objects_allocated, 3);
        assert_eq!(stats.total_complex_values, 3);
    }
}
