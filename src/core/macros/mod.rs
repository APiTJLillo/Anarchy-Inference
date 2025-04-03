// src/core/macros/mod.rs
// This file contains macro definitions for the language

/// Macro for defining modules
#[macro_export]
macro_rules! define_module {
    ($name:expr, $exports:expr) => {{
        let module = crate::core::module::Module::new($name, $exports);
        module
    }};
}

/// Macro for creating a garbage collected value
#[macro_export]
macro_rules! gc_value {
    ($gc:expr, $value:expr) => {{
        $gc.allocate($value)
    }};
}
