// Update the gc/mod.rs file to fix import issues

pub mod collector;
pub mod managed;

// Re-export key types
pub use collector::GarbageCollector;
pub use managed::GcValueImpl;
pub use crate::core::gc_types::{GcStats, GarbageCollected};
