mod client;
mod commands;
mod error;

use clap::{Parser, Subcommand};
use error::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Console {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 6379)]
        port: u16,
    },
    Txlog {
        #[arg(long)]
        read: PathBuf,
    },
    Backup {
        #[arg(long)]
        read: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Console { host, port } => commands::start_console(&host, port),
        Commands::Txlog { read } => commands::read_transaction_log(read),
        Commands::Backup { read } => commands::read_backup(read),
    }
}
