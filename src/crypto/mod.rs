//! mod.rs - Handles cryptographic operations and stream cipher traits.

use crate::config::{CipherKind, Config};
use crate::crypto::x_chacha20_poly1305::ChaChaCipher;
use crate::error::VaultError;

pub mod x_chacha20_poly1305;
pub mod kdf;

/// Authenticated Encryption with Associated Data (AEAD) interface.
///
/// StreamCipher defines the interface for chunk-based authenticated encryption.
///
/// # Why AEAD?
/// Standard encryption provides confidentiality only. AEAD adds integrity:
/// if any byte is tampered with in transit/storage, decryption fails loudly.
/// No silent corruption. No bit-flip attacks.
///
/// # Nonce Safety
/// Each chunk MUST use a unique nonce. Reusing a nonce with the same key
/// completely breaks ChaCha20-Poly1305 security. Our format stores a fresh
/// random 24-byte nonce per chunk, making reuse astronomically unlikely.
///
/// # Implementors
/// - [`ChaChaCipher`](x_chacha20_poly1305::ChaChaCipher) — XChaCha20-Poly1305 (default)
/// - AES-256-GCM — Planned for v1.0
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