# String Dictionary and Agent Memory Management

This document provides documentation for the newly implemented advanced memory management features in Anarchy Inference:

1. String Dictionary Extensions with String Interning
2. Agent Memory Management System

## String Dictionary Extensions

### Overview

The string dictionary system has been extended with advanced memory management capabilities through string interning and pooling. This significantly reduces memory usage by storing only one copy of each unique string, while also improving performance of string operations through efficient comparison and lookup.

### Key Components

#### StringPool

The `StringPool` is the core component responsible for string interning:

- Stores unique strings in memory and provides references to them
- Uses reference counting to track string usage
- Provides garbage collection to reclaim memory from unused strings
- Maintains statistics about memory usage and string operations

```rust
// Create a new string pool with a chunk size of 16KB
let pool = StringPool::new(16 * 1024);

// Intern strings
let hello_id = pool.intern("hello");
let world_id = pool.intern("world");
let hello_again_id = pool.intern("hello");

// hello_id and hello_again_id are equal
assert_eq!(hello_id, hello_again_id);

// Look up strings by their ID
assert_eq!(pool.lookup(&hello_id), Some("hello"));

// Remove strings when no longer needed
pool.remove(&hello_id);
```

#### StringDictionary

The `StringDictionary` uses the `StringPool` to efficiently store key-value pairs:

- Keys and values are both interned in the string pool
- Provides efficient lookup and modification operations
- Shares the string pool with other dictionaries to maximize memory efficiency

```rust
// Create a string dictionary with a shared pool
let pool = Rc::new(RefCell::new(StringPool::new(1024)));
let mut dict = StringDictionary::new("my_dict", Rc::clone(&pool));

// Set key-value pairs
dict.set("key1", "value1");
dict.set("key2", "value2");

// Get values
assert_eq!(dict.get("key1"), Some("value1".to_string()));

// Remove entries
dict.remove("key1");
```

#### StringDictionaryManager

The `StringDictionaryManager` manages multiple dictionaries:

- Maintains a collection of named dictionaries
- Provides a shared string pool for all dictionaries
- Supports switching between dictionaries
- Offers string formatting and dictionary loading/saving

```rust
// Create a dictionary manager
let mut manager = StringDictionaryManager::new(1024);

// Set strings in the default dictionary
manager.set_string("greeting", "Hello, {}!");

// Format strings with arguments
let result = manager.format_string("greeting", &["World".to_string()]);
assert_eq!(result.unwrap(), "Hello, World!");

// Switch to a different dictionary
manager.switch_dictionary("another_dict").unwrap();
```

### Memory Management Features

1. **String Interning**: Stores only one copy of each unique string
2. **Reference Counting**: Tracks string usage to know when strings can be freed
3. **Garbage Collection**: Reclaims memory from unused strings
4. **Memory Statistics**: Provides insights into memory usage and efficiency

## Agent Memory Management

### Overview

The Agent Memory Management system provides a structured approach to managing memory for AI agents. It organizes memories into different segments based on their purpose and importance, and provides mechanisms for storing, retrieving, consolidating, and pruning memories.

### Key Components

#### MemorySegmentType

Defines different types of memory segments:

- `ShortTerm`: Recent interactions and temporary information
- `Working`: Current task context and active information
- `LongTerm`: Persistent knowledge and important information
- `Episodic`: Specific experiences and events
- `Semantic`: General knowledge and concepts

#### MemoryEntry

Represents a single memory with metadata:

- `content`: The actual memory content
- `priority`: Importance score (0.0 to 1.0)
- `created_at`: Creation timestamp
- `last_accessed`: Last access timestamp
- `access_count`: Number of times accessed
- `tags`: Categories or labels for the memory

```rust
// Create a new memory entry
let memory = MemoryEntry::new("Important information".to_string(), 0.8);

// Add tags
memory.add_tag("important".to_string());
memory.add_tag("information".to_string());

// Record access
memory.record_access();

// Calculate relevance score (combines priority, recency, and frequency)
let score = memory.relevance_score();
```

#### MemorySegment

Contains related memories:

- Organizes memories by segment type
- Enforces capacity limits
- Provides priority-based eviction
- Supports searching and sorting memories

