//! envelope.rs - Manages the custom binary file format and metadata serialization.

use crate::{config::KeyMode, error::VaultError};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Magic bytes to identify hush encrypted files ("VAUL").
pub const MAGIC_BYTES: [u8; 4] = [0x56, 0x41, 0x55, 0x4C];
/// Current version of the hush file format.
pub const CURRENT_VERSION: u8 = 1;

/// FileMetadata contains the sensitive metadata that will be encrypted in the file header.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub original_filename: String,
    pub mime_type: String,
    pub original_size: u64,
    pub chunk_count: u32,
    /// The byte offset in the file where each encrypted chunk starts.
    pub chunk_offsets: Vec<u64>,
}

/// FileHeader represents the unencrypted portion of the custom binary format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub salt: [u8; 16],
    pub argon2_params: (u32, u32, u32),
    pub yubikey_challenge: [u8; 32],
    pub key_mode: KeyMode,
}

impl FileHeader {
    /// Creates a new FileHeader with default magic and version.
    pub fn new(
        salt: [u8; 16],
        argon2_params: (u32, u32, u32),
        yubikey_challenge: [u8; 32],
        key_mode: KeyMode,
    ) -> Self {
        Self {
            magic: MAGIC_BYTES,
            version: CURRENT_VERSION,
            salt,
            argon2_params,
            yubikey_challenge,
            key_mode,
        }
    }

    /// Serializes the header to bytes using bincode.
    pub fn to_bytes(&self) -> Result<Vec<u8>, VaultError> {
        bincode::serialize(self)
            .map_err(|e| VaultError::Encryption(format!("Header serialization failed: {}", e)))
    }

    /// Deserializes the header from bytes and validates magic/version.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VaultError> {
        let header: Self = bincode::deserialize(bytes)
            .map_err(|e| VaultError::Decryption(format!("Header deserialization failed: {}", e)))?;

        if header.magic != MAGIC_BYTES {
            return Err(VaultError::Decryption(
                "Invalid magic bytes. Not a hush file.".to_string(),
            ));
        }
        if header.version != CURRENT_VERSION {
            return Err(VaultError::Decryption(format!(
                "Unsupported version: {}",
                header.version
            )));
        }

        Ok(header)
    }

    /// Writes the header to a writer, prefixed with its u32 length.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), VaultError> {
        let bytes = self.to_bytes()?;
        let len = bytes.len() as u32;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Reads the header from a reader safely.
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, VaultError> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Sanity check to prevent Out-Of-Memory attacks from malicious files
        if len > 1024 * 1024 {
            return Err(VaultError::Decryption(
                "Header size exceeds 1MB sanity limit.".to_string(),
            ));
        }

        let mut bytes = vec![0u8; len];
        reader.read_exact(&mut bytes)?;

        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_roundtrip() {
        let salt = [1u8; 16];
        let params = (19456, 2, 1);
        let challenge = [2u8; 32];
        let mode = KeyMode::Password;

        let header = FileHeader::new(salt, params, challenge, mode);

        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let parsed = FileHeader::read_from(&mut cursor).unwrap();

        assert_eq!(parsed.magic, MAGIC_BYTES);
        assert_eq!(parsed.salt, salt);
        assert_eq!(parsed.key_mode, mode);
    }
}
