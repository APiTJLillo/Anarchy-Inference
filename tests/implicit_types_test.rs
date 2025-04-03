#[cfg(test)]
mod implicit_types_tests {
    use anarchy_inference::core::implicit_types::{infer_type_from_literal, infer_type_from_value, can_coerce, coerce_value};
    use anarchy_inference::value::Value;

    #[test]
    fn test_infer_type_from_literal() {
        // Test number inference
        let num = infer_type_from_literal("42").unwrap();
        assert!(matches!(num, Value::Number(n) if n == 42.0));
        
        let float = infer_type_from_literal("3.14").unwrap();
        assert!(matches!(float, Value::Number(n) if n == 3.14));
        
        // Test boolean inference
        let bool_true = infer_type_from_literal("true").unwrap();
        assert!(matches!(bool_true, Value::Boolean(true)));
        
        let bool_false = infer_type_from_literal("⊥").unwrap();
        assert!(matches!(bool_false, Value::Boolean(false)));
        
        // Test string inference
        let string = infer_type_from_literal("\"hello\"").unwrap();
        assert!(matches!(string, Value::String(s) if s == "hello"));
        
        let string2 = infer_type_from_literal("'world'").unwrap();
        assert!(matches!(string2, Value::String(s) if s == "world"));
    }
    
    #[test]
    fn test_infer_type_from_value() {
        // Test number type
        let num_type = infer_type_from_value(&Value::number(42.0)).unwrap();
        assert_eq!(num_type, "ι");
        
        // Test string type
        let str_type = infer_type_from_value(&Value::string("hello")).unwrap();
        assert_eq!(str_type, "σ");
        
        // Test boolean type
        let bool_type = infer_type_from_value(&Value::boolean(true)).unwrap();
        assert_eq!(bool_type, "β");
        
        // Test null type
        let null_type = infer_type_from_value(&Value::null()).unwrap();
        assert_eq!(null_type, "ν");
    }
    
    #[test]
    fn test_can_coerce() {
        // Test number coercions
        assert!(can_coerce(&Value::string("42"), "ι"));
        assert!(can_coerce(&Value::boolean(true), "ι"));
        assert!(!can_coerce(&Value::string("hello"), "ι"));
        
        // Test string coercions
        assert!(can_coerce(&Value::number(42.0), "σ"));
        assert!(can_coerce(&Value::boolean(true), "σ"));
        
        // Test boolean coercions
        assert!(can_coerce(&Value::number(1.0), "β"));
        assert!(can_coerce(&Value::string("true"), "β"));
        assert!(can_coerce(&Value::string("false"), "β"));
        assert!(!can_coerce(&Value::string("hello"), "β"));
        
        // Test same type (no coercion needed)
        assert!(can_coerce(&Value::number(42.0), "ι"));
        assert!(can_coerce(&Value::string("hello"), "σ"));
        assert!(can_coerce(&Value::boolean(true), "β"));
    }
    
    #[test]
    fn test_coerce_value() {
        // Test number coercion
        let num = coerce_value(&Value::string("42"), "ι").unwrap();
        assert!(matches!(num, Value::Number(n) if n == 42.0));
        
        let bool_to_num = coerce_value(&Value::boolean(true), "ι").unwrap();
        assert!(matches!(bool_to_num, Value::Number(n) if n == 1.0));
        
        // Test string coercion
        let num_to_str = coerce_value(&Value::number(42.0), "σ").unwrap();
        assert!(matches!(num_to_str, Value::String(s) if s == "42"));
        
        let bool_to_str = coerce_value(&Value::boolean(true), "σ").unwrap();
        assert!(matches!(bool_to_str, Value::String(s) if s == "true"));
        
        // Test boolean coercion
        let num_to_bool = coerce_value(&Value::number(1.0), "β").unwrap();
        assert!(matches!(num_to_bool, Value::Boolean(true)));
        
        let zero_to_bool = coerce_value(&Value::number(0.0), "β").unwrap();
        assert!(matches!(zero_to_bool, Value::Boolean(false)));
        
        let str_to_bool_true = coerce_value(&Value::string("true"), "β").unwrap();
        assert!(matches!(str_to_bool_true, Value::Boolean(true)));
        
        let str_to_bool_false = coerce_value(&Value::string("false"), "β").unwrap();
        assert!(matches!(str_to_bool_false, Value::Boolean(false)));
        
        // Test error cases
        let invalid_str_to_num = coerce_value(&Value::string("hello"), "ι");
        assert!(invalid_str_to_num.is_err());
        
        let invalid_str_to_bool = coerce_value(&Value::string("hello"), "β");
        assert!(invalid_str_to_bool.is_err());
    }
}
