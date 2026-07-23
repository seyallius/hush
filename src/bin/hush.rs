//! Binary hush. hush.rs is the CLI entry point that parses arguments,
//! loads config (3-layer merge), and dispatches to encrypt/decrypt/stream/info/init.

use clap::Parser;
use hush::{
    config::{CipherKind, Config},
    progress::ProgressReporter,
    stream,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;

/// Cli defines the command-line interface structure using clap.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "hush 🔇 — silence your files. Client-side encryption for sensitive media.",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Override chunk size in bytes (e.g., 2097152 for 2MB)
    #[arg(long, global = true)]
    chunk_size: Option<usize>,

    /// Override cipher algorithm (x_chacha20_poly1305 | aes256_gcm)
    #[arg(long, global = true)]
    cipher: Option<CipherKind>,

    /// Override Argon2id memory cost in KiB
    #[arg(long, global = true)]
    argon2_m_cost: Option<u32>,

    /// Override Argon2id time cost (iterations)
    #[arg(long, global = true)]
    argon2_t_cost: Option<u32>,

    /// Override Argon2id parallelism
    #[arg(long, global = true)]
    argon2_p_cost: Option<u32>,
}

/// Commands defines the available subcommands for the hush CLI.
#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Encrypt a file into the hush binary format
    Encrypt {
        /// Path to the input file to encrypt
        #[arg(short, long)]
        input: PathBuf,

        /// Path for the output .hush file (defaults to <input>.hush)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decrypt a .hush file back to its original form
    Decrypt {
        /// Path to the .hush file to decrypt
        #[arg(short, long)]
        input: PathBuf,

        /// Path for the decrypted output (defaults to original filename from metadata)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Stream decrypted content to stdout (for piping to mpv, etc.)
    Stream {
        /// Path to the .hush file to stream
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Inspect a .hush file's header (no password required)
    Info {
        /// Path to the .hush file to inspect
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Generate a default config file at ~/.config/hush/config.toml
    Init,
}

// ----------------------- Public Functions -----------------------

/// The main entry point for the CLI application.
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

// ----------------------- Private Functions -----------------------

/// Parses CLI args, loads config with 3-layer merge, and dispatches.
fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // --- 3-Layer Config Merge: Default → File → CLI ---
    let mut config = Config::load();
    config.apply_overrides(
        cli.chunk_size,
        cli.cipher,
        cli.argon2_m_cost,
        cli.argon2_t_cost,
        cli.argon2_p_cost,
    );

    match cli.command {
        Commands::Encrypt { input, output } => handle_encrypt(&input, output, &config)?,
        Commands::Decrypt { input, output } => handle_decrypt(&input, output)?,
        Commands::Stream { input } => handle_stream(&input)?,
        Commands::Info { input } => handle_info(&input)?,
        Commands::Init => handle_init()?,
    }

    Ok(())
}

/// Handles `hush encrypt`: prompts for password, encrypts with progress bar.
fn handle_encrypt(input: &PathBuf, output: Option<PathBuf>, config: &Config) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    let output_path = output.unwrap_or_else(|| {
        let mut p = input.clone();
        let mut name = p.file_name().unwrap().to_os_string();
        name.push(".hush");
        p.set_file_name(name);
        p
    });

    eprint!("Enter password: ");
    let password = rpassword::read_password()?;
    eprint!("Confirm password: ");
    let password_confirm = rpassword::read_password()?;

    if password != password_confirm {
        anyhow::bail!("Passwords do not match!");
    }

    let file_size = fs::metadata(input)?.len();
    let progress = CliProgressReporter::new(file_size, "Encrypting");

    println!(
        "Config: cipher={:?}, chunk_size={}B, argon2(m={}, t={}, p={})",
        config.cipher,
        config.chunk_size,
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost
    );

    let bytes_written =
        stream::encrypt_file(input, &output_path, password.as_bytes(), config, &progress)?;

    println!(
        "\nDone! Wrote {} bytes → {} 🔇 (≧◡≦)",
        bytes_written,
        output_path.display()
    );
    Ok(())
}

/// Handles `hush decrypt`: prompts for password, decrypts with progress bar.
fn handle_decrypt(input: &PathBuf, output: Option<PathBuf>) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    eprint!("Enter password: ");
    let password = rpassword::read_password()?;

    // Determine output path from metadata if not specified
    let output_path = if let Some(p) = output {
        p
    } else {
        // Peek at metadata to get original filename (requires full decrypt pass)
        let mut dev_null = std::io::sink();
        let noop = hush::progress::NoopReporter;
        let metadata = stream::decrypt_to_writer(input, &mut dev_null, password.as_bytes(), &noop)?;
        PathBuf::from(&metadata.original_filename)
    };

    let file_size = fs::metadata(input)?.len();
    let progress = CliProgressReporter::new(file_size, "Decrypting");

    let mut out_file = fs::File::create(&output_path)?;
    let metadata = stream::decrypt_to_writer(input, &mut out_file, password.as_bytes(), &progress)?;

    println!(
        "\nDecrypted: {} ({} bytes) → {} 🔓 ٩(ˊᗜˋ*)و",
        metadata.original_filename,
        metadata.original_size,
        output_path.display()
    );
    Ok(())
}

