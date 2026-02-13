mod repo;
mod status;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status,
    // Log,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = repo::open_repo()?;

    match cli.command {
        Commands::Status => status::status(&repo)?,
    };

    Ok(())
}
