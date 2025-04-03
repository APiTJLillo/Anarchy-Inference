#[cfg(test)]
mod std_tests {
    use std::fs;
    use std::path::Path;
    use std::env;
    use anarchy_inference::value::Value;
    use anarchy_inference::std::fs as ai_fs;
    use anarchy_inference::std::shell as ai_shell;
    use anarchy_inference::std::http as ai_http;
    use anarchy_inference::std::crypto as ai_crypto;
    use anarchy_inference::std::mem as ai_mem;
    use anarchy_inference::std::security;

    // Helper function to create a test file
    fn create_test_file(path: &str, content: &str) {
        fs::write(path, content).expect("Failed to write test file");
    }

    // Helper function to clean up test files
    fn cleanup_test_file(path: &str) {
        if Path::new(path).exists() {
            fs::remove_file(path).expect("Failed to remove test file");
        }
    }

    #[test]
    fn test_fs_operations() {
        // Enable file system operations
        security::set_allow_fs(true);

        // Test file_exists
        let test_path = "test_file.txt";
        assert_eq!(ai_fs::file_exists(test_path).unwrap(), Value::boolean(false));

        // Test write_file
        let test_content = "Hello, world!";
        ai_fs::write_file(test_path, test_content, None).unwrap();
        assert_eq!(ai_fs::file_exists(test_path).unwrap(), Value::boolean(true));

        // Test read_file
        let read_content = ai_fs::read_file(test_path).unwrap();
        if let Value::String(content) = read_content {
            assert_eq!(content, test_content);
        } else {
            panic!("Expected string value from read_file");
        }

        // Test append mode
        let append_content = "\nAppended content";
        ai_fs::write_file(test_path, append_content, Some("a")).unwrap();
        let read_content = ai_fs::read_file(test_path).unwrap();
        if let Value::String(content) = read_content {
            assert_eq!(content, format!("{}{}", test_content, append_content));
        } else {
            panic!("Expected string value from read_file after append");
        }

        // Test copy_file
        let copy_path = "test_file_copy.txt";
        ai_fs::copy_file(test_path, copy_path).unwrap();
        assert_eq!(ai_fs::file_exists(copy_path).unwrap(), Value::boolean(true));

        // Test list_dir
        let dir_contents = ai_fs::list_dir(".").unwrap();
        if let Value::Complex(complex) = dir_contents {
            let borrowed = complex.borrow();
            if let Some(array_data) = &borrowed.array_data {
                assert!(array_data.iter().any(|item| {
                    if let Value::String(name) = item {
                        name == test_path
                    } else {
                        false
                    }
                }));
            } else {
                panic!("Expected array data from list_dir");
            }
        } else {
            panic!("Expected complex value from list_dir");
        }

        // Test move_file
        let move_path = "test_file_moved.txt";
        ai_fs::move_file(copy_path, move_path).unwrap();
        assert_eq!(ai_fs::file_exists(copy_path).unwrap(), Value::boolean(false));
        assert_eq!(ai_fs::file_exists(move_path).unwrap(), Value::boolean(true));

        // Test remove_path
        ai_fs::remove_path(test_path).unwrap();
        ai_fs::remove_path(move_path).unwrap();
        assert_eq!(ai_fs::file_exists(test_path).unwrap(), Value::boolean(false));
        assert_eq!(ai_fs::file_exists(move_path).unwrap(), Value::boolean(false));
    }

    #[test]
    fn test_shell_operations() {
        // Enable shell operations
        security::set_allow_shell(true);

        // Test current_os
        let os = ai_shell::current_os().unwrap();
        if let Value::String(os_name) = os {
            assert!(!os_name.is_empty());
        } else {
            panic!("Expected string value from current_os");
        }

        // Test get_env_var
        let path_var = ai_shell::get_env_var("PATH").unwrap();
        if let Value::String(path) = path_var {
            assert!(!path.is_empty());
        } else {
            panic!("Expected string value from get_env_var");
        }

        // Test execute_shell
        let result = ai_shell::execute_shell("echo 'test'").unwrap();
        if let Value::Complex(complex) = result {
            let borrowed = complex.borrow();
            if let Some(obj_data) = &borrowed.object_data {
                if let Some(Value::String(stdout)) = obj_data.get("o") {
                    assert!(stdout.contains("test"));
                } else {
                    panic!("Expected stdout in shell result");
                }
            } else {
                panic!("Expected object data from execute_shell");
            }
        } else {
            panic!("Expected complex value from execute_shell");
        }
    }

    #[test]
    fn test_crypto_operations() {
        // Test hash_string
        let input = "test string";
        
        // Test SHA256
        let sha256_result = ai_crypto::hash_string(input, "sha256").unwrap();
        if let Value::String(hash) = sha256_result {
            assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
        } else {
            panic!("Expected string value from hash_string with SHA256");
        }
        
        // Test MD5
        let md5_result = ai_crypto::hash_string(input, "md5").unwrap();
        if let Value::String(hash) = md5_result {
            assert_eq!(hash.len(), 32); // MD5 produces 32 hex characters
        } else {
            panic!("Expected string value from hash_string with MD5");
        }
        
        // Test hash_file
        security::set_allow_fs(true);
        let test_path = "test_hash_file.txt";
        create_test_file(test_path, input);
        
        let file_hash = ai_crypto::hash_file(test_path, "sha256").unwrap();
        if let Value::String(hash) = file_hash {
            assert_eq!(hash.len(), 64);
        } else {
            panic!("Expected string value from hash_file");
        }
        
        cleanup_test_file(test_path);
    }

    #[test]
    fn test_memory_operations() {
        // Test set_memory and get_memory
        let key = "test_key";
        let value = Value::string("test_value");
        
        ai_mem::set_memory(key, value.clone()).unwrap();
        let retrieved = ai_mem::get_memory(key).unwrap();
        
        if let Value::String(val) = retrieved {
            assert_eq!(val, "test_value");
        } else {
            panic!("Expected string value from get_memory");
        }
        
        // Test forget_memory
        ai_mem::forget_memory(key).unwrap();
        let after_forget = ai_mem::get_memory(key).unwrap();
        assert_eq!(after_forget, Value::null());
    }

    // Note: HTTP and Browser tests are not included as they require network access
    // and would make the tests dependent on external services
}
