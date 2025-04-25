// src/reasoning/memory_integration.rs - Memory integration for reasoning operations

use crate::error::LangError;
use crate::value::Value;
use crate::agent_memory::{
    AgentMemoryManager,
    MemorySegment,
    MemoryPriority,
    Memory
};

/// Context for accessing agent memory during reasoning
pub struct MemoryContext {
    /// Reference to the agent memory manager
    memory_manager: AgentMemoryManager,
    /// Current working memory for reasoning operations
    working_memory: Vec<Memory>,
}

impl MemoryContext {
    /// Create a new memory context
    pub fn new(memory_manager: AgentMemoryManager) -> Self {
        Self {
            memory_manager,
            working_memory: Vec::new(),
        }
    }
    
    /// Retrieve memories relevant to a query
    pub fn retrieve_relevant(&self, query: Value) -> Result<Vec<Memory>, LangError> {
        // Convert the query to a string if it's not already
        let query_str = match &query {
            Value::String(s) => s.clone(),
            _ => format!("{:?}", query),
        };
        
        // Retrieve relevant memories from different segments
        let mut relevant_memories = Vec::new();
        
        // First check working memory (highest priority)
        for memory in &self.working_memory {
            if self.is_relevant(memory, &query_str) {
                relevant_memories.push(memory.clone());
            }
        }
        
        // Then check short-term memory
        let short_term_memories = self.memory_manager.retrieve_from_segment(
            MemorySegment::ShortTerm,
            &query_str,
            10
        )?;
        relevant_memories.extend(short_term_memories);
        
        // Then check episodic memory
        let episodic_memories = self.memory_manager.retrieve_from_segment(
            MemorySegment::Episodic,
            &query_str,
            5
        )?;
        relevant_memories.extend(episodic_memories);
        
        // Finally check long-term memory
        let long_term_memories = self.memory_manager.retrieve_from_segment(
            MemorySegment::LongTerm,
            &query_str,
            5
        )?;
        relevant_memories.extend(long_term_memories);
        
        // Sort by relevance and return
        relevant_memories.sort_by(|a, b| {
            b.get_priority().partial_cmp(&a.get_priority()).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(relevant_memories)
    }
    
    /// Store a reasoning trace in memory
    pub fn store_reasoning_trace(&mut self, trace: Value) -> Result<(), LangError> {
        // Convert the trace to a string if it's not already
        let trace_str = match &trace {
            Value::String(s) => s.clone(),
            _ => format!("{:?}", trace),
        };
        
        // Create a new memory from the trace
        let memory = Memory::new(
            trace_str,
            MemorySegment::Episodic,
            MemoryPriority::Medium,
            "reasoning_trace"
        );
        
        // Store the memory
        self.memory_manager.store_memory(memory)?;
        
        Ok(())
    }
    
    /// Update working memory with new content
    pub fn update_working_memory(&mut self, content: Value) -> Result<(), LangError> {
        // Convert the content to a string if it's not already
        let content_str = match &content {
            Value::String(s) => s.clone(),
            _ => format!("{:?}", content),
        };
        
        // Create a new memory from the content
        let memory = Memory::new(
            content_str,
            MemorySegment::Working,
            MemoryPriority::High,
            "working_memory"
        );
        
        // Add to working memory
        self.working_memory.push(memory);
        
        // Limit working memory size to prevent overflow
        if self.working_memory.len() > 10 {
            self.working_memory.remove(0);
        }
        
        Ok(())
    }
    
    /// Clear working memory
    pub fn clear_working_memory(&mut self) {
        self.working_memory.clear();
    }
    
    /// Get the agent memory manager
    pub fn get_memory_manager(&self) -> &AgentMemoryManager {
        &self.memory_manager
    }
    
    /// Get a mutable reference to the agent memory manager
    pub fn get_memory_manager_mut(&mut self) -> &mut AgentMemoryManager {
        &mut self.memory_manager
    }
    
    /// Check if a memory is relevant to a query
    fn is_relevant(&self, memory: &Memory, query: &str) -> bool {
        // Simple relevance check based on string matching
        // In a real implementation, this would use more sophisticated semantic matching
        memory.get_content().contains(query)
    }
}
