//! Root crate for hush_cli, exposing the CLI contract and exit code mappings.

pub mod args;
pub mod exit_code;

pub use args::{Cli, Command};
