//! lib.rs - Acts as the main entry point for the library modules.

/// Exposes the configuration module.
pub mod config;
/// Exposes the cryptographic operations module.
pub mod crypto;
/// Exposes the file envelope and metadata module.
pub mod envelope;
/// Exposes the custom error types.
pub mod error;
/// Exposes the progress reporting trait.
pub mod progress;
/// Exposes the streaming encrypt/decrypt orchestrator.
pub mod stream;
