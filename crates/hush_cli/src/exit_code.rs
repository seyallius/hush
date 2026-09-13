//! Defines the CLI exit code contract and maps core errors to process exit codes.
//!
//! # Exit Code Contract (Issue #4)
//! - `0`: Success
//! - `1`: Generic error (I/O, internal bugs)
//! - `2`: Usage error (bad flags, missing args, bad config)
//! - `3`: Integrity error (tampered file, bad crypto tag, malformed envelope)
//! - `4`: Wrong password / key failure
//! - `5`: Unsupported feature (unknown cipher, future envelope version)

use hush_core::{Error, ExitCode};

/// Maps a core domain error to the standard process exit code.
///
/// This ensures the CLI strictly adheres to the exit code contract.
/// The mapping is exhaustive and derivable from the error variant alone.
pub fn to_process_exit_code(err: &Error) -> std::process::ExitCode {
    let code = match err.exit_code() {
        ExitCode::Success => 0,
        ExitCode::Generic => 1,
        ExitCode::Usage => 2,
        ExitCode::Integrity => 3,
        ExitCode::WrongKey => 4,
        ExitCode::Unsupported => 5,
    };

    std::process::ExitCode::from(code)
}
