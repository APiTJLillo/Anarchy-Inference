use std::collections::HashMap;
use crate::string_pool::{StringDictionaryManager, StringPool};

/// Agent memory segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemorySegmentType {
    /// Short-term memory (recent interactions)
    ShortTerm,
    /// Working memory (current task context)
    Working,
    /// Long-term memory (persistent knowledge)
    LongTerm,
    /// Episodic memory (specific experiences)
    Episodic,
    /// Semantic memory (general knowledge)
    Semantic,
}

impl MemorySegmentType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ShortTerm => "short_term",
            Self::Working => "working",
            Self::LongTerm => "long_term",
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "short_term" => Some(Self::ShortTerm),
            "working" => Some(Self::Working),
            "long_term" => Some(Self::LongTerm),
            "episodic" => Some(Self::Episodic),
            "semantic" => Some(Self::Semantic),
            _ => None,
        }
    }
}

/// Memory entry with metadata
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// The memory content
    pub content: String,
    /// Priority/importance score (0.0 to 1.0)
    pub priority: f32,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Access count
    pub access_count: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(content: String, priority: f32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            content,
            priority: priority.max(0.0).min(1.0), // Clamp to 0.0-1.0
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
        }
    }
    
    /// Record an access to this memory
    pub fn record_access(&mut self) {
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        self.access_count += 1;
    }
    
    /// Add a tag to this memory
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Calculate the current relevance score based on recency and priority
    pub fn relevance_score(&self) -> f32 {
        // Current time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Recency factor (decays over time)
        let time_diff = now.saturating_sub(self.last_accessed) as f32;
        let recency = 1.0 / (1.0 + 0.01 * time_diff); // Simple decay function
        
        // Frequency factor (increases with access count)
        let frequency = (1.0 + self.access_count as f32).ln();
        
        // Combined score
        0.5 * self.priority + 0.3 * recency + 0.2 * frequency
    }
}

/// Memory segment containing related memories
#[derive(Debug)]
pub struct MemorySegment {
    /// Type of memory segment
    pub segment_type: MemorySegmentType,
    /// Memories in this segment
    pub memories: HashMap<String, MemoryEntry>,
    /// Maximum capacity
    pub capacity: usize,
}

impl MemorySegment {
    /// Create a new memory segment
    pub fn new(segment_type: MemorySegmentType, capacity: usize) -> Self {
        Self {
            segment_type,
            memories: HashMap::new(),
            capacity,
        }
    }
    
    /// Add a memory to this segment
    pub fn add(&mut self, key: String, entry: MemoryEntry) -> bool {
        // Check if we're at capacity
        if self.memories.len() >= self.capacity && !self.memories.contains_key(&key) {
            // Need to evict a memory
            self.evict_lowest_priority();
        }
        
        self.memories.insert(key, entry);
        true
    }
    
    /// Get a memory from this segment
    pub fn get(&mut self, key: &str) -> Option<&MemoryEntry> {
        if let Some(entry) = self.memories.get_mut(key) {
            entry.record_access();
        }
        
        self.memories.get(key)
    }
    
    /// Remove a memory from this segment
    pub fn remove(&mut self, key: &str) -> Option<MemoryEntry> {
        self.memories.remove(key)
    }
    
    /// Evict the lowest priority memory
    fn evict_lowest_priority(&mut self) -> Option<(String, MemoryEntry)> {
        if self.memories.is_empty() {
            return None;
        }
        
        // Find the key with the lowest relevance score
        let mut lowest_key = None;
        let mut lowest_score = f32::MAX;
        
        for (key, entry) in &self.memories {
            let score = entry.relevance_score();
            if score < lowest_score {
                lowest_score = score;
                lowest_key = Some(key.clone());
            }
        }
        
        // Remove and return the lowest priority memory
        lowest_key.and_then(|key| self.memories.remove(&key).map(|entry| (key, entry)))
    }
    
