//! lib.rs establishes the cryptographic traits and boundaries for hush.
//! It abstracts the underlying AEAD implementations so the rest of the app remains cipher-agnostic.

/// The core trait for streaming authenticated encryption.
/// Any cipher we support (XChaCha20, AES-GCM) must implement this contract.
pub trait StreamCipher: Send + Sync {
    /// Encrypts a single chunk of plaintext with a given nonce.
    fn encrypt_chunk(&self, plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// Decrypts a single chunk, verifying the authentication tag.
    fn decrypt_chunk(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// Returns the required nonce size in bytes for this cipher.
    fn nonce_size(&self) -> usize;
}

/// Internal error type for cryptographic operations.
#[derive(Debug)]
pub enum CryptoError {
    /// The authentication tag did not match (tampered data or wrong key).
    IntegrityFailure,
    /// The provided nonce or ciphertext was of an invalid length.
    InvalidLength,
}
