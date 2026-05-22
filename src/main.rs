mod db;
mod proxy;
mod red_team;

use std::error::Error;

use clap::{Parser, Subcommand};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Parser)]
#[command(
    name = "ragops-harness",
    version,
    about = "Local RAGOps proxy and red-team scanner"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the local OpenAI Chat Completions proxy.
    Serve,
    /// Run the red-team prompt scanner against an OpenAI-compatible endpoint.
    Scan {
        /// Target URL that accepts OpenAI Chat Completions JSON.
        target: String,
    },
}

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Serve => proxy::run().await?,
        Command::Scan { target } => red_team::run_scan(&target).await?,
    }

    Ok(())
}
