// src/std_lib.rs - Modified to include string dictionary support
// This file contains the standard library functions

use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::core::string_dict::StringDictionary;
use std::fs;

/// Initialize the standard library
pub fn init(interpreter: &mut Interpreter) {
    // Initialize standard library functions
    // ...

    // Initialize string dictionary functions
    init_string_dict_functions(interpreter);
}

/// Initialize string dictionary functions
fn init_string_dict_functions(interpreter: &mut Interpreter) {
    // Define string dictionary functions in the global environment
    
    // 🔠 - Load string dictionary from file
    interpreter.environment.define("🔠".to_string(), Value::native_function(|interpreter, args| {
        if args.len() != 1 {
            return Err("🔠 requires 1 argument: path".into());
        }
        
        let path = args[0].to_string();
        interpreter.load_string_dictionary(&path)?;
        Ok(Value::boolean(true))
    }));
    
    // 📝 - Set string in dictionary
    interpreter.environment.define("📝".to_string(), Value::native_function(|interpreter, args| {
        if args.len() != 2 {
            return Err("📝 requires 2 arguments: key, value".into());
        }
        
        let key = args[0].to_string();
        let value = args[1].to_string();
        
        interpreter.set_string(key, value);
        Ok(Value::boolean(true))
    }));
    
    // 📖 - Get string from dictionary
    interpreter.environment.define("📖".to_string(), Value::native_function(|interpreter, args| {
        if args.len() != 1 {
            return Err("📖 requires 1 argument: key".into());
        }
        
        let key = args[0].to_string();
        
        if let Some(value) = interpreter.get_string(&key) {
            Ok(Value::string(value))
        } else {
            Ok(Value::null())
        }
    }));
    
    // 💾 - Save string dictionary to file
    interpreter.environment.define("💾".to_string(), Value::native_function(|interpreter, args| {
        if args.len() != 2 {
            return Err("💾 requires 2 arguments: dictionary_name, path".into());
        }
        
        let dict_name = args[0].to_string();
        let path = args[1].to_string();
        
        let dict_manager = interpreter.get_string_dict_manager();
        dict_manager.save_dictionary(&dict_name, &path)?;
        
        Ok(Value::boolean(true))
    }));
    
    // 🔄 - Switch active dictionary
    interpreter.environment.define("🔄".to_string(), Value::native_function(|interpreter, args| {
        if args.len() != 1 {
            return Err("🔄 requires 1 argument: dictionary_name".into());
        }
        
        let dict_name = args[0].to_string();
        
        let dict_manager = interpreter.get_string_dict_manager_mut();
        dict_manager.set_current(&dict_name)?;
        
        Ok(Value::boolean(true))
    }));
}
