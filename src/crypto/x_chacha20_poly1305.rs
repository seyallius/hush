use crate::{crypto::StreamCipher, error::VaultError};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};

/// ChaChaCipher implements the [StreamCipher] trait using XChaCha20-Poly1305.
pub struct ChaChaCipher {
    cipher: XChaCha20Poly1305,
}

impl ChaChaCipher {
    /// Creates a new ChaChaCipher instance with the provided 32-byte key.
    pub fn new(key: &[u8]) -> Self {
        let cipher = XChaCha20Poly1305::new_from_slice(key)
            .expect("Invalid key length for XChaCha20Poly1305");
        Self { cipher }
    }
}
impl StreamCipher for ChaChaCipher {
    /// Encrypts a single chunk of plaintext using XChaCha20-Poly1305.
    fn encrypt_chunk(&self, plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, VaultError> {
        if nonce.len() != self.nonce_size() {
            return Err(VaultError::Encryption(format!(
                "Invalid nonce size: expected {}, got {}",
                self.nonce_size(),
                nonce.len()
            )));
        }

        let xnonce = XNonce::from_slice(nonce);
        self.cipher
            .encrypt(xnonce, plaintext)
            .map_err(|e| VaultError::Encryption(e.to_string()))
    }

    /// Decrypts a single chunk of ciphertext using XChaCha20-Poly1305.
    fn decrypt_chunk(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, VaultError> {
        if nonce.len() != self.nonce_size() {
            return Err(VaultError::Decryption(format!(
                "Invalid nonce size: expected {}, got {}",
                self.nonce_size(),
                nonce.len()
            )));
        }

        let xnonce = XNonce::from_slice(nonce);
        self.cipher
            .decrypt(xnonce, ciphertext)
            .map_err(|e| VaultError::Decryption(e.to_string()))
    }

    /// Returns the required nonce size for XChaCha20 (24 bytes).
    fn nonce_size(&self) -> usize {
        24
    }

    /// Returns the Poly1305 authentication tag size (16 bytes).
    fn tag_size(&self) -> usize {
        16
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, crypto::get_cipher};
    use rand::RngCore;

    #[test]
    fn test_chacha_chunk_roundtrip() {
        // Generate a random 32-byte key
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);

        let config = Config::default();
        let cipher = get_cipher(&config, &key);

        let plaintext = b"Hello, Hush! This is a secret video chunk.";

        // Generate a random nonce of the correct size
        let mut nonce = vec![0u8; cipher.nonce_size()];
        rand::thread_rng().fill_bytes(&mut nonce);

        // Encrypt
        let ciphertext = cipher.encrypt_chunk(plaintext, &nonce).unwrap();
        assert_ne!(plaintext.as_slice(), ciphertext.as_slice());
        assert_eq!(ciphertext.len(), plaintext.len() + cipher.tag_size());

        // Decrypt
        let decrypted = cipher.decrypt_chunk(&ciphertext, &nonce).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
}
