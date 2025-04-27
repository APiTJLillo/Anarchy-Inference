# Advanced Memory Management for String Dictionaries

This document outlines the design for extending the string dictionary functionality in Anarchy Inference with advanced memory management capabilities.

## Current Implementation Analysis

The current string dictionary implementation in Anarchy Inference consists of:

1. `StringDictionary` - A simple wrapper around a HashMap that maps string keys to string values
2. `StringDictionaryManager` - Manages multiple dictionaries with operations for:
   - Getting/setting strings
   - Formatting strings with placeholders
   - Loading/saving dictionaries from/to files
   - Switching between dictionaries

While functional, the current implementation has several limitations:

1. No deduplication of string data
2. Inefficient memory usage with redundant string storage
3. No memory pooling or interning
4. Limited garbage collection integration
5. No analytics for token usage optimization

## Design Goals

The extended string dictionary functionality will address these limitations with the following goals:

1. Reduce memory usage through string interning and deduplication
2. Improve performance of string operations
3. Enhance memory management with pooling and garbage collection
4. Add analytics for token usage optimization
5. Support agent memory management requirements

## Technical Design

### 1. String Interning System

We will implement a string interning system similar to Python's approach but optimized for Anarchy Inference's needs:

```rust
pub struct InternedString {
    // The actual string data is stored once in a central pool
    id: StringId,
    // Generation counter to handle invalidation
    generation: u32,
}

pub struct StringPool {
    // Map from string hash to string data and metadata
    strings: HashMap<u64, Vec<StringEntry>>,
    // Memory chunks for storing actual string data
    chunks: Vec<StringChunk>,
    // Current generation counter
    generation: u32,
    // Statistics for analytics
    stats: StringPoolStats,
}

struct StringEntry {
    // Hash of the string
    hash: u64,
    // Reference count
    ref_count: u32,
    // Generation when this string was interned
    generation: u32,
    // Location of the string in the chunks
    location: StringLocation,
}

struct StringLocation {
    // Index of the chunk containing the string
    chunk_index: usize,
    // Offset within the chunk
    offset: usize,
    // Length of the string
    length: usize,
}

struct StringChunk {
    // Actual string data
    data: Vec<u8>,
    // Free space map
    free_map: Vec<(usize, usize)>, // (offset, length)
    // Total capacity
    capacity: usize,
    // Used space
    used: usize,
}

pub struct StringPoolStats {
    // Number of unique strings
    unique_strings: usize,
    // Total string bytes stored
    total_bytes: usize,
    // Memory saved through deduplication
    bytes_saved: usize,
    // Number of lookups
    lookups: usize,
    // Number of string comparisons
    comparisons: usize,
    // Cache hits/misses
    cache_hits: usize,
    cache_misses: usize,
}
```

### 2. Enhanced StringDictionaryManager

The existing `StringDictionaryManager` will be extended to use the string interning system:

```rust
pub struct StringDictionaryManager {
    // Map of dictionary names to dictionaries
    dictionaries: HashMap<String, StringDictionary>,
    // Current active dictionary
    current: String,
    // Shared string pool for interning
    string_pool: StringPool,
    // Memory usage statistics
    memory_stats: StringDictionaryStats,
}

pub struct StringDictionary {
    // Map from string keys to interned string values
    entries: HashMap<InternedString, InternedString>,
    // Dictionary metadata
    metadata: DictionaryMetadata,
}

pub struct DictionaryMetadata {
    // Creation timestamp
    created_at: u64,
    // Last modified timestamp
    modified_at: u64,
    // Version information
    version: String,
    // Language/locale information
    locale: Option<String>,
    // Usage statistics
    usage_stats: DictionaryUsageStats,
}

pub struct DictionaryUsageStats {
    // Number of gets/sets
    get_count: usize,
    set_count: usize,
    // Most frequently accessed keys
    top_keys: Vec<(String, usize)>,
    // Token usage statistics
    token_stats: TokenStats,
}

pub struct TokenStats {
    // Total tokens saved
    tokens_saved: usize,
    // Token usage by key
    token_usage: HashMap<String, usize>,
}
```

### 3. Memory Management Functions

We'll implement the following memory management functions:

