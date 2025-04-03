#[cfg(test)]
mod string_dict_tests {
    use anarchy_inference::core::string_dict::{StringDictionary, StringDictionaryManager};
    use anarchy_inference::error::LangError;

    #[test]
    fn test_string_dictionary_basic() {
        let mut dict = StringDictionary::new("test");
        dict.set("greeting".to_string(), "Hello, {}!".to_string());
        dict.set("farewell".to_string(), "Goodbye, {}!".to_string());
        
        assert_eq!(dict.get("greeting"), Some(&"Hello, {}!".to_string()));
        assert_eq!(dict.get("farewell"), Some(&"Goodbye, {}!".to_string()));
        assert_eq!(dict.get("nonexistent"), None);
    }
    
    #[test]
    fn test_string_formatting() {
        let mut dict = StringDictionary::new("test");
        dict.set("greeting".to_string(), "Hello, {}!".to_string());
        dict.set("complex".to_string(), "Hello, {}! Your score is {}.".to_string());
        
        let result = dict.format("greeting", &["World".to_string()]).unwrap();
        assert_eq!(result, "Hello, World!");
        
        let result = dict.format("complex", &["Alice".to_string(), "42".to_string()]).unwrap();
        assert_eq!(result, "Hello, Alice! Your score is 42.");
    }
    
    #[test]
    fn test_string_dictionary_manager() {
        let mut manager = StringDictionaryManager::new();
        
        // Test default dictionary
        manager.set_string("greeting".to_string(), "Hello, {}!".to_string());
        assert_eq!(manager.get_string("greeting"), Some(&"Hello, {}!".to_string()));
        
        // Test adding a new dictionary
        let mut dict = StringDictionary::new("test");
        dict.set("farewell".to_string(), "Goodbye, {}!".to_string());
        manager.add_dictionary(dict);
        
        // Test switching dictionaries
        manager.set_current("test").unwrap();
        assert_eq!(manager.get_string("farewell"), Some(&"Goodbye, {}!".to_string()));
        assert_eq!(manager.get_string("greeting"), None);
        
        // Test formatting
        let result = manager.format_string("farewell", &["World".to_string()]).unwrap();
        assert_eq!(result, "Goodbye, World!");
    }
    
    #[test]
    fn test_error_handling() {
        let mut dict = StringDictionary::new("test");
        dict.set("greeting".to_string(), "Hello, {}!".to_string());
        
        // Test missing key
        let result = dict.format("nonexistent", &["World".to_string()]);
        assert!(result.is_err());
        
        // Test not enough arguments
        let result = dict.format("greeting", &[]);
        assert!(result.is_err());
    }
}