    /// Get all memories sorted by relevance
    pub fn get_all_sorted(&self) -> Vec<(&String, &MemoryEntry)> {
        let mut memories: Vec<(&String, &MemoryEntry)> = self.memories.iter().collect();
        
        // Sort by relevance score (descending)
        memories.sort_by(|(_, a), (_, b)| {
            b.relevance_score().partial_cmp(&a.relevance_score()).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        memories
    }
    
    /// Search memories by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<(&String, &MemoryEntry)> {
        self.memories
            .iter()
            .filter(|(_, entry)| entry.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// Get the current utilization percentage
    pub fn utilization(&self) -> f32 {
        self.memories.len() as f32 / self.capacity as f32
    }
}

/// Memory limits for an agent
#[derive(Debug, Clone)]
pub struct MemoryLimits {
    /// Maximum entries per segment
    pub segment_capacities: HashMap<MemorySegmentType, usize>,
    /// Maximum total entries across all segments
    pub total_capacity: usize,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        let mut segment_capacities = HashMap::new();
        segment_capacities.insert(MemorySegmentType::ShortTerm, 100);
        segment_capacities.insert(MemorySegmentType::Working, 50);
        segment_capacities.insert(MemorySegmentType::LongTerm, 1000);
        segment_capacities.insert(MemorySegmentType::Episodic, 500);
        segment_capacities.insert(MemorySegmentType::Semantic, 1000);
        
        Self {
            segment_capacities,
            total_capacity: 2000,
        }
    }
}

/// Memory context for an agent
#[derive(Debug)]
pub struct AgentMemoryContext {
    /// Dictionary name for this agent
    pub dictionary: String,
    /// Memory segments
    pub segments: HashMap<MemorySegmentType, MemorySegment>,
    /// Memory limits
    pub limits: MemoryLimits,
}

impl AgentMemoryContext {
    /// Create a new agent memory context
    pub fn new(dictionary: String) -> Self {
        let limits = MemoryLimits::default();
        let mut segments = HashMap::new();
        
        // Create segments with default capacities
        for (&segment_type, &capacity) in &limits.segment_capacities {
            segments.insert(segment_type, MemorySegment::new(segment_type, capacity));
        }
        
        Self {
            dictionary,
            segments,
            limits,
        }
    }
    
    /// Store a memory in a specific segment
    pub fn store(&mut self, segment_type: MemorySegmentType, key: String, content: String, priority: f32) -> bool {
        let entry = MemoryEntry::new(content, priority);
        
        if let Some(segment) = self.segments.get_mut(&segment_type) {
            segment.add(key, entry)
        } else {
            // Create the segment if it doesn't exist
            let capacity = self.limits.segment_capacities
                .get(&segment_type)
                .copied()
                .unwrap_or(100);
                
            let mut segment = MemorySegment::new(segment_type, capacity);
            let result = segment.add(key, entry);
            self.segments.insert(segment_type, segment);
            result
        }
    }
    
    /// Retrieve a memory from a specific segment
    pub fn retrieve(&mut self, segment_type: MemorySegmentType, key: &str) -> Option<&MemoryEntry> {
        self.segments.get_mut(&segment_type)?.get(key)
    }
    
    /// Forget (remove) a memory from a specific segment
    pub fn forget(&mut self, segment_type: MemorySegmentType, key: &str) -> Option<MemoryEntry> {
        self.segments.get_mut(&segment_type)?.remove(key)
    }
    
    /// Move a memory from one segment to another
    pub fn move_memory(&mut self, from_type: MemorySegmentType, to_type: MemorySegmentType, key: &str) -> bool {
        // Get the memory from the source segment
        let entry = match self.forget(from_type, key) {
            Some(entry) => entry,
            None => return false,
        };
        
        // Store it in the destination segment
        self.store(to_type, key.to_string(), entry.content, entry.priority)
    }
    
    /// Consolidate memories (move from short-term to long-term based on priority)
    pub fn consolidate(&mut self, threshold: f32) -> usize {
        let mut consolidated = 0;
        
        // Get all memories from short-term memory with priority above threshold
        let to_consolidate: Vec<(String, MemoryEntry)> = {
            if let Some(short_term) = self.segments.get(&MemorySegmentType::ShortTerm) {
                short_term.memories
                    .iter()
                    .filter(|(_, entry)| entry.priority >= threshold)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                Vec::new()
            }
        };
        
        // Move each memory to long-term memory
        for (key, entry) in to_consolidate {
            if self.forget(MemorySegmentType::ShortTerm, &key).is_some() {
                if self.store(MemorySegmentType::LongTerm, key, entry.content, entry.priority) {
                    consolidated += 1;
                }
            }
        }
        
        consolidated
    }
    
    /// Get total memory count
    pub fn total_memories(&self) -> usize {
        self.segments.values().map(|segment| segment.memories.len()).sum()
    }
    
    /// Check if total memory is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.total_memories() >= self.limits.total_capacity
    }
}

/// Statistics for agent memory
#[derive(Debug, Clone, Default)]
pub struct AgentMemoryStats {
    /// Number of memories per segment
    pub segment_counts: HashMap<MemorySegmentType, usize>,
    /// Total memories
    pub total_memories: usize,
    /// Average priority
    pub avg_priority: f32,
    /// Memory utilization percentage
    pub utilization: f32,
}

/// Agent memory manager
pub struct AgentMemoryManager {
    /// Reference to the string dictionary manager
    dict_manager: StringDictionaryManager,
    /// Agent-specific memory contexts
    contexts: HashMap<String, AgentMemoryContext>,
}

impl AgentMemoryManager {
    /// Create a new agent memory manager
    pub fn new(chunk_size: usize) -> Self {
        Self {
            dict_manager: StringDictionaryManager::new(chunk_size),
            contexts: HashMap::new(),
        }
    }
    
    /// Create a new agent memory context
    pub fn create_context(&mut self, agent_id: &str) -> Result<(), String> {
        if self.contexts.contains_key(agent_id) {
            return Err(format!("Agent context already exists: {}", agent_id));
        }
        
        // Create a dictionary for this agent
        let dict_name = format!("agent_{}", agent_id);
        self.dict_manager.switch_dictionary(&dict_name)?;
        
        // Create a memory context
        let context = AgentMemoryContext::new(dict_name);
        self.contexts.insert(agent_id.to_string(), context);
        
        Ok(())
    }
    
