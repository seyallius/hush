//! hush.rs - The CLI entry point that parses arguments and executes commands.

use clap::Parser;

/// Cli defines the command-line interface structure using clap.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Commands defines the available subcommands for the hush CLI.
#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Encrypt a file
    Encrypt {
        #[arg(short, long)]
        input: String,
    },
    /// Decrypt a file
    Decrypt {
        #[arg(short, long)]
        input: String,
    },
    /// Stream a decrypted file to stdout
    Stream {
        #[arg(short, long)]
        input: String,
    },
}

/// The main entry point for the CLI application.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt { input } => {
            println!("Encrypting {}... (Not implemented yet) (≧◡≦)", input);
        }
        Commands::Decrypt { input } => {
            println!("Decrypting {}... (Not implemented yet) (｡◕‿◕｡)", input);
        }
        Commands::Stream { input } => {
            println!("Streaming {}... (Not implemented yet) ٩(ˊᗜˋ*)و", input);
        }
    }
}