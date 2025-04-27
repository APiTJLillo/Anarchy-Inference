// LSP-like Component for Anarchy Inference Language Hub Server
//
// This module implements a Language Server Protocol-like component
// that provides intelligent code editing capabilities through a
// standardized interface.

mod protocol;
mod document;
mod router;
mod parser_integration;
mod server;

pub use server::LspServer;
pub use protocol::{Request, Response, Notification, ErrorCode};
pub use document::Document;

/// Initialize and start the LSP server
pub fn start_server(host: &str, port: u16) -> Result<LspServer, String> {
    let server = LspServer::new(host, port)?;
    server.start()?;
    Ok(server)
}
