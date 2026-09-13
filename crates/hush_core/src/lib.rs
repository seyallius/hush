//! Root crate for hush_core, exposing the shared error model and future domain contracts.

/// Error taxonomy shared by CLI, TUI, and core domain logic.
pub mod error;

// Re-export the most common error items so callers can use `hush_core::Error`.
pub use error::{BoxError, Error, ExitCode, KeyError, Result};
