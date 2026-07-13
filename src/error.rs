//! error.rs - Defines the custom error types for the application.

use thiserror::Error;

/// VaultError encapsulates all possible errors that can occur in the hush application.
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Not implemented yet (≧◡≦)")]
    NotImplemented,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
