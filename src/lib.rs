pub mod ast;
pub mod value;
pub mod concurrency;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod lsp;
pub mod network;
pub mod parser;
pub mod semantic;
pub mod ui;

// Re-export UI components
pub use ui::components::*;

// Export the main App component
pub use ui::UI;
