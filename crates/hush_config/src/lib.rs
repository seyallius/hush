//! Defines the configuration schema, defaults, validation rules, and precedence model for Hush.
//!
//! # Precedence Model
//! Configuration is resolved in the following order (highest priority wins):
//! 1. CLI Flags (e.g., `--chunk-size 2097152`)
//! 2. Environment Variables (e.g., `HUSH_CHUNK_SIZE`)
//! 3. Config File (`~/.config/hush/config.toml`)
//! 4. Hardcoded Defaults (defined in this module)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default chunk size in bytes (1 MiB).
const CHUNK_SIZE_BYTES: usize = 1_048_576;

/// Minimum allowed chunk size in bytes (64 KiB).
const CHUNK_SIZE_MIN_BYTES: usize = 65_536;

/// Maximum allowed chunk size in bytes (1 GiB).
const CHUNK_SIZE_MAX_BYTES: usize = 1_073_741_824;

/// Default Argon2id memory cost in KiB (~19 MiB).
const ARGON2_MEMORY_COST_KIB: u32 = 19_456;

/// Minimum allowed Argon2id memory cost in KiB (8 MiB).
const ARGON2_MEMORY_COST_MIN_KIB: usize = 8_192;

/// Default Argon2id time cost (iterations).
const ARGON2_TIME_COST: u32 = 2;

/// Minimum allowed Argon2id time cost.
const ARGON2_TIME_COST_MIN: u32 = 1;

/// Default Argon2id parallelism.
const ARGON2_PARALLELISM: u32 = 1;

/// Minimum allowed Argon2id parallelism.
const ARGON2_PARALLELISM_MIN: u32 = 1;

/// Fallback worker count if logical CPU detection fails.
const JOBS_FALLBACK: usize = 4;

/// Supported authenticated encryption algorithms.
///
/// XChaCha20-Poly1305 is the default for software performance and 192-bit nonce safety.
/// AES-256-GCM is provided for hardware-accelerated environments (AES-NI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "snake_case")]
pub enum CipherKind {
    /// ChaCha20 stream cipher with Poly1305 MAC (192-bit nonce).
    #[default]
    XChaCha20Poly1305,
    /// AES in Galois/Counter Mode (96-bit nonce).
    Aes256Gcm,
}

/// Supported key derivation and authentication modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "snake_case")]
pub enum KeyMode {
    /// Password only (Argon2id).
    #[default]
    Password,
    /// YubiKey challenge-response only (HKDF).
    YubiKey,
    /// Both password and YubiKey required.
    Combined,
}

/// Log verbosity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// Only fatal errors.
    Error,
    /// Warnings and errors.
    Warn,
    /// Standard operational messages.
    #[default]
    Info,
    /// Debugging information.
    Debug,
    /// Extremely verbose tracing.
    Trace,
}

/// Parameters for the Argon2id Key Derivation Function.
///
/// Defaults are based on OWASP recommendations for standard security.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Argon2Params {
    /// Memory cost in KiB. Default [`ARGON2_MEMORY_COST_KIB`].
    pub m_cost: u32,
    /// Iteration count (time cost). Default: [`ARGON2_TIME_COST`].
    pub t_cost: u32,
    /// Parallelism degree. Default: [`ARGON2_PARALLELISM`].
    pub p_cost: u32,
}
impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            m_cost: ARGON2_MEMORY_COST_KIB,
            t_cost: ARGON2_TIME_COST,
            p_cost: ARGON2_PARALLELISM,
        }
    }
}

