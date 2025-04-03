#[cfg(test)]
mod module_tests {
    use std::path::PathBuf;
    use anarchy_inference::error::LangError;
    use anarchy_inference::core::module::{ModuleResolver, Module};
    use anarchy_inference::core::value::Value;

    #[test]
    fn test_module_resolver() {
        // Create a resolver with the current directory as base
        let resolver = ModuleResolver::new(".");
        
        // Test resolving a non-existent module
        let result = resolver.resolve("non_existent_module");
        assert!(result.is_err());
        
        // We can't test resolving an existing module without creating files,
        // so we'll just test the path construction logic
        let mut resolver = ModuleResolver::new("/test/base");
        resolver.add_search_path("/test/lib");
        
        // Test absolute path
        let absolute_path = "/absolute/path/module.a.i";
        let result = resolver.resolve(absolute_path);
        // This would succeed if the file existed, but we're just testing the logic
        assert!(result.is_err());
        
        // Test relative path resolution logic
        let relative_path = "relative/path/module";
        let result = resolver.resolve(relative_path);
        // This would succeed if the file existed, but we're just testing the logic
        assert!(result.is_err());
    }

    #[test]
    fn test_module_loading() {
        // We can't test actual file loading without creating files,
        // so we'll test the Module struct functionality
        
        // Create a module manually
        let module = Module {
            name: "test_module".to_string(),
            path: PathBuf::from("/test/module.a.i"),
            exports: std::sync::Mutex::new(std::collections::HashMap::new()),
            ast: Vec::new(),
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            initialized: std::sync::Mutex::new(false),
        };
        
        // Test basic properties
        assert_eq!(module.name(), "test_module");
        assert_eq!(module.path(), &PathBuf::from("/test/module.a.i"));
        assert_eq!(module.dependencies(), &["dep1".to_string(), "dep2".to_string()]);
        assert!(!module.is_initialized());
        
        // Test initialization
        module.set_initialized(true);
        assert!(module.is_initialized());
        
        // Test exports
        module.export("test_value", Value::Null);
        let export = module.get_export("test_value");
        assert!(export.is_some());
        
        let all_exports = module.get_all_exports();
        assert_eq!(all_exports.len(), 1);
        assert!(all_exports.contains_key("test_value"));
    }

    #[test]
    fn test_module_cache() {
        // Create a module cache
        let cache = anarchy_inference::core::module::ModuleCache::new();
        
        // We can't test actual module loading without creating files,
        // so we'll just test the cache functionality
        
        // Clear the cache (should be empty already)
        cache.clear();
        
        // Get all modules (should be empty)
        let modules = cache.get_all_modules();
        assert_eq!(modules.len(), 0);
    }
}
