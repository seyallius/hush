//! config.rs - Defines configuration structures, TOML loading,
//! CLI override merging, and the 3-layer precedence system (default < file < CLI).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use clap::ValueEnum;

/// CipherKind represents the supported encryption algorithms.
/// Swappable without touching business logic (Open/Closed Principle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CipherKind {
    #[serde(rename = "xchacha20poly1305")]
    XChaCha20Poly1305,
    #[serde(rename = "aes256gcm")]
    Aes256Gcm,
}

/// KeyMode defines how the master encryption key is derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum KeyMode {
    Password,
    YubiKey,
    Combined,
}

/// Config holds the application-wide configuration parameters.
/// All fields use serde defaults so a partial config.toml is valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Chunk size in bytes for streaming encryption (default: 1 MB).
    pub chunk_size: usize,
    /// The AEAD cipher algorithm to use.
    pub cipher: CipherKind,
    /// Argon2id memory cost in KiB.
    pub argon2_m_cost: u32,
    /// Argon2id time cost (iterations).
    pub argon2_t_cost: u32,
    /// Argon2id parallelism factor.
    pub argon2_p_cost: u32,
}

impl Default for Config {
    /// Provides secure, sensible defaults for the MVP.
    fn default() -> Self {
        Self {
            chunk_size: 1_048_576, // 1 MB
            cipher: CipherKind::XChaCha20Poly1305,
            argon2_m_cost: 19_456, // ~19 MB
            argon2_t_cost: 2,
            argon2_p_cost: 1,
        }
    }
}

impl Config {
    /// Returns the path to the hush config file: ~/.config/hush/config.toml
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("hush").join("config.toml"))
    }

    /// Loads config from the TOML file, falling back to defaults for missing fields.
    /// Returns Default config if the file doesn't exist (not an error).
    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) if path.exists() => match std::fs::read_to_string(&path) {
                Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: Failed to parse {}: {}. Using defaults.",
                        path.display(),
                        e
                    );
                    Config::default()
                }),
                Err(e) => {
                    eprintln!(
                        "Warning: Could not read {}: {}. Using defaults.",
                        path.display(),
                        e
                    );
                    Config::default()
                }
            },
            _ => Config::default(),
        }
    }

    /// Writes the default config to disk as a starting template.
    /// Returns the path where the config was written.
    pub fn save_default() -> Result<PathBuf, crate::error::VaultError> {
        let path = Self::config_path().ok_or_else(|| {
            crate::error::VaultError::Config("Cannot determine config directory".into())
        })?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).map_err(|e| {
            crate::error::VaultError::Config(format!("TOML serialization failed: {}", e))
        })?;

        let header = "\
# hush configuration file
# Docs: https://github.com/seyallius/hush
# All values shown are the defaults. Uncomment and modify as needed.

";
        std::fs::write(&path, format!("{}{}", header, toml_str))?;
        Ok(path)
    }

    /// Applies CLI overrides on top of the loaded config.
    /// Only `Some` values override; `None` means "keep whatever was loaded."
    pub fn apply_overrides(
        &mut self,
        chunk_size: Option<usize>,
        cipher: Option<CipherKind>,
        argon2_m_cost: Option<u32>,
        argon2_t_cost: Option<u32>,
        argon2_p_cost: Option<u32>,
    ) {
        if let Some(v) = chunk_size {
            self.chunk_size = v;
        }
        if let Some(v) = cipher {
            self.cipher = v;
        }
        if let Some(v) = argon2_m_cost {
            self.argon2_m_cost = v;
        }
        if let Some(v) = argon2_t_cost {
            self.argon2_t_cost = v;
        }
        if let Some(v) = argon2_p_cost {
            self.argon2_p_cost = v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = Config::default();
        assert_eq!(config.chunk_size, 1_048_576);
        assert_eq!(config.cipher, CipherKind::XChaCha20Poly1305);
        assert_eq!(config.argon2_m_cost, 19_456);
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.chunk_size, config.chunk_size);
        assert_eq!(parsed.cipher, config.cipher);
    }

    #[test]
    fn test_partial_toml_uses_defaults() {
        let partial = "chunk_size = 2097152\n";
        let config: Config = toml::from_str(partial).unwrap();
        assert_eq!(config.chunk_size, 2_097_152); // Overridden
        assert_eq!(config.cipher, CipherKind::XChaCha20Poly1305); // Default
        assert_eq!(config.argon2_m_cost, 19_456); // Default
    }

    #[test]
    fn test_cli_overrides() {
        let mut config = Config::default();
        config.apply_overrides(
            Some(4_194_304),
            Some(CipherKind::Aes256Gcm),
            None,
            None,
            None,
        );
        assert_eq!(config.chunk_size, 4_194_304);
        assert_eq!(config.cipher, CipherKind::Aes256Gcm);
        assert_eq!(config.argon2_m_cost, 19_456); // Untouched
    }
}
