use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

/// A unique identifier for an interned string
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringId {
    /// Hash of the string
    hash: u64,
    /// Generation counter to handle invalidation
    generation: u32,
}

/// An interned string reference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedString {
    /// The string ID
    id: StringId,
}

/// Entry in the string pool
struct StringEntry {
    /// Hash of the string
    hash: u64,
    /// Reference count
    ref_count: u32,
    /// Generation when this string was interned
    generation: u32,
    /// The actual string data
    data: String,
}

/// Location of a string in a memory chunk
#[derive(Debug, Clone, Copy)]
struct StringLocation {
    /// Index of the chunk containing the string
    chunk_index: usize,
    /// Offset within the chunk
    offset: usize,
    /// Length of the string
    length: usize,
}

/// A chunk of memory for storing strings
struct StringChunk {
    /// Actual string data
    data: Vec<u8>,
    /// Free space map (offset, length)
    free_map: Vec<(usize, usize)>,
    /// Total capacity
    capacity: usize,
    /// Used space
    used: usize,
}

impl StringChunk {
    /// Create a new string chunk with the specified capacity
    fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            free_map: vec![(0, capacity)],
            capacity,
            used: 0,
        }
    }
    
    /// Allocate space for a string in this chunk
    fn allocate(&mut self, length: usize) -> Option<usize> {
        // Find a free block that can fit the string
        let mut best_fit_idx = None;
        let mut best_fit_size = usize::MAX;
        
        for (i, &(offset, size)) in self.free_map.iter().enumerate() {
            if size >= length && size < best_fit_size {
                best_fit_idx = Some(i);
                best_fit_size = size;
            }
        }
        
        if let Some(idx) = best_fit_idx {
            let (offset, size) = self.free_map[idx];
            
            // Remove this block from the free map
            self.free_map.remove(idx);
            
            // If there's leftover space, add it back to the free map
            if size > length {
                self.free_map.push((offset + length, size - length));
            }
            
            self.used += length;
            Some(offset)
        } else {
            None
        }
    }
    
    /// Free space in this chunk
    fn free(&mut self, offset: usize, length: usize) {
        self.free_map.push((offset, length));
        self.used -= length;
        
        // Merge adjacent free blocks (simple implementation)
        self.free_map.sort_by_key(|&(offset, _)| offset);
        
        let mut i = 0;
        while i < self.free_map.len() - 1 {
            let (curr_offset, curr_size) = self.free_map[i];
            let (next_offset, next_size) = self.free_map[i + 1];
            
            if curr_offset + curr_size == next_offset {
                // Merge these blocks
                self.free_map[i] = (curr_offset, curr_size + next_size);
                self.free_map.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
    
    /// Write a string to this chunk at the specified offset
    fn write_string(&mut self, offset: usize, s: &str) {
        let bytes = s.as_bytes();
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
    
    /// Read a string from this chunk
    fn read_string(&self, offset: usize, length: usize) -> String {
        let bytes = &self.data[offset..offset + length];
        String::from_utf8_lossy(bytes).to_string()
    }
    
    /// Get the utilization percentage of this chunk
    fn utilization(&self) -> f32 {
        self.used as f32 / self.capacity as f32
    }
}

/// Statistics for the string pool
#[derive(Debug, Clone, Default)]
pub struct StringPoolStats {
    /// Number of unique strings
    pub unique_strings: usize,
    /// Total string bytes stored
    pub total_bytes: usize,
    /// Memory saved through deduplication
    pub bytes_saved: usize,
    /// Number of lookups
    pub lookups: usize,
    /// Number of string comparisons
    pub comparisons: usize,
    /// Cache hits/misses
    pub cache_hits: usize,
    pub cache_misses: usize,
}

/// A pool for interning strings
pub struct StringPool {
    /// Map from string hash to string entries
    strings: HashMap<u64, Vec<StringEntry>>,
    /// Memory chunks for storing string data
    chunks: Vec<StringChunk>,
    /// Current generation counter
    generation: u32,
    /// Default chunk size
    chunk_size: usize,
    /// Statistics
    stats: StringPoolStats,
}

impl StringPool {
    /// Create a new string pool with the specified chunk size
    pub fn new(chunk_size: usize) -> Self {
        Self {
            strings: HashMap::new(),
            chunks: Vec::new(),
            generation: 0,
            chunk_size,
            stats: StringPoolStats::default(),
        }
    }
    
    /// Hash a string
    fn hash_string(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Intern a string, returning an interned string reference
    pub fn intern(&mut self, s: &str) -> InternedString {
        let hash = Self::hash_string(s);
        
        // Check if this string is already interned
        if let Some(entries) = self.strings.get_mut(&hash) {
            for entry in entries.iter_mut() {
                if entry.data == s {
                    // String already exists, increment ref count
                    entry.ref_count += 1;
                    self.stats.cache_hits += 1;
                    
                    return InternedString {
                        id: StringId {
                            hash,
                            generation: entry.generation,
                        },
                    };
                }
            }
        }
        
        // String doesn't exist, add it
        self.stats.cache_misses += 1;
        self.stats.unique_strings += 1;
        self.stats.total_bytes += s.len();
        
        // Increment generation
        self.generation = self.generation.wrapping_add(1);
        
        // Create a new entry
        let entry = StringEntry {
            hash,
            ref_count: 1,
            generation: self.generation,
            data: s.to_string(),
        };
        
        // Add to the hash map
        self.strings.entry(hash).or_insert_with(Vec::new).push(entry);
        
        InternedString {
            id: StringId {
                hash,
                generation: self.generation,
            },
        }
    }
    
    /// Look up a string by its interned reference
    pub fn lookup(&self, interned: &InternedString) -> Option<&str> {
        self.stats.lookups += 1;
        
        if let Some(entries) = self.strings.get(&interned.id.hash) {
            for entry in entries {
                if entry.generation == interned.id.generation {
                    return Some(&entry.data);
                }
                self.stats.comparisons += 1;
            }
        }
        
        None
    }
    
    /// Remove a string reference, decrementing its reference count
    pub fn remove(&mut self, interned: &InternedString) -> bool {
        if let Some(entries) = self.strings.get_mut(&interned.id.hash) {
            for i in 0..entries.len() {
                if entries[i].generation == interned.id.generation {
                    entries[i].ref_count -= 1;
                    
                    // If ref count is 0, remove the entry
                    if entries[i].ref_count == 0 {
                        let removed_len = entries[i].data.len();
                        self.stats.total_bytes -= removed_len;
                        self.stats.unique_strings -= 1;
                        entries.remove(i);
                    }
                    
                    // If the vector is empty, remove it from the hash map
                    if entries.is_empty() {
                        self.strings.remove(&interned.id.hash);
                    }
                    
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Garbage collect unused strings
    pub fn gc(&mut self) -> usize {
        let mut removed = 0;
        
        // Collect hashes with empty entry vectors
        let empty_hashes: Vec<u64> = self.strings
            .iter()
            .filter(|(_, entries)| entries.is_empty())
            .map(|(&hash, _)| hash)
            .collect();
        
        // Remove empty entry vectors
        for hash in empty_hashes {
            self.strings.remove(&hash);
            removed += 1;
        }
        
        removed
    }
    
    /// Get memory usage statistics
    pub fn stats(&self) -> &StringPoolStats {
        &self.stats
    }
}

/// Enhanced string dictionary with interning
pub struct StringDictionary {
    /// Map from interned keys to interned values
    entries: HashMap<InternedString, InternedString>,
    /// Reference to the string pool
    pool: Rc<RefCell<StringPool>>,
    /// Dictionary name
    name: String,
}

impl StringDictionary {
    /// Create a new string dictionary
    pub fn new(name: String, pool: Rc<RefCell<StringPool>>) -> Self {
        Self {
            entries: HashMap::new(),
            pool,
            name,
        }
    }
    
    /// Set a string in the dictionary
    pub fn set(&mut self, key: &str, value: &str) {
        let mut pool = self.pool.borrow_mut();
        let key_interned = pool.intern(key);
        let value_interned = pool.intern(value);
        
        // If the key already exists, remove the old value reference
        if let Some(old_value) = self.entries.get(&key_interned) {
            pool.remove(old_value);
        }
        
        self.entries.insert(key_interned, value_interned);
    }
    
    /// Get a string from the dictionary
    pub fn get(&self, key: &str) -> Option<String> {
        let mut pool = self.pool.borrow_mut();
        let key_interned = pool.intern(key);
        
        // Immediately remove our temporary reference to the key
        pool.remove(&key_interned);
        
        if let Some(value_interned) = self.entries.get(&key_interned) {
            if let Some(value) = pool.lookup(value_interned) {
                return Some(value.to_string());
            }
        }
        
        None
    }
    
    /// Remove a string from the dictionary
    pub fn remove(&mut self, key: &str) -> bool {
        let mut pool = self.pool.borrow_mut();
        let key_interned = pool.intern(key);
        
        // Immediately remove our temporary reference to the key
        pool.remove(&key_interned);
        
        if let Some(value_interned) = self.entries.remove(&key_interned) {
            pool.remove(&value_interned);
            true
        } else {
            false
        }
    }
    
    /// Get the number of entries in the dictionary
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get the name of the dictionary
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Statistics for the string dictionary manager
#[derive(Debug, Clone, Default)]
pub struct StringDictionaryStats {
    /// Number of dictionaries
    pub dictionary_count: usize,
    /// Total number of entries across all dictionaries
    pub total_entries: usize,
    /// Total bytes used for keys and values
    pub total_bytes: usize,
    /// Memory saved through interning
    pub bytes_saved: usize,
}

/// Enhanced string dictionary manager with interning
pub struct StringDictionaryManager {
    /// Map of dictionary names to dictionaries
    dictionaries: HashMap<String, StringDictionary>,
    /// Current active dictionary
    current: String,
    /// Shared string pool for interning
    string_pool: Rc<RefCell<StringPool>>,
    /// Memory usage statistics
    memory_stats: StringDictionaryStats,
}

impl StringDictionaryManager {
    /// Create a new string dictionary manager
    pub fn new(chunk_size: usize) -> Self {
        let string_pool = Rc::new(RefCell::new(StringPool::new(chunk_size)));
        
        let mut dictionaries = HashMap::new();
        let default_name = "default".to_string();
        
        dictionaries.insert(
            default_name.clone(),
            StringDictionary::new(default_name.clone(), Rc::clone(&string_pool))
        );
        
        Self {
            dictionaries,
            current: default_name,
            string_pool,
            memory_stats: StringDictionaryStats::default(),
        }
    }
    
    /// Set a string in the current dictionary
    pub fn set_string(&mut self, key: &str, value: &str) {
        if let Some(dict) = self.dictionaries.get_mut(&self.current) {
            dict.set(key, value);
            self.update_stats();
        }
    }
    
    /// Get a string from the current dictionary
    pub fn get_string(&self, key: &str) -> Option<String> {
        if let Some(dict) = self.dictionaries.get(&self.current) {
            dict.get(key)
        } else {
            None
        }
    }
    
    /// Switch to a different dictionary
    pub fn switch_dictionary(&mut self, name: &str) -> Result<(), String> {
        if self.dictionaries.contains_key(name) {
            self.current = name.to_string();
            Ok(())
        } else {
            // Create a new dictionary with this name
            let dict = StringDictionary::new(
                name.to_string(),
                Rc::clone(&self.string_pool)
            );
            
            self.dictionaries.insert(name.to_string(), dict);
            self.current = name.to_string();
            self.update_stats();
            
            Ok(())
        }
    }
    
    /// Format a string with arguments
    pub fn format_string(&self, key: &str, args: &[String]) -> Result<String, String> {
        let template = self.get_string(key)
            .ok_or_else(|| format!("String key not found: {}", key))?;
        
        // Simple placeholder replacement (assumes {} format)
        let mut result = template;
        for arg in args {
            result = result.replacen("{}", arg, 1);
        }
        
        Ok(result)
    }
    
    /// Load a dictionary from a file
    pub fn load_dictionary(&mut self, path: &str) -> Result<(), String> {
        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read dictionary file: {}", e))?;
        
        // Parse JSON
        let entries: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse dictionary file: {}", e))?;
        
        // Create a new dictionary with the parsed entries
        let dict_name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported")
            .to_string();
        
        let mut dict = StringDictionary::new(
            dict_name.clone(),
            Rc::clone(&self.string_pool)
        );
        
        // Add all entries to the dictionary
        for (key, value) in entries {
            dict.set(&key, &value);
        }
        
        // Add the dictionary
        self.dictionaries.insert(dict_name.clone(), dict);
        
        // Set as current
        self.current = dict_name;
        self.update_stats();
        
        Ok(())
    }
    
    /// Save a dictionary to a file
    pub fn save_dictionary(&self, dict_name: &str, path: &str) -> Result<(), String> {
        let dict = self.dictionaries.get(dict_name)
            .ok_or_else(|| format!("Dictionary not found: {}", dict_name))?;
        
        // Convert dictionary to a regular HashMap for serialization
        let mut entries = HashMap::new();
        
        // This is inefficient but necessary for the current implementation
        // In a real implementation, we would need to iterate over the dictionary entries
        // For now, we'll just create a dummy implementation
        for (key, value) in &dict.entries {
            if let Some(key_str) = self.string_pool.borrow().lookup(key) {
                if let Some(value_str) = self.string_pool.borrow().lookup(value) {
                    entries.insert(key_str.to_string(), value_str.to_string());
                }
            }
        }
        
        // Write to file
        std::fs::write(path, serde_json::to_string_pretty(&entries).unwrap())
            .map_err(|e| format!("Failed to save dictionary: {}", e))?;
        
        Ok(())
    }
    
    /// Perform garbage collection
    pub fn gc(&mut self) -> usize {
        let removed = self.string_pool.borrow_mut().gc();
        self.update_stats();
        removed
    }
    
    /// Get memory usage statistics
    pub fn memory_stats(&self) -> &StringDictionaryStats {
        &self.memory_stats
    }
    
    /// Update memory usage statistics
    fn update_stats(&mut self) {
        let mut stats = StringDictionaryStats::default();
        
        stats.dictionary_count = self.dictionaries.len();
        
        for dict in self.dictionaries.values() {
            stats.total_entries += dict.len();
        }
        
        let pool_stats = self.string_pool.borrow().stats();
        stats.total_bytes = pool_stats.total_bytes;
        stats.bytes_saved = pool_stats.bytes_saved;
        
        self.memory_stats = stats;
    }
}

impl fmt::Debug for StringDictionaryManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringDictionaryManager")
            .field("dictionaries", &self.dictionaries.keys().collect::<Vec<_>>())
            .field("current", &self.current)
            .field("memory_stats", &self.memory_stats)
            .finish()
    }
}

impl fmt::Debug for StringDictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringDictionary")
            .field("name", &self.name)
            .field("entries_count", &self.entries.len())
            .finish()
    }
}
