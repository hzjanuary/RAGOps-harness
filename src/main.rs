mod dashboard;
mod db;
mod eval;
mod proxy;
mod red_team;

use std::env;
use std::error::Error;
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::time::Duration;

use axum::routing::{get, post};
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Serialize;
use tracing::{error, info, warn};

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
    /// Run the local OpenAI Chat Completions proxy with a live CLI dashboard.
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

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
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
        None | Some(Commands::Serve) => run_proxy_with_dashboard(&db_path).await?,
        Some(Commands::Dashboard) => {
            let db = db::Database::open(&db_path)?;
            crate::dashboard::run_dashboard(&db).await?;
        }
        Some(Commands::Scan { target }) => red_team::run_scan(target).await?,
        Some(Commands::Eval { dataset }) => eval::run_eval(dataset).await?,
    }

    Ok(())
}

async fn run_proxy_with_dashboard(db_path: &str) -> AppResult<()> {
    let db = db::Database::open(db_path)?;

    let openai_api_key = env::var("OPENAI_API_KEY").ok();
    if openai_api_key.is_none() {
        warn!("OPENAI_API_KEY is not set; proxy requests will return JSON configuration errors");
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("ragops-harness/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let state = proxy::AppState::new(client, db.clone(), openai_api_key);
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(proxy::chat_completions))
        .with_state(state);

    let address = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(
        address = %listener.local_addr()?,
        db_path = %db_path,
        "RAGOps Harness proxy listening"
    );

    let server = axum::serve(listener, app);
    tokio::spawn(async move {
        if let Err(error) = server.await {
            error!(error = %error, "RAGOps Harness proxy server stopped with an error");
        }
    });

    execute!(stdout(), EnterAlternateScreen, Hide).unwrap();

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                execute!(stdout(), MoveTo(0, 0), Clear(ClearType::All)).unwrap();
                if let Err(error) = crate::dashboard::run_dashboard(&db).await {
                    tracing::error!("Dashboard error: {}", error);
                }
                stdout().flush().unwrap();
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    execute!(stdout(), Show, LeaveAlternateScreen).unwrap();

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "ragops-harness",
    })
}