```rust
// Create a memory segment
let mut segment = MemorySegment::new(MemorySegmentType::ShortTerm, 100);

// Add memories
segment.add("key1".to_string(), memory_entry);

// Get a memory (also updates access stats)
let memory = segment.get("key1");

// Get all memories sorted by relevance
let sorted_memories = segment.get_all_sorted();

// Search memories by tag
let tagged_memories = segment.search_by_tag("important");
```

#### AgentMemoryContext

Manages memory for a single agent:

- Contains multiple memory segments
- Enforces memory limits
- Provides operations for storing, retrieving, and managing memories
- Supports memory consolidation (moving from short-term to long-term)

```rust
// Create an agent memory context
let mut context = AgentMemoryContext::new("agent1".to_string());

// Store a memory in a specific segment
context.store(MemorySegmentType::ShortTerm, "key1".to_string(), 
              "Important memory".to_string(), 0.8);

// Retrieve a memory
let memory = context.retrieve(MemorySegmentType::ShortTerm, "key1");

// Move a memory between segments
context.move_memory(MemorySegmentType::ShortTerm, 
                   MemorySegmentType::LongTerm, "key1");

// Consolidate high-priority memories from short-term to long-term
let consolidated = context.consolidate(0.7);
```

#### AgentMemoryManager

Manages memory for multiple agents:

- Creates and manages agent memory contexts
- Integrates with the string dictionary system
- Provides operations for storing, retrieving, and managing memories
- Supports memory consolidation and pruning

```rust
// Create an agent memory manager
let mut manager = AgentMemoryManager::new(1024);

// Create agent contexts
manager.create_context("agent1").unwrap();
manager.create_context("agent2").unwrap();

// Store memories for different agents
manager.store("agent1", MemorySegmentType::ShortTerm, 
              "key1", "Agent 1 memory", 0.5).unwrap();
manager.store("agent2", MemorySegmentType::ShortTerm, 
              "key1", "Agent 2 memory", 0.5).unwrap();

// Retrieve memories
let memory1 = manager.retrieve("agent1", MemorySegmentType::ShortTerm, "key1").unwrap();
let memory2 = manager.retrieve("agent2", MemorySegmentType::ShortTerm, "key1").unwrap();

// Consolidate memories
manager.consolidate("agent1", 0.7).unwrap();

// Prune low-priority memories when at capacity
manager.prune("agent1").unwrap();

// Get memory statistics
let stats = manager.stats("agent1").unwrap();
```

### Memory Management Features

1. **Segmented Memory**: Organizes memories by type and purpose
2. **Priority-Based Management**: Prioritizes important memories
3. **Memory Consolidation**: Moves important memories from short-term to long-term
4. **Memory Pruning**: Removes low-priority memories when capacity is reached
5. **Relevance Scoring**: Calculates memory importance based on priority, recency, and frequency
6. **Memory Statistics**: Provides insights into memory usage and efficiency

## Integration with Anarchy Inference

These new components integrate with Anarchy Inference in the following ways:

1. The string dictionary extensions enhance the existing string dictionary system with advanced memory management
2. The agent memory management system provides a foundation for agent reasoning operations
3. Both systems work together to provide efficient memory management for AI agents

## Performance Considerations

1. **Memory Efficiency**: String interning significantly reduces memory usage by eliminating duplicate strings
2. **Comparison Performance**: Interned strings can be compared by reference instead of content
3. **Garbage Collection**: Automatic memory reclamation reduces memory leaks
4. **Priority-Based Eviction**: Ensures the most important memories are retained when capacity is reached
5. **Relevance Scoring**: Balances priority, recency, and frequency to determine memory importance

## Best Practices

1. **String Interning**: Use string interning for strings that are frequently compared or have many duplicates
2. **Memory Segmentation**: Organize agent memories into appropriate segments based on their purpose
3. **Memory Consolidation**: Regularly consolidate important short-term memories to long-term memory
4. **Memory Pruning**: Periodically prune low-priority memories to stay within memory limits
5. **Memory Statistics**: Monitor memory usage and efficiency to identify optimization opportunities
