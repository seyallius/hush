//! kdf.rs - Handles "Key Derivation Functions" like Argon2id.

use crate::{
    config::Config,
    error::VaultError
};
use argon2::Argon2;

/// Derives a 32-byte master key from a password and salt using Argon2id.
///
/// # Algorithm
/// `Argon2id(password, salt, m_cost, t_cost, p_cost) → [u8; 32]`
///
/// # Security Properties
/// - Memory-hard: forces ~19MB RAM per attempt (GPU-resistant)
/// - Time-hard: ~250ms on modern hardware
/// - Always outputs uniform 32 bytes regardless of password length
///
/// # Parameters (from Config)
/// - `m_cost`: 19456 KiB (~19 MB)
/// - `t_cost`: 2 iterations
/// - `p_cost`: 1 parallelism lane
///
/// # Arguments
/// * `password` - The user's raw password bytes.
/// * `salt` - A 16-byte random salt (stored in the file header).
/// * `config` - Application config containing Argon2 cost parameters.
///
/// ⚠️ Never store the password. Only the derived key is used for encryption.
pub fn derive_key(password: &[u8], salt: &[u8], config: &Config) -> Result<[u8; 32], VaultError> {
    // Initialize Argon2id parameters based on our config
    let params = argon2::Params::new(
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost,
        Some(32), // We always want a 32-byte (256-bit) key output
    )
    .map_err(|e| VaultError::Config(format!("Invalid Argon2 params: {}", e)))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut output_key = [0u8; 32];

    // Hash the password into our output key buffer
    argon2
        .hash_password_into(password, salt, &mut output_key)
        .map_err(|e| VaultError::Encryption(format!("Argon2 KDF failed: {}", e)))?;

    Ok(output_key)
}
