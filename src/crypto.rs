//! crypto.rs - Handles cryptographic operations and stream cipher traits.

use crate::error::VaultError;

/// StreamCipher defines the interface for chunk-based authenticated encryption.
pub trait StreamCipher: Send + Sync {
    /// Encrypts a single chunk of plaintext. Returns the ciphertext + auth tag.
    fn encrypt_chunk(&self, plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, VaultError>;

    /// Decrypts a single chunk. Returns plaintext.
    fn decrypt_chunk(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, VaultError>;

    /// Required nonce size for this cipher.
    fn nonce_size(&self) -> usize;

    /// Required tag size for this cipher.
    fn tag_size(&self) -> usize;
}
