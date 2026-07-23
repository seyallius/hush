//! hush.rs - The CLI entry point that parses arguments and executes commands.

use clap::Parser;
use hush::stream;
use std::path::PathBuf;

/// Cli defines the command-line interface structure using clap.
#[derive(Parser, Debug)]
#[command(author, version, about = "hush - silence your files 🔇", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

/// The main entry point for the CLI application.
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Parses CLI args and dispatches to the appropriate handler.
fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt { input, output } => handle_encrypt(&input, output)?,
        Commands::Decrypt { input, output } => handle_decrypt(&input, output)?,
        Commands::Stream { input } => handle_stream(&input)?,
    }

    Ok(())
}

/// Handles the `encrypt` subcommand: prompts for password, encrypts file.
fn handle_encrypt(input: &PathBuf, output: Option<PathBuf>) -> anyhow::Result<()> {
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

    let config = hush::config::Config::default();

    println!(
        "Encrypting {} → {} ...",
        input.display(),
        output_path.display()
    );

    let bytes_written = stream::encrypt_file(input, &output_path, password.as_bytes(), &config)?;

    println!("Done! Wrote {} bytes. 🔇 (≧◡≦)", bytes_written);
    Ok(())
}

/// Handles the `decrypt` subcommand: prompts for password, decrypts to file.
fn handle_decrypt(input: &PathBuf, output: Option<PathBuf>) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    eprint!("Enter password: ");
    let password = rpassword::read_password()?;

    // First pass: decrypt to get metadata (we need the original filename)
    // We'll decrypt to a temp buffer if no output specified, or directly to file
    let output_path = if let Some(p) = output {
        p
    } else {
        // Peek at metadata to get original filename
        let temp_output = std::path::PathBuf::from("/dev/null");
        let mut dev_null = std::fs::File::create(&temp_output)?;
        let metadata = stream::decrypt_to_writer(input, &mut dev_null, password.as_bytes())?;
        PathBuf::from(&metadata.original_filename)
    };

    let mut out_file = std::fs::File::create(&output_path)?;
    let metadata = stream::decrypt_to_writer(input, &mut out_file, password.as_bytes())?;

    println!(
        "Decrypted: {} ({} bytes) → {} 🔓 ٩(ˊᗜˋ*)و",
        metadata.original_filename,
        metadata.original_size,
        output_path.display()
    );
    Ok(())
}

/// Handles the `stream` subcommand: decrypts to stdout for piping.
fn handle_stream(input: &PathBuf) -> anyhow::Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    eprint!("Enter password: ");
    let password = rpassword::read_password()?;

    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    stream::decrypt_to_writer(input, &mut stdout_lock, password.as_bytes())?;

    Ok(())
}
