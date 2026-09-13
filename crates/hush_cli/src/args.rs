//! Defines the CLI surface contract using Clap derive macros.
//!
//! # Help Text Conventions
//! - All doc comments on structs and fields are used by Clap to generate `--help` output.
//! - Keep descriptions concise, imperative, and lowercase (e.g., "encrypt a file" not "Encrypts a file").
//! - Always specify `value_name` for arguments that take a value (e.g., `FILE`, `BYTES`).
//! - Use `global = true` for flags that apply to all subcommands.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use hush_config::{CipherKind, KeyMode};

/// hush: Zero-knowledge encryption for sensitive media.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// path to the configuration file
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// increase logging verbosity (can be used multiple times)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// suppress all non-fatal output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    /// disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
    /// CLI subcommand.
    #[command(subcommand)]
    pub command: Command,
}

/// All top-level Hush CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// encrypt a file or directory
    Encrypt(EncryptArgs),
    /// decrypt a file or directory
    Decrypt(DecryptArgs),
    /// stream decrypted media to stdout or a local HTTP server
    Stream(StreamArgs),
    /// manage configuration
    #[command(subcommand)]
    Config(ConfigCommand),
    /// initialize a new configuration file with sensible defaults
    Init(InitArgs),
    /// launch the Terminal User Interface
    Tui(TuiArgs),
    /// rotate encryption keys for existing files
    Rekey(RekeyArgs),
    /// Shamir's Secret Sharing operations
    #[command(subcommand)]
    Sss(SssCommand),
    /// recovery QR code operations
    #[command(subcommand)]
    Recovery(RecoveryCommand),
    /// batch process files using glob patterns
    Batch(BatchArgs),
}

/// Arguments for `hush encrypt`.
#[derive(Parser, Debug)]
pub struct EncryptArgs {
    /// input file or directory
    pub input: PathBuf,
    /// output file or directory
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// key mode to use
    #[arg(long, value_enum)]
    pub key_mode: Option<KeyMode>,
    /// cipher algorithm
    #[arg(long, value_enum)]
    pub cipher: Option<CipherKind>,
    /// chunk size in bytes
    #[arg(long, value_name = "BYTES")]
    pub chunk_size: Option<usize>,
    /// read password from a file instead of prompting
    #[arg(long, value_name = "FILE")]
    pub password_file: Option<PathBuf>,
}

/// Arguments for `hush decrypt`.
#[derive(Parser, Debug)]
pub struct DecryptArgs {
    /// input `.hush` file or directory
    pub input: PathBuf,
    /// output file or directory
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// read password from a file instead of prompting
    #[arg(long, value_name = "FILE")]
    pub password_file: Option<PathBuf>,
}

/// Arguments for `hush stream`.
#[derive(Parser, Debug)]
pub struct StreamArgs {
    /// input `.hush` file
    pub input: PathBuf,
    /// read password from a file instead of prompting
    #[arg(long, value_name = "FILE")]
    pub password_file: Option<PathBuf>,
    /// start a local HTTP server instead of writing to stdout
    #[arg(long)]
    pub http: bool,
    /// port for the local HTTP server
    #[arg(long, requires = "http", value_name = "PORT")]
    pub port: Option<u16>,
}

/// Subcommands for `hush config`.
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// print the current resolved configuration
    Show,
    /// validate the configuration file without running an operation
    Validate,
}

/// Arguments for `hush init`.
#[derive(Parser, Debug)]
pub struct InitArgs {
    /// overwrite existing configuration file
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `hush tui`.
#[derive(Parser, Debug)]
pub struct TuiArgs {
    /// directory to browse for encrypted files
    pub directory: Option<PathBuf>,
}

/// Arguments for `hush rekey`.
#[derive(Parser, Debug)]
pub struct RekeyArgs {
    /// input file or directory
    pub input: PathBuf,
    /// output file or directory
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// new key mode to apply
    #[arg(long, value_enum)]
    pub new_key_mode: Option<KeyMode>,
}

/// Subcommands for `hush sss`.
#[derive(Subcommand, Debug)]
pub enum SssCommand {
    /// split a master key into N shares with threshold K
    Split {
        /// number of shares to generate
        #[arg(long)]
        n: u8,
        /// threshold required to reconstruct
        #[arg(long)]
        k: u8,
    },
    /// recover a master key from shares
    Recover,
}

/// Subcommands for `hush recovery`.
#[derive(Subcommand, Debug)]
pub enum RecoveryCommand {
    /// generate and display a recovery QR code
    Qr,
    /// decode a recovery payload from a string or image
    Decode,
}

/// Arguments for `hush batch`.
#[derive(Parser, Debug)]
pub struct BatchArgs {
    /// glob pattern for input files
    pub pattern: String,
    /// operation to perform
    #[arg(long, value_enum)]
    pub op: BatchOp,
    /// number of parallel jobs
    #[arg(long, value_name = "NUM")]
    pub jobs: Option<usize>,
    /// continue processing on error
    #[arg(long)]
    pub continue_on_error: bool,
    /// dry run: list files without processing
    #[arg(long)]
    pub dry_run: bool,
}

/// Batch operation mode.
#[derive(ValueEnum, Clone, Debug)]
pub enum BatchOp {
    /// encrypt matched files
    Encrypt,
    /// decrypt matched files
    Decrypt,
}
