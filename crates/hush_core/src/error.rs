//! Defines Hush's core error taxonomy, stable exit-code mapping, and TUI-friendly messages.

/// A boxed, thread-safe error source used to wrap lower-level failures.
///
/// This is useful when Hush needs to preserve an underlying error source
/// without exposing the lower-level crate's error type in the public API.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// The canonical result alias used across Hush domain code.
///
/// Library crates may define their own low-level errors later, but
/// application-facing code should generally converge on this result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Stable process exit codes used by the CLI.
///
/// These correspond to the exit-code contract described in issue #4:
/// 0 success, 1 generic error, 2 usage error, 3 integrity error,
/// 4 wrong password, 5 unsupported feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// The operation completed successfully.
    Success,
    /// A generic or internal failure.
    Generic,
    /// Invalid arguments, missing required input, or invalid configuration.
    Usage,
    /// Data failed authentication, validation, or envelope integrity.
    Integrity,
    /// The user supplied the wrong password/key or failed hardware challenge.
    WrongKey,
    /// The requested feature, cipher, mode, or version is unsupported.
    Unsupported,
}
impl ExitCode {
    /// Returns the numeric exit code expected by shells.
    ///
    /// This is deliberately separate from [`std::process::ExitCode`]
    /// so core logic does not need to depend on process-specific types.
    pub fn as_u8(self) -> u8 {
        match self {
            ExitCode::Success => 0,
            ExitCode::Generic => 1,
            ExitCode::Usage => 2,
            ExitCode::Integrity => 3,
            ExitCode::WrongKey => 4,
            ExitCode::Unsupported => 5,
        }
    }
}

/// Key-specific failure categories.
///
/// Key errors are separated from generic crypto errors because they often
/// require distinct user-facing behavior:
/// - wrong password should be reported as an authentication failure
/// - missing password/YubiKey is usually a usage problem
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KeyError {
    /// The password or key did not authenticate.
    #[error("wrong password or key")]
    WrongPassword,
    /// A password is required but was not provided.
    #[error("password is required")]
    MissingPassword,
    /// A YubiKey is required but was not present.
    #[error("YubiKey is required")]
    MissingYubiKey,
    /// The YubiKey challenge-response failed.
    #[error("YubiKey challenge failed")]
    YubiKeyChallengeFailed,
    /// Key material has an invalid size or format.
    #[error("invalid key material")]
    InvalidKeyMaterial,
}
impl KeyError {
    /// Maps key-related failures to CLI exit codes.
    ///
    /// Wrong secrets and failed challenges map to the wrong-key exit code.
    /// Missing required secrets are usage errors.
    pub fn exit_code(self) -> ExitCode {
        match self {
            KeyError::WrongPassword
            | KeyError::YubiKeyChallengeFailed
            | KeyError::InvalidKeyMaterial => ExitCode::WrongKey,

            KeyError::MissingPassword | KeyError::MissingYubiKey => ExitCode::Usage,
        }
    }

    /// Returns a TUI-friendly, user-facing message.
    ///
    /// This message should be safe to render directly in the terminal UI.
    pub fn user_message(self) -> &'static str {
        match self {
            KeyError::WrongPassword => "Wrong password or key.",
            KeyError::MissingPassword => "A password is required.",
            KeyError::MissingYubiKey => "A YubiKey is required.",
            KeyError::YubiKeyChallengeFailed => "YubiKey challenge failed.",
            KeyError::InvalidKeyMaterial => "The provided key material is invalid.",
        }
    }
}

