mod dashboard;
mod db;
mod eval;
mod proxy;
mod red_team;

use std::env;
use std::error::Error;

use clap::{Parser, Subcommand};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Parser)]
#[command(
    name = "ragops-harness",
    version,
    about = "Local RAGOps proxy, CLI dashboard, red-team scanner, and RAG evaluator"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the local OpenAI Chat Completions proxy.
    Serve,
    /// Print a one-shot FinOps report from the local SQLite database.
    Dashboard,
    /// Run the red-team prompt scanner against an OpenAI-compatible endpoint.
    Scan {
        /// Target URL that accepts OpenAI Chat Completions JSON.
        #[arg(short, long)]
        target: String,
    },
    /// Run RAG faithfulness evaluation against a local JSON dataset.
    Eval {
        /// Path to a JSON array of records with question, context, and answer fields.
        #[arg(short, long)]
        dataset: String,
    },
}

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();
    let _ = dotenvy::dotenv();
    let db_path =
        env::var("RAGOPS_DB_PATH").unwrap_or_else(|_| "ragops_harness.sqlite3".to_owned());

    match &cli.command {
        None | Some(Commands::Serve) => proxy::run().await?,
        Some(Commands::Dashboard) => crate::dashboard::run_dashboard(&db_path).await?,
        Some(Commands::Scan { target }) => red_team::run_scan(target).await?,
        Some(Commands::Eval { dataset }) => eval::run_eval(dataset).await?,
    }

    Ok(())
}