/// Handles `hush stream`: decrypts to stdout for piping to media players.
/// No progress bar (would corrupt the stdout pipe).
fn handle_stream(input: &PathBuf) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    eprint!("Enter password: ");
    let password = rpassword::read_password()?;

    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    let noop = hush::progress::NoopReporter;

    stream::decrypt_to_writer(input, &mut stdout_lock, password.as_bytes(), &noop)?;

    Ok(())
}

/// Handles `hush info`: reads and displays the plaintext header (no password needed).
fn handle_info(input: &PathBuf) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    let file = fs::File::open(input)?;
    let mut reader = std::io::BufReader::new(file);
    let header = hush::envelope::FileHeader::read_from(&mut reader)?;

    println!("┌─────────────────────────────────────────┐");
    println!("│  hush file info                         │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│  Magic:       {:?}",
        String::from_utf8_lossy(&header.magic)
    );
    println!("│  Version:     {}", header.version);
    println!("│  Key Mode:    {:?}", header.key_mode);
    println!("│  Salt:        {:02x?}", &header.salt[..8]);
    println!("│  Argon2 m:    {} KiB", header.argon2_params.0);
    println!("│  Argon2 t:    {} iterations", header.argon2_params.1);
    println!("│  Argon2 p:    {} threads", header.argon2_params.2);
    println!(
        "│  YubiKey:     {}",
        if header.yubikey_challenge.iter().all(|&b| b == 0) {
            "not used"
        } else {
            "challenge present"
        }
    );
    println!("└─────────────────────────────────────────┘");

    Ok(())
}

/// Handles `hush init`: creates the default config file.
fn handle_init() -> anyhow::Result<()> {
    let path = Config::save_default()?;
    println!("Created default config at: {}", path.display());
    println!("Edit it to customize chunk size, cipher, or Argon2 parameters. (≧◡≦)");
    Ok(())
}

// ----------------------- Progress Reporter (CLI-specific) -----------------------

/// CliProgressReporter wraps indicatif's ProgressBar for terminal display.
/// Lives in the binary crate, NOT the library (keeps lib terminal-agnostic).
struct CliProgressReporter {
    bar: ProgressBar,
}

impl CliProgressReporter {
    /// Creates a new progress bar with a nice style.
    fn new(total_bytes: u64, verb: &str) -> Self {
        let bar = ProgressBar::new(total_bytes);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{}  {{spinner:.green}} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}})",
                    verb
                ))
                .unwrap()
                .progress_chars("█▓▒░ "),
        );
        Self { bar }
    }
}

impl ProgressReporter for CliProgressReporter {
    /// Updates the progress bar position.
    fn report(&self, bytes_done: u64, _total_bytes: u64) {
        self.bar.set_position(bytes_done);
    }

    /// Finishes and clears the progress bar.
    fn finish(&self) {
        self.bar.finish_and_clear();
    }
}
