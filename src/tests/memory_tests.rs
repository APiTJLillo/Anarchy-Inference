#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_pool::{StringPool, StringDictionaryManager};
    use crate::agent_memory::{AgentMemoryManager, MemorySegmentType};
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_string_pool_basic() {
        let mut pool = StringPool::new(1024);
        
        // Test interning
        let s1 = pool.intern("hello");
        let s2 = pool.intern("world");
        let s3 = pool.intern("hello");
        
        // Same string should return same ID
        assert_eq!(s1, s3);
        // Different strings should have different IDs
        assert_ne!(s1, s2);
        
        // Test lookup
        assert_eq!(pool.lookup(&s1), Some("hello"));
        assert_eq!(pool.lookup(&s2), Some("world"));
        
        // Test removal
        assert!(pool.remove(&s2));
        assert_eq!(pool.lookup(&s2), None);
        
        // First removal of s1/s3 should not remove the string since ref count is 2
        assert!(pool.remove(&s1));
        assert_eq!(pool.lookup(&s1), Some("hello"));
        
        // Second removal should remove the string
        assert!(pool.remove(&s3));
        assert_eq!(pool.lookup(&s3), None);
    }
    
    #[test]
    fn test_string_dictionary_basic() {
        let pool = Rc::new(RefCell::new(StringPool::new(1024)));
        let mut dict = crate::string_pool::StringDictionary::new("test".to_string(), Rc::clone(&pool));
        
        // Test setting and getting
        dict.set("key1", "value1");
        dict.set("key2", "value2");
        
        assert_eq!(dict.get("key1"), Some("value1".to_string()));
        assert_eq!(dict.get("key2"), Some("value2".to_string()));
        assert_eq!(dict.get("key3"), None);
        
        // Test removal
        assert!(dict.remove("key1"));
        assert_eq!(dict.get("key1"), None);
        assert!(!dict.remove("key3"));
    }
    
    #[test]
    fn test_string_dictionary_manager() {
        let mut manager = StringDictionaryManager::new(1024);
        
        // Test setting and getting in default dictionary
        manager.set_string("key1", "value1");
        assert_eq!(manager.get_string("key1"), Some("value1".to_string()));
        
        // Test switching dictionaries
        assert!(manager.switch_dictionary("test").is_ok());
        assert_eq!(manager.get_string("key1"), None); // Different dictionary
        
        manager.set_string("key2", "value2");
        assert_eq!(manager.get_string("key2"), Some("value2".to_string()));
        
        // Switch back to default
        assert!(manager.switch_dictionary("default").is_ok());
        assert_eq!(manager.get_string("key1"), Some("value1".to_string()));
        assert_eq!(manager.get_string("key2"), None);
    }
    
    #[test]
    fn test_string_formatting() {
        let mut manager = StringDictionaryManager::new(1024);
        
        // Set a template string
        manager.set_string("greeting", "Hello, {}! Welcome to {}.");
        
        // Format with arguments
        let result = manager.format_string("greeting", &["Alice".to_string(), "Wonderland".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Alice! Welcome to Wonderland.");
    }
    
    #[test]
    fn test_agent_memory_context() {
        let mut context = crate::agent_memory::AgentMemoryContext::new("agent1".to_string());
        
        // Test storing and retrieving
        assert!(context.store(MemorySegmentType::ShortTerm, "memory1".to_string(), "This is a test memory".to_string(), 0.8));
        
        let memory = context.retrieve(MemorySegmentType::ShortTerm, "memory1");
        assert!(memory.is_some());
        assert_eq!(memory.unwrap().content, "This is a test memory");
        
        // Test forgetting
        let removed = context.forget(MemorySegmentType::ShortTerm, "memory1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().content, "This is a test memory");
        
        let memory = context.retrieve(MemorySegmentType::ShortTerm, "memory1");
        assert!(memory.is_none());
    }
    
    #[test]
    fn test_memory_consolidation() {
        let mut context = crate::agent_memory::AgentMemoryContext::new("agent1".to_string());
        
        // Add some memories to short-term with different priorities
        context.store(MemorySegmentType::ShortTerm, "low".to_string(), "Low priority".to_string(), 0.2);
        context.store(MemorySegmentType::ShortTerm, "medium".to_string(), "Medium priority".to_string(), 0.5);
        context.store(MemorySegmentType::ShortTerm, "high".to_string(), "High priority".to_string(), 0.8);
        
        // Consolidate memories with priority >= 0.5
        let consolidated = context.consolidate(0.5);
        assert_eq!(consolidated, 2); // medium and high should be consolidated
        
        // Check that they were moved to long-term
        assert!(context.retrieve(MemorySegmentType::ShortTerm, "low").is_some());
        assert!(context.retrieve(MemorySegmentType::ShortTerm, "medium").is_none());
        assert!(context.retrieve(MemorySegmentType::ShortTerm, "high").is_none());
        
        assert!(context.retrieve(MemorySegmentType::LongTerm, "medium").is_some());
        assert!(context.retrieve(MemorySegmentType::LongTerm, "high").is_some());
    }
    
    #[test]
    fn test_memory_pruning() {
        let mut manager = AgentMemoryManager::new(1024);
        
        // Create an agent context
        assert!(manager.create_context("agent1").is_ok());
        
        // Fill short-term memory to capacity
        let segment_capacity = 100; // Default from MemoryLimits
        for i in 0..segment_capacity + 10 {
            // Use varying priorities
            let priority = (i % 10) as f32 / 10.0;
            let key = format!("memory{}", i);
            let value = format!("Memory with priority {}", priority);
            
            assert!(manager.store("agent1", MemorySegmentType::ShortTerm, &key, &value, priority).is_ok());
        }
        
        // Prune should remove some memories
        let pruned = manager.prune("agent1").unwrap();
        assert!(pruned > 0);
        
        // Check that we're below capacity
        let stats = manager.stats("agent1").unwrap();
        assert!(stats.segment_counts.get(&MemorySegmentType::ShortTerm).unwrap() <= &segment_capacity);
    }
    
    #[test]
    fn test_memory_relevance_scoring() {
        let mut entry = crate::agent_memory::MemoryEntry::new("Test memory".to_string(), 0.5);
        
        // Initial relevance should be based primarily on priority
        let initial_score = entry.relevance_score();
        
        // Record some accesses
        for _ in 0..5 {
            entry.record_access();
        }
        
        // Score should increase with access count
        let new_score = entry.relevance_score();
        assert!(new_score > initial_score);
    }
    
    #[test]
    fn test_agent_memory_manager() {
        let mut manager = AgentMemoryManager::new(1024);
        
        // Create two agent contexts
        assert!(manager.create_context("agent1").is_ok());
        assert!(manager.create_context("agent2").is_ok());
        
        // Store memories for each agent
        assert!(manager.store("agent1", MemorySegmentType::ShortTerm, "key1", "Agent 1 memory", 0.5).is_ok());
        assert!(manager.store("agent2", MemorySegmentType::ShortTerm, "key1", "Agent 2 memory", 0.5).is_ok());
        
        // Retrieve memories
        let memory1 = manager.retrieve("agent1", MemorySegmentType::ShortTerm, "key1").unwrap();
        let memory2 = manager.retrieve("agent2", MemorySegmentType::ShortTerm, "key1").unwrap();
        
        assert_eq!(memory1, "Agent 1 memory");
        assert_eq!(memory2, "Agent 2 memory");
        
        // Forget a memory
        assert!(manager.forget("agent1", MemorySegmentType::ShortTerm, "key1").is_ok());
        assert!(manager.retrieve("agent1", MemorySegmentType::ShortTerm, "key1").is_err());
        
        // Agent 2's memory should still be there
        assert_eq!(manager.retrieve("agent2", MemorySegmentType::ShortTerm, "key1").unwrap(), "Agent 2 memory");
    }
}