```rust
impl StringPool {
    // Create a new string pool with specified chunk size
    pub fn new(chunk_size: usize) -> Self;
    
    // Intern a string, returning an interned string reference
    pub fn intern(&mut self, s: &str) -> InternedString;
    
    // Look up a string by its interned reference
    pub fn lookup(&self, interned: &InternedString) -> Option<&str>;
    
    // Remove a string reference, decrementing its reference count
    pub fn remove(&mut self, interned: &InternedString);
    
    // Garbage collect unused strings
    pub fn gc(&mut self) -> usize;
    
    // Compact memory to reduce fragmentation
    pub fn compact(&mut self) -> usize;
    
    // Get memory usage statistics
    pub fn stats(&self) -> &StringPoolStats;
}

impl StringDictionaryManager {
    // Enhanced versions of existing functions
    
    // Set a string with automatic interning
    pub fn set_string(&mut self, key: &str, value: &str);
    
    // Get a string with efficient lookup
    pub fn get_string(&self, key: &str) -> Option<&str>;
    
    // New memory management functions
    
    // Optimize the current dictionary for memory usage
    pub fn optimize_memory(&mut self) -> StringDictionaryStats;
    
    // Analyze token usage and suggest optimizations
    pub fn analyze_token_usage(&self) -> TokenAnalysis;
    
    // Deduplicate strings across all dictionaries
    pub fn deduplicate(&mut self) -> usize;
    
    // Perform garbage collection
    pub fn gc(&mut self) -> usize;
    
    // Get memory usage statistics
    pub fn memory_stats(&self) -> &StringDictionaryStats;
}
```

### 4. Agent Memory Management Integration

To support agent memory management, we'll add the following features:

```rust
pub struct AgentMemoryManager {
    // Reference to the string dictionary manager
    dict_manager: &mut StringDictionaryManager,
    // Agent-specific memory contexts
    contexts: HashMap<String, AgentMemoryContext>,
}

pub struct AgentMemoryContext {
    // Dictionary name for this agent
    dictionary: String,
    // Memory segments (short-term, long-term, etc.)
    segments: HashMap<String, MemorySegment>,
    // Memory usage limits
    limits: MemoryLimits,
}

pub struct MemorySegment {
    // Keys in this segment
    keys: Vec<String>,
    // Priority/importance scores
    priorities: HashMap<String, f32>,
    // Last access timestamps
    last_accessed: HashMap<String, u64>,
}

pub struct MemoryLimits {
    // Maximum number of entries
    max_entries: usize,
    // Maximum total string length
    max_bytes: usize,
    // Maximum token count
    max_tokens: usize,
}

impl AgentMemoryManager {
    // Create a new agent memory context
    pub fn create_context(&mut self, agent_id: &str) -> AgentMemoryContext;
    
    // Store a memory in a specific context and segment
    pub fn store(&mut self, agent_id: &str, segment: &str, key: &str, value: &str, priority: f32);
    
    // Retrieve a memory
    pub fn retrieve(&mut self, agent_id: &str, segment: &str, key: &str) -> Option<&str>;
    
    // Forget (remove) a memory
    pub fn forget(&mut self, agent_id: &str, segment: &str, key: &str);
    
    // Consolidate memories (move from short-term to long-term)
    pub fn consolidate(&mut self, agent_id: &str);
    
    // Prune low-priority memories when limits are reached
    pub fn prune(&mut self, agent_id: &str) -> usize;
    
    // Get memory usage statistics for an agent
    pub fn stats(&self, agent_id: &str) -> AgentMemoryStats;
}
```

## Implementation Strategy

The implementation will be phased:

1. **Phase 1**: Implement the core string interning system
   - Create the `StringPool` with basic interning functionality
   - Modify `StringDictionary` to use interned strings
   - Add memory usage statistics

2. **Phase 2**: Enhance memory management
   - Implement garbage collection
   - Add memory pooling with chunks
   - Implement compaction to reduce fragmentation

3. **Phase 3**: Add agent memory management
   - Implement the `AgentMemoryManager`
   - Add memory contexts and segments
   - Implement priority-based memory management

4. **Phase 4**: Add analytics and optimization
   - Implement token usage tracking
   - Add optimization suggestions
   - Create visualization tools for memory usage

## Performance Considerations

1. **String Hashing**: Use a fast, high-quality hash function (FNV-1a or SipHash)
2. **Memory Chunks**: Use power-of-two sized chunks for efficient allocation
3. **Cache Efficiency**: Store strings contiguously in memory chunks
4. **Concurrency**: Add thread-safe versions of key operations
5. **Benchmarking**: Create benchmarks to measure performance improvements

## Integration with Existing Code

The enhanced string dictionary functionality will be integrated with:

1. The interpreter for efficient string operations
2. The garbage collector for memory management
3. The agent system for reasoning operations
4. The token optimization system for analytics

## Testing Strategy

1. Unit tests for each component
2. Integration tests for the entire system
3. Performance benchmarks to verify improvements
4. Memory usage tests to verify reduced footprint
5. Stress tests with large dictionaries

## Documentation

1. API documentation for all new functions
2. Usage examples for common scenarios
3. Performance guidelines
4. Memory management best practices