/// The master configuration schema for Hush.
///
/// This struct maps directly to the `config.toml` file.
/// Missing fields in the TOML file will automatically fall back to `Default` values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Size of each encrypted chunk in bytes. Default: [`CHUNK_SIZE_BYTES`].
    pub chunk_size: usize,
    /// The cipher algorithm to use for encryption.
    pub cipher: CipherKind,
    /// Argon2id parameters for password-based key derivation.
    pub argon2: Argon2Params,
    /// The default key mode to prompt for if not specified via CLI.
    pub default_key_mode: KeyMode,
    /// Optional path to the VLC executable for TUI streaming.
    pub vlc_path: Option<PathBuf>,
    /// Minimum log level to output.
    pub log_level: LogLevel,
    /// Number of parallel worker threads for batch operations.
    /// Default: Number of logical CPUs.
    pub jobs: usize,
}
impl Default for Config {
    fn default() -> Self {
        let jobs = std::thread::available_parallelism().map_or(JOBS_FALLBACK, |n| n.get());

        Self {
            chunk_size: CHUNK_SIZE_BYTES,
            cipher: CipherKind::default(),
            argon2: Argon2Params::default(),
            default_key_mode: KeyMode::default(),
            vlc_path: None,
            log_level: LogLevel::default(),
            jobs,
        }
    }
}
impl Config {
    /// Validates the configuration values against strict security and operational bounds.
    ///
    /// # Validation Rules:
    /// - `chunk_size`: Must be between [`CHUNK_SIZE_MIN_BYTES`] and [`CHUNK_SIZE_MAX_BYTES`].
    /// - `argon2.m_cost`: Must be at least [`ARGON2_MEMORY_COST_MIN_KIB`].
    /// - `argon2.t_cost`: Must be at least [`ARGON2_TIME_COST_MIN`].
    /// - `argon2.p_cost`: Must be at least [`ARGON2_PARALLELISM_MIN`].
    /// - `jobs`: Must be at least 1.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.chunk_size < CHUNK_SIZE_MIN_BYTES || self.chunk_size > CHUNK_SIZE_MAX_BYTES {
            return Err(ConfigError::OutOfBounds {
                field: "chunk_size",
                min: CHUNK_SIZE_MIN_BYTES,
                max: CHUNK_SIZE_MAX_BYTES,
                actual: self.chunk_size,
            });
        }

        if (self.argon2.m_cost as usize) < ARGON2_MEMORY_COST_MIN_KIB {
            return Err(ConfigError::OutOfBounds {
                field: "argon2.m_cost",
                min: ARGON2_MEMORY_COST_MIN_KIB,
                max: usize::MAX,
                actual: self.argon2.m_cost as usize,
            });
        }

        if self.argon2.t_cost < ARGON2_TIME_COST_MIN {
            return Err(ConfigError::OutOfBounds {
                field: "argon2.t_cost",
                min: ARGON2_TIME_COST_MIN as usize,
                max: u32::MAX as usize,
                actual: self.argon2.t_cost as usize,
            });
        }

        if self.argon2.p_cost < ARGON2_PARALLELISM_MIN {
            return Err(ConfigError::OutOfBounds {
                field: "argon2.p_cost",
                min: ARGON2_PARALLELISM_MIN as usize,
                max: u32::MAX as usize,
                actual: self.argon2.p_cost as usize,
            });
        }

        if self.jobs == 0 {
            return Err(ConfigError::OutOfBounds {
                field: "jobs",
                min: 1,
                max: usize::MAX,
                actual: self.jobs,
            });
        }

        Ok(())
    }
}

/// Errors occurring during configuration validation.
///
/// Note: This is a local error type. `hush_core` will map this to `hush_core::Error::Config`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A numeric value is outside its acceptable bounds.
    OutOfBounds {
        field: &'static str,
        min: usize,
        max: usize,
        actual: usize,
    },
    /// A required path does not exist or is invalid.
    InvalidPath { field: &'static str, path: PathBuf },
}
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds {
                field,
                min,
                max,
                actual,
            } => {
                write!(
                    f,
                    "Config field '{field}' must be between {min} and {max}, but got {actual}"
                )
            }
            Self::InvalidPath { field, path } => {
                write!(
                    f,
                    "Config field '{}' points to an invalid path: {:?}",
                    field, path
                )
            }
        }
    }
}
impl std::error::Error for ConfigError {}

/// Loads the configuration from a TOML file.
///
/// If the file does not exist, returns `Ok(Config::default())`.
/// If the file exists but is malformed, returns an error.
///
/// *Implementation deferred to Issue #10.*
pub fn load_from_file(_path: &std::path::Path) -> Result<Config, ConfigError> {
    todo!("Implement TOML parsing and loading in Issue #10")
}

/// Merges environment variables into the configuration.
///
/// Environment variables override config file values but are overridden by CLI flags.
/// Expected format: `HUSH_CHUNK_SIZE=2097152`.
///
/// *Implementation deferred to Issue #38.*
pub fn merge_env(_config: &mut Config) {
    todo!("Implement environment variable merging in Issue #38")
}