/// The top-level Hush error taxonomy.
///
/// # Adding New Variants
///
/// This enum is intentionally exhaustive. When adding a new failure mode, you must:
/// 1. Add a new variant here (or reuse an existing one like `Internal` or `Usage`).
/// 2. Map it to a stable `ExitCode` in the `exit_code()` method.
/// 3. Provide a user-friendly message in the `user_message()` method.
///
/// **When to reuse vs. create new:**
/// - Reuse `Usage` for any user-input or CLI-flag problem.
/// - Reuse `Internal` for logic bugs or unreachable states.
/// - Create a **new variant** only when the TUI or CLI needs to render a distinctly
///   different UI state, or when the user needs to take a specific, unique recovery action.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid CLI usage or invalid user-provided arguments.
    #[error("usage error: {message}")]
    Usage {
        /// Human-readable explanation.
        message: String,
    },
    /// Invalid configuration file or config value.
    #[error("configuration error: {message}")]
    Config {
        /// Human-readable explanation.
        message: String,
    },
    /// Filesystem or stream I/O failure.
    #[error("I/O error: {message}")]
    Io {
        /// Contextual message.
        message: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Cryptographic operation failure.
    ///
    /// This is distinct from `Integrity` because some crypto failures may be
    /// operational rather than authentication failures. However, by default
    /// we treat crypto failures as integrity-sensitive errors.
    #[error("crypto error: {message}")]
    Crypto {
        /// Contextual message.
        message: String,
        /// Optional underlying crypto crate error.
        #[source]
        source: Option<BoxError>,
    },
    /// Envelope structure, magic, version, or layout failure.
    #[error("envelope error: {message}")]
    Envelope {
        /// Human-readable explanation.
        message: String,
    },
    /// Authenticated integrity failure.
    ///
    /// This is used when AEAD authentication fails, a checksum mismatches,
    /// or encrypted data is otherwise detected as corrupted/tampered.
    #[error("integrity error: {message}")]
    Integrity {
        /// Human-readable explanation.
        message: String,
    },
    /// Key derivation or key material failure.
    #[error("key error: {kind}")]
    Key {
        /// Specific key failure type.
        kind: KeyError,
    },
    /// Unsupported cipher, mode, version, or feature.
    #[error("unsupported feature: {message}")]
    Unsupported {
        /// Human-readable explanation.
        message: String,
    },
    /// Unexpected internal bug.
    ///
    /// If this variant reaches the user, we should also log enough context
    /// to diagnose the issue, but logging policy is handled by issue #26.
    #[error("internal error: {message}")]
    Internal {
        /// Contextual message.
        message: String,
        /// Optional underlying error.
        #[source]
        source: Option<BoxError>,
    },
}
impl Error {
    /// Creates a usage error.
    ///
    /// Use this for invalid CLI arguments, missing required flags, or other
    /// user-input problems that do not involve file corruption.
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage {
            message: message.into(),
        }
    }

    /// Creates a configuration error.
    ///
    /// Use this when a config file is malformed, missing required fields,
    /// or contains invalid values.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Creates an I/O error with contextual information.
    ///
    /// Prefer this over bare `std::io::Error` when the operation context is
    /// important, such as "failed to read envelope header".
    pub fn io(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source,
        }
    }

    /// Creates a crypto error without an underlying source.
    pub fn crypto(message: impl Into<String>) -> Self {
        Self::Crypto {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a crypto error with an underlying source.
    ///
    /// Use this when wrapping errors from cryptographic crates while keeping
    /// the public Hush error type stable.
    pub fn crypto_with_source(message: impl Into<String>, source: impl Into<BoxError>) -> Self {
        Self::Crypto {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Creates an envelope-format error.
    ///
    /// Use this for invalid magic bytes, unsupported versions, truncated
    /// headers, or malformed metadata sections.
    pub fn envelope(message: impl Into<String>) -> Self {
        Self::Envelope {
            message: message.into(),
        }
    }

    /// Creates an integrity error.
    ///
    /// Use this when authentication fails or data is detected as corrupted.
    pub fn integrity(message: impl Into<String>) -> Self {
        Self::Integrity {
            message: message.into(),
        }
    }

    /// Creates a key error from a specific key failure kind.
    pub fn key(kind: KeyError) -> Self {
        Self::Key { kind }
    }

    /// Creates a wrong-password or wrong-key error.
    pub fn wrong_password() -> Self {
        Self::key(KeyError::WrongPassword)
    }

    /// Creates an error for a missing required password.
    pub fn missing_password() -> Self {
        Self::key(KeyError::MissingPassword)
    }

    /// Creates an error for a missing required YubiKey.
    pub fn missing_yubikey() -> Self {
        Self::key(KeyError::MissingYubiKey)
    }

    /// Creates an error for a failed YubiKey challenge-response.
    pub fn yubikey_challenge_failed() -> Self {
        Self::key(KeyError::YubiKeyChallengeFailed)
    }

    /// Creates an error for invalid key material.
    pub fn invalid_key_material() -> Self {
        Self::key(KeyError::InvalidKeyMaterial)
    }

    /// Creates an unsupported-feature error.
    ///
    /// Use this for unsupported ciphers, unsupported envelope versions,
    /// disabled key modes, or platform-specific unavailable features.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }

    /// Creates an internal error without an underlying source.
    ///
    /// Use this for logic bugs that should never happen in a correct program.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an internal error with an underlying source.
    ///
    /// Use this when an unexpected lower-level failure indicates a bug or
    /// invariant violation inside Hush.
    pub fn internal_with_source(message: impl Into<String>, source: impl Into<BoxError>) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Returns the stable CLI exit code for this error.
    ///
    /// This mapping must be derivable from the error variant alone.
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage { .. } | Self::Config { .. } => ExitCode::Usage,
            Self::Io { .. } => ExitCode::Generic,
            Self::Crypto { .. } | Self::Envelope { .. } | Self::Integrity { .. } => {
                ExitCode::Integrity
            }
            Self::Key { kind } => kind.exit_code(),
            Self::Unsupported { .. } => ExitCode::Unsupported,
            Self::Internal { .. } => ExitCode::Generic,
        }
    }

    /// Returns a user-facing message suitable for the TUI or CLI stderr.
    ///
    /// This is intentionally separate from `Display`. `Display` may include
    /// more technical detail, while `user_message` should be safe and clear
    /// for end users.
    pub fn user_message(&self) -> String {
        match self {
            Self::Usage { message } => format!("Usage error: {message}"),
            Self::Config { message } => format!("Configuration error: {message}"),
            Self::Io { message, source } => format!("I/O error: {message}: {source}"),
            Self::Crypto { message, .. } => format!("Crypto error: {message}"),
            Self::Envelope { message } => format!("Envelope error: {message}"),
            Self::Integrity { message } => format!("Integrity error: {message}"),
            Self::Key { kind } => kind.user_message().to_owned(),
            Self::Unsupported { message } => format!("Unsupported: {message}"),
            Self::Internal { message, .. } => format!("Internal error: {message}"),
        }
    }
}
impl From<std::io::Error> for Error {
    /// Converts a bare I/O error into the Hush error taxonomy.
    ///
    /// Prefer `Error::io` when you have useful operation context.
    fn from(source: std::io::Error) -> Self {
        Self::io("I/O operation failed", source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures the numeric exit codes match the CLI contract.
    #[test]
    fn exit_codes_match_cli_contract() {
        assert_eq!(ExitCode::Success.as_u8(), 0);
        assert_eq!(ExitCode::Generic.as_u8(), 1);
        assert_eq!(ExitCode::Usage.as_u8(), 2);
        assert_eq!(ExitCode::Integrity.as_u8(), 3);
        assert_eq!(ExitCode::WrongKey.as_u8(), 4);
        assert_eq!(ExitCode::Unsupported.as_u8(), 5);
    }

    /// Ensures each top-level error category maps to the expected exit code.
    #[test]
    fn error_variants_map_to_expected_exit_codes() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "boom");

        assert_eq!(Error::usage("bad args").exit_code(), ExitCode::Usage);
        assert_eq!(Error::config("bad config").exit_code(), ExitCode::Usage);
        assert_eq!(
            Error::io("read failed", io_error).exit_code(),
            ExitCode::Generic
        );
        assert_eq!(
            Error::crypto("cipher failed").exit_code(),
            ExitCode::Integrity
        );
        assert_eq!(
            Error::envelope("bad magic").exit_code(),
            ExitCode::Integrity
        );
        assert_eq!(
            Error::integrity("tag mismatch").exit_code(),
            ExitCode::Integrity
        );
        assert_eq!(Error::wrong_password().exit_code(), ExitCode::WrongKey);
        assert_eq!(Error::missing_password().exit_code(), ExitCode::Usage);
        assert_eq!(Error::missing_yubikey().exit_code(), ExitCode::Usage);
        assert_eq!(
            Error::yubikey_challenge_failed().exit_code(),
            ExitCode::WrongKey
        );
        assert_eq!(
            Error::invalid_key_material().exit_code(),
            ExitCode::WrongKey
        );
        assert_eq!(
            Error::unsupported("aes-gcm").exit_code(),
            ExitCode::Unsupported
        );
        assert_eq!(Error::internal("bug").exit_code(), ExitCode::Generic);
    }

    /// Ensures every error variant produces a non-empty user-facing message.
    #[test]
    fn every_error_has_user_message() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "boom");

        let errors = vec![
            Error::usage("bad args"),
            Error::config("bad config"),
            Error::io("read failed", io_error),
            Error::crypto("cipher failed"),
            Error::envelope("bad magic"),
            Error::integrity("tag mismatch"),
            Error::wrong_password(),
            Error::missing_password(),
            Error::missing_yubikey(),
            Error::yubikey_challenge_failed(),
            Error::invalid_key_material(),
            Error::unsupported("aes-gcm"),
            Error::internal("bug"),
        ];

        for error in errors {
            assert!(!error.user_message().is_empty());
        }
    }
}
