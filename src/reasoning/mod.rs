// src/reasoning/mod.rs - Module definition for reasoning operations

mod engine;
mod strategies;
mod planning;
mod memory_integration;
mod tool_integration;

pub use engine::ReasoningEngine;
pub use strategies::{
    ReasoningStrategy, 
    ReasoningType,
    ConditionalReasoning,
    HeuristicReasoning,
    ReActReasoning,
    SelfReflectionReasoning,
    MultiAgentReasoning
};
pub use planning::{Plan, PlanStep, PlanStatus, StepStatus};
pub use memory_integration::MemoryContext;
pub use tool_integration::ToolManager;

// Re-export common types and functions for easier access
pub mod prelude {
    pub use super::ReasoningEngine;
    pub use super::ReasoningStrategy;
    pub use super::ReasoningType;
    pub use super::Plan;
    pub use super::MemoryContext;
    pub use super::ToolManager;
}
