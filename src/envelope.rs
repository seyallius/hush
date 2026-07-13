//! envelope.rs - Manages the custom binary file format and metadata serialization.

use crate::config::KeyMode;
use serde::{Deserialize, Serialize};

/// FileMetadata contains the sensitive metadata that will be encrypted in the file header.
#[derive(Serialize, Deserialize)]
pub struct FileMetadata {
    pub original_filename: String,
    pub mime_type: String,
    pub original_size: u64,
    pub chunk_count: u32,
    pub chunk_offsets: Vec<u64>,
}

/// FileHeader represents the unencrypted portion of the custom binary format.
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub salt: [u8; 16],
    pub argon2_params: (u32, u32, u32),
    pub yubikey_challenge: [u8; 32],
    pub key_mode: KeyMode,
    pub encrypted_metadata_len: u32,
}