    /// Store a memory in a specific context and segment
    pub fn store(&mut self, agent_id: &str, segment: MemorySegmentType, key: &str, value: &str, priority: f32) -> Result<(), String> {
        let context = self.contexts.get_mut(agent_id)
            .ok_or_else(|| format!("Agent context not found: {}", agent_id))?;
            
        // Switch to the agent's dictionary
        self.dict_manager.switch_dictionary(&context.dictionary)?;
        
        // Store the memory in the string dictionary
        self.dict_manager.set_string(key, value);
        
        // Store the memory in the agent's context
        context.store(segment, key.to_string(), value.to_string(), priority);
        
        Ok(())
    }
    
    /// Retrieve a memory
    pub fn retrieve(&mut self, agent_id: &str, segment: MemorySegmentType, key: &str) -> Result<String, String> {
        let context = self.contexts.get_mut(agent_id)
            .ok_or_else(|| format!("Agent context not found: {}", agent_id))?;
            
        // Switch to the agent's dictionary
        self.dict_manager.switch_dictionary(&context.dictionary)?;
        
        // Try to get the memory from the context first (to update access stats)
        context.retrieve(segment, key);
        
        // Get the actual value from the dictionary
        self.dict_manager.get_string(key)
            .ok_or_else(|| format!("Memory not found: {}", key))
    }
    
    /// Forget (remove) a memory
    pub fn forget(&mut self, agent_id: &str, segment: MemorySegmentType, key: &str) -> Result<(), String> {
        let context = self.contexts.get_mut(agent_id)
            .ok_or_else(|| format!("Agent context not found: {}", agent_id))?;
            
        // Remove from the context
        if context.forget(segment, key).is_none() {
            return Err(format!("Memory not found in segment: {}", key));
        }
        
        // We don't remove from the dictionary because other segments might reference the same key
        // The garbage collection will handle this
        
        Ok(())
    }
    
    /// Consolidate memories (move from short-term to long-term)
    pub fn consolidate(&mut self, agent_id: &str, threshold: f32) -> Result<usize, String> {
        let context = self.contexts.get_mut(agent_id)
            .ok_or_else(|| format!("Agent context not found: {}", agent_id))?;
            
        Ok(context.consolidate(threshold))
    }
    
    /// Prune low-priority memories when limits are reached
    pub fn prune(&mut self, agent_id: &str) -> Result<usize, String> {
        let context = self.contexts.get_mut(agent_id)
            .ok_or_else(|| format!("Agent context not found: {}", agent_id))?;
            
        if !context.is_at_capacity() {
            return Ok(0);
        }
        
        let mut pruned = 0;
        
        // Prune from short-term memory first
        if let Some(segment) = context.segments.get_mut(&MemorySegmentType::ShortTerm) {
            while segment.memories.len() > segment.capacity / 2 {
                if segment.evict_lowest_priority().is_some() {
                    pruned += 1;
                } else {
                    break;
                }
            }
        }
        
        // If still at capacity, prune from other segments
        if context.is_at_capacity() {
            for segment_type in [
                MemorySegmentType::Working,
                MemorySegmentType::Episodic,
                MemorySegmentType::Semantic,
                MemorySegmentType::LongTerm,
            ] {
                if let Some(segment) = context.segments.get_mut(&segment_type) {
                    while context.is_at_capacity() && segment.memories.len() > 0 {
                        if segment.evict_lowest_priority().is_some() {
                            pruned += 1;
                        } else {
                            break;
                        }
                    }
                }
                
                if !context.is_at_capacity() {
                    break;
                }
            }
        }
        
        // Perform garbage collection on the dictionary
        self.dict_manager.gc();
        
        Ok(pruned)
    }
    
    /// Get memory usage statistics for an agent
    pub fn stats(&self, agent_id: &str) -> Result<AgentMemoryStats, String> {
        let context = self.contexts.get(agent_id)
            .ok_or_else(|| format!("Agent context not found: {}", agent_id))?;
            
        let mut stats = AgentMemoryStats::default();
        let mut total_priority = 0.0;
        let mut memory_count = 0;
        
        for (segment_type, segment) in &context.segments {
            let count = segment.memories.len();
            stats.segment_counts.insert(*segment_type, count);
            stats.total_memories += count;
            
            // Calculate average priority
            for (_, entry) in &segment.memories {
                total_priority += entry.priority;
                memory_count += 1;
            }
            
            // Calculate utilization
            let capacity = context.limits.segment_capacities.get(segment_type).copied().unwrap_or(100);
            let utilization = count as f32 / capacity as f32;
            stats.utilization = stats.utilization.max(utilization);
        }
        
        if memory_count > 0 {
            stats.avg_priority = total_priority / memory_count as f32;
        }
        
        Ok(stats)
    }
    
    /// Get the string dictionary manager
    pub fn get_dict_manager(&self) -> &StringDictionaryManager {
        &self.dict_manager
    }
    
    /// Get a mutable reference to the string dictionary manager
    pub fn get_dict_manager_mut(&mut self) -> &mut StringDictionaryManager {
        &mut self.dict_manager
    }
}
