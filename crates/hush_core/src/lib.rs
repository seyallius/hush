//! lib.rs orchestrates the core business logic of hush.
//! It connects the configuration, cryptography, and envelope formats into cohesive operations.

/// The primary entry point for encrypting a stream of bytes.
/// Coordinates the envelope writer and the stream cipher to produce a `.hush` file.
pub fn encrypt_stream() -> Result<(), CoreError> {
    // Implementation will wire the EnvelopeWriter and StreamCipher together.
    Ok(())
}

/// Internal errors representing high-level business logic failures.
#[derive(Debug)]
pub enum CoreError {
    /// An underlying cryptographic operation failed.
    CryptoFailure,
    /// An underlying envelope operation failed.
    EnvelopeFailure,
}
