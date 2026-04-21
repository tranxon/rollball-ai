//! IPC server module

pub mod server;
pub mod transport;
pub mod session;

// Re-export SharedState for convenience
pub use server::SharedState;
