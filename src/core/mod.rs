// src/core/mod.rs - Core module definitions

pub mod gc_types;
pub mod macros;
pub mod module;
pub mod profiler;
pub mod string_dict;
pub mod value;
pub mod implicit_types;

pub use gc_types::*;
pub use macros::*;
pub use module::*;
pub use profiler::*;
pub use string_dict::*;
pub use value::*;
pub use implicit_types::*;
