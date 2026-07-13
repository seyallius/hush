//! config.rs - Defines the configuration structures and enums for the application.

use serde::{Deserialize, Serialize};

/// CipherKind represents the supported encryption algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherKind {
    XChaCha20Poly1305,
    Aes256Gcm,
}

/// Config holds the application-wide configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chunk_size: usize,
    pub cipher: CipherKind,
    pub argon2_m_cost: u32,
    pub argon2_t_cost: u32,
    pub argon2_p_cost: u32,
}

// ----------------------- Public Functions -----------------------

impl Default for Config {
    /// Provides the default configuration values for the MVP.
    fn default() -> Self {
        Self {
            chunk_size: 1_048_576, // 1 MB
            cipher: CipherKind::XChaCha20Poly1305,
            argon2_m_cost: 19_456, // ~19 MB memory
            argon2_t_cost: 2,
            argon2_p_cost: 1,
        }
    }
}

/// KeyMode defines how the master encryption key is derived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyMode {
    Password,
    YubiKey,
    Combined,
}
