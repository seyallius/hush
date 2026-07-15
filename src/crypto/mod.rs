//! mod.rs - Handles cryptographic operations and stream cipher traits.

use crate::config::{CipherKind, Config};
use crate::crypto::x_chacha20_poly1305::ChaChaCipher;
use crate::error::VaultError;

pub mod x_chacha20_poly1305;
pub mod kdf;

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

/// Factory function to instantiate the correct StreamCipher based on the Config.
pub fn get_cipher(config: &Config, key: &[u8]) -> Box<dyn StreamCipher> {
    match config.cipher {
        CipherKind::XChaCha20Poly1305 => Box::new(ChaChaCipher::new(key)),
        CipherKind::Aes256Gcm => unimplemented!("AES-256-GCM is slated for a future release (≧◡≦)"),
    }
}