// src/core/implicit_types.rs - Implicit type inference implementation
// This file contains the implementation of implicit type inference

use crate::error::LangError;
use crate::value::Value;

/// Infer the type of a value from its literal representation
pub fn infer_type_from_literal(literal: &str) -> Result<Value, LangError> {
    // Try to parse as number
    if let Ok(num) = literal.parse::<f64>() {
        return Ok(Value::number(num));
    }
    
    // Try to parse as boolean
    match literal.to_lowercase().as_str() {
        "true" | "⊤" => return Ok(Value::boolean(true)),
        "false" | "⊥" => return Ok(Value::boolean(false)),
        _ => {}
    }
    
    // If it starts and ends with quotes, it's a string
    if (literal.starts_with('"') && literal.ends_with('"')) || 
       (literal.starts_with('\'') && literal.ends_with('\'')) {
        let content = &literal[1..literal.len()-1];
        return Ok(Value::string(content));
    }
    
    // Default to string if no other type matches
    Ok(Value::string(literal))
}

/// Infer the type of a value from another value
pub fn infer_type_from_value(value: &Value) -> Result<String, LangError> {
    match value {
        Value::Number(_) => Ok("ι".to_string()),
        Value::String(_) => Ok("σ".to_string()),
        Value::Boolean(_) => Ok("β".to_string()),
        Value::Complex(_) => {
            // For complex values, we need to check the specific type
            match value.get_type() {
                crate::value::ValueType::Object => Ok("ο".to_string()),
                crate::value::ValueType::Array => Ok("α".to_string()),
                crate::value::ValueType::Function => Ok("φ".to_string()),
                _ => Err(LangError::runtime_error("Cannot infer type for complex value"))
            }
        },
        Value::Null => Ok("ν".to_string()),
    }
}

/// Check if a value can be coerced to a specific type
pub fn can_coerce(value: &Value, target_type: &str) -> bool {
    match (value, target_type) {
        // Number coercions
        (Value::String(s), "ι") => s.parse::<f64>().is_ok(),
        (Value::Boolean(_b), "ι") => true, // true -> 1, false -> 0
        
        // String coercions
        (Value::Number(_), "σ") => true,
        (Value::Boolean(_), "σ") => true,
        
        // Boolean coercions
        (Value::Number(_n), "β") => true, // 0 -> false, non-0 -> true
        (Value::String(s), "β") => match s.to_lowercase().as_str() {
            "true" | "yes" | "1" | "⊤" => true,
            "false" | "no" | "0" | "⊥" => true,
            _ => false,
        },
        
        // Same type, no coercion needed
        (Value::Number(_), "ι") => true,
        (Value::String(_), "σ") => true,
        (Value::Boolean(_), "β") => true,
        
        // Other cases
        _ => false,
    }
}

/// Coerce a value to a specific type
pub fn coerce_value(value: &Value, target_type: &str) -> Result<Value, LangError> {
    match (value, target_type) {
        // Number coercions
        (Value::String(s), "ι") => {
            match s.parse::<f64>() {
                Ok(n) => Ok(Value::number(n)),
                Err(_) => Err(LangError::runtime_error(&format!("Cannot coerce string '{}' to number", s))),
            }
        },
        (Value::Boolean(b), "ι") => Ok(Value::number(if *b { 1.0 } else { 0.0 })),
        
        // String coercions
        (Value::Number(n), "σ") => Ok(Value::string(n.to_string())),
        (Value::Boolean(b), "σ") => Ok(Value::string(b.to_string())),
        
        // Boolean coercions
        (Value::Number(n), "β") => Ok(Value::boolean(*n != 0.0)),
        (Value::String(s), "β") => {
            match s.to_lowercase().as_str() {
                "true" | "yes" | "1" | "⊤" => Ok(Value::boolean(true)),
                "false" | "no" | "0" | "⊥" => Ok(Value::boolean(false)),
                _ => Err(LangError::runtime_error(&format!("Cannot coerce string '{}' to boolean", s))),
            }
        },
        
        // Same type, no coercion needed
        (Value::Number(_), "ι") => Ok(value.clone()),
        (Value::String(_), "σ") => Ok(value.clone()),
        (Value::Boolean(_), "β") => Ok(value.clone()),
        
        // Other cases
        _ => Err(LangError::runtime_error(&format!("Cannot coerce value to type '{}'", target_type))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_infer_type_from_literal() {
        // Test number inference
        let num = infer_type_from_literal("42").unwrap();
        assert!(matches!(num, Value::Number(n) if n == 42.0));
        
        // Test boolean inference
        let bool_true = infer_type_from_literal("true").unwrap();
        assert!(matches!(bool_true, Value::Boolean(true)));
        
        let bool_false = infer_type_from_literal("⊥").unwrap();
        assert!(matches!(bool_false, Value::Boolean(false)));
        
        // Test string inference
        let string = infer_type_from_literal("\"hello\"").unwrap();
        assert!(matches!(string, Value::String(s) if s == "hello"));
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
    }
    
    #[test]
    fn test_coercion() {
        // Test number coercion
        let num = coerce_value(&Value::string("42"), "ι").unwrap();
        assert!(matches!(num, Value::Number(n) if n == 42.0));
        
        // Test boolean coercion
        let bool_true = coerce_value(&Value::number(1.0), "β").unwrap();
        assert!(matches!(bool_true, Value::Boolean(true)));
        
        let bool_false = coerce_value(&Value::string("false"), "β").unwrap();
        assert!(matches!(bool_false, Value::Boolean(false)));
        
        // Test string coercion
        let string = coerce_value(&Value::number(42.0), "σ").unwrap();
        assert!(matches!(string, Value::String(s) if s == "42"));
    }
}
