//! error.rs - Defines the custom error types for the application.

use std::{error, io};
use thiserror::Error;

/// VaultError encapsulates all possible errors that can occur in the hush application.
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Not implemented yet (≧◡≦)")]
    NotImplemented,

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Configuration error: {0}")]
    Config(String),

    /// Error originating from external crates or libraries.
    #[error("External error: {0}")]
    External(String),

    /// Error related to invalid input or data.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Error when a resource is not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Error when an operation is not permitted.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
