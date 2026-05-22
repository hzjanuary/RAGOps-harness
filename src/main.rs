mod db;
mod proxy;

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tracing::{info, warn};

use crate::db::Database;
use crate::proxy::AppState;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    load_dotenv_file();

    let db_path =
        env::var("RAGOPS_DB_PATH").unwrap_or_else(|_| "ragops_harness.sqlite3".to_owned());
    let db = Database::open(&db_path)?;

    let openai_api_key = env::var("OPENAI_API_KEY").ok();
    if openai_api_key.is_none() {
        warn!("OPENAI_API_KEY is not set; proxy requests will return JSON configuration errors");
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("ragops-harness/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let state = AppState::new(client, db, openai_api_key);
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

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn load_dotenv_file() {
    match dotenvy::dotenv() {
        Ok(path) => info!(path = %path.display(), "Loaded .env configuration"),
        Err(error) if error.not_found() => {}
        Err(_) => warn!("Failed to load .env configuration; using existing process environment"),
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "ragops-harness",
    })
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        warn!(error = %error, "failed to install Ctrl-C shutdown handler");
    }
}
