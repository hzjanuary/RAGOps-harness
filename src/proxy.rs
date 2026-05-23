use std::env;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::{Database, UsageLogEntry};

const OPENAI_CHAT_COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_PROVIDER: &str = "openai";
const CHAT_COMPLETIONS_ENDPOINT: &str = "/v1/chat/completions";

#[derive(Clone)]
pub struct AppState {
    client: Client,
    pub(crate) db: Database,
    openai_api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

impl AppState {
    pub fn new(client: Client, db: Database, openai_api_key: Option<String>) -> Self {
        Self {
            client,
            db,
            openai_api_key,
        }
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        .route("/v1/chat/completions", post(chat_completions))
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

#[derive(Debug, Deserialize)]
struct ChatCompletionRequestShape {
    model: String,
    #[serde(default)]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    #[serde(default)]
    prompt_tokens_details: PromptTokensDetails,
}

#[derive(Debug, Default, Deserialize)]
struct PromptTokensDetails {
    #[serde(default)]
    cached_tokens: u64,
}

#[derive(Debug, Clone, Copy)]
struct Pricing {
    input_per_million: f64,
    cached_input_per_million: f64,
    output_per_million: f64,
}

#[derive(Debug, Clone, Copy)]
struct CostEstimate {
    cost_usd: f64,
    pricing_known: bool,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    upstream_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    upstream_body: Option<Value>,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

pub async fn chat_completions(State(state): State<AppState>, body: Bytes) -> Response {
    match handle_chat_completions(state, body).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "ragops-harness",
    })
}

fn load_dotenv_file() {
    match dotenvy::dotenv() {
        Ok(path) => info!(path = %path.display(), "Loaded .env configuration"),
        Err(error) if error.not_found() => {}
        Err(_) => warn!("Failed to load .env configuration; using existing process environment"),
    }
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        warn!(error = %error, "failed to install Ctrl-C shutdown handler");
    }
}

async fn handle_chat_completions(state: AppState, body: Bytes) -> Result<Response, ApiError> {
    let request_id = Uuid::new_v4().to_string();
    let started_at = Utc::now();
    let timer = Instant::now();

    let api_key = state.openai_api_key.as_deref().ok_or_else(|| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "missing_openai_api_key",
            "OPENAI_API_KEY is not configured for the proxy process.",
            request_id.clone(),
        )
    })?;

    let request_shape: ChatCompletionRequestShape =
        serde_json::from_slice(&body).map_err(|error| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "invalid_chat_completion_request",
                format!("Request body must be valid Chat Completions JSON: {error}"),
                request_id.clone(),
            )
        })?;

    if request_shape.model.trim().is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "missing_model",
            "Request field `model` must be a non-empty string.",
            request_id,
        ));
    }

    if request_shape.stream.unwrap_or(false) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "streaming_not_supported",
            "US-011 only supports non-streaming Chat Completions because FinOps logging requires a JSON usage block.",
            request_id,
        ));
    }

    let request_payload: Value = serde_json::from_slice(&body).map_err(|error| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            format!("Request body must be valid JSON: {error}"),
            request_id.clone(),
        )
    })?;

    let upstream_response = state
        .client
        .post(OPENAI_CHAT_COMPLETIONS_URL)
        .bearer_auth(api_key)
        .json(&request_payload)
        .send()
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::BAD_GATEWAY,
                "upstream_request_failed",
                format!("Failed to reach OpenAI Chat Completions: {error}"),
                request_id.clone(),
            )
        })?;

    let upstream_status = upstream_response.status();
    let response_text = upstream_response.text().await.map_err(|error| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "upstream_body_read_failed",
            format!("Failed to read OpenAI response body: {error}"),
            request_id.clone(),
        )
    })?;

    if !upstream_status.is_success() {
        warn!(
            request_id = %request_id,
            status_code = upstream_status.as_u16(),
            "OpenAI returned an error response"
        );

        return Err(ApiError::with_upstream(
            upstream_status,
            "openai_error",
            "OpenAI returned an error response.",
            request_id,
            parse_upstream_body(&response_text),
        ));
    }

    let response_json: Value = serde_json::from_str(&response_text).map_err(|error| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "upstream_json_parse_failed",
            format!("OpenAI returned a success response that was not valid JSON: {error}"),
            request_id.clone(),
        )
    })?;

    let usage = parse_usage(&response_json, &request_id)?;
    let response_model = response_json
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(request_shape.model.as_str())
        .to_owned();
    let upstream_id = response_json
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let cost = calculate_cost_usd(&response_model, &usage);
    let latency_ms = duration_millis(timer.elapsed());
    let status_code = upstream_status.as_u16();

    if !cost.pricing_known {
        warn!(
            request_id = %request_id,
            model = %response_model,
            "No pricing rule matched model; cost_usd was logged as 0.0"
        );
    }

    state
        .db
        .log_usage(UsageLogEntry {
            request_id: request_id.clone(),
            created_at: started_at,
            provider: OPENAI_PROVIDER.to_owned(),
            endpoint: CHAT_COMPLETIONS_ENDPOINT.to_owned(),
            model: response_model.clone(),
            upstream_id,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            cached_prompt_tokens: usage.prompt_tokens_details.cached_tokens,
            cost_usd: cost.cost_usd,
            latency_ms,
            status_code,
            pricing_known: cost.pricing_known,
        })
        .await
        .map_err(|error| {
            error!(
                request_id = %request_id,
                error = %error,
                "Failed to persist FinOps usage log"
            );
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "finops_log_failed",
                format!(
                    "OpenAI returned a response, but the local FinOps log write failed: {error}"
                ),
                request_id.clone(),
            )
        })?;

    info!(
        request_id = %request_id,
        model = %response_model,
        prompt_tokens = usage.prompt_tokens,
        completion_tokens = usage.completion_tokens,
        total_tokens = usage.total_tokens,
        cost_usd = cost.cost_usd,
        latency_ms,
        status_code,
        "Proxied OpenAI chat completion and logged usage"
    );

    Ok((StatusCode::OK, Json(response_json)).into_response())
}

impl ApiError {
    fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        request_id: String,
    ) -> Self {
        Self {
            status,
            body: ErrorBody {
                code,
                message: message.into(),
                request_id,
                upstream_status: None,
                upstream_body: None,
            },
        }
    }

    fn with_upstream(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        request_id: String,
        upstream_body: Option<Value>,
    ) -> Self {
        Self {
            status,
            body: ErrorBody {
                code,
                message: message.into(),
                request_id,
                upstream_status: Some(status.as_u16()),
                upstream_body,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorEnvelope { error: self.body })).into_response()
    }
}

fn parse_usage(response_json: &Value, request_id: &str) -> Result<OpenAiUsage, ApiError> {
    let usage_value = response_json.get("usage").cloned().ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "missing_usage",
            "OpenAI success response did not include a usage block.",
            request_id.to_owned(),
        )
    })?;

    serde_json::from_value(usage_value).map_err(|error| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "invalid_usage",
            format!("OpenAI usage block had an unexpected shape: {error}"),
            request_id.to_owned(),
        )
    })
}

fn calculate_cost_usd(model: &str, usage: &OpenAiUsage) -> CostEstimate {
    let Some(pricing) = pricing_for_model(model) else {
        return CostEstimate {
            cost_usd: 0.0,
            pricing_known: false,
        };
    };

    let cached_prompt_tokens = usage
        .prompt_tokens_details
        .cached_tokens
        .min(usage.prompt_tokens);
    let uncached_prompt_tokens = usage.prompt_tokens - cached_prompt_tokens;

    let input_cost = tokens_to_millions(uncached_prompt_tokens) * pricing.input_per_million;
    let cached_input_cost =
        tokens_to_millions(cached_prompt_tokens) * pricing.cached_input_per_million;
    let output_cost = tokens_to_millions(usage.completion_tokens) * pricing.output_per_million;

    CostEstimate {
        cost_usd: input_cost + cached_input_cost + output_cost,
        pricing_known: true,
    }
}

fn pricing_for_model(model: &str) -> Option<Pricing> {
    let normalized = model.trim().to_ascii_lowercase();

    if model_matches(&normalized, "gpt-5.5") {
        return Some(Pricing {
            input_per_million: 5.00,
            cached_input_per_million: 0.50,
            output_per_million: 30.00,
        });
    }

    if model_matches(&normalized, "gpt-5.4-mini") {
        return Some(Pricing {
            input_per_million: 0.75,
            cached_input_per_million: 0.075,
            output_per_million: 4.50,
        });
    }

    if model_matches(&normalized, "gpt-5.4-nano") {
        return Some(Pricing {
            input_per_million: 0.20,
            cached_input_per_million: 0.02,
            output_per_million: 1.25,
        });
    }

    if model_matches(&normalized, "gpt-5.4") {
        return Some(Pricing {
            input_per_million: 2.50,
            cached_input_per_million: 0.25,
            output_per_million: 15.00,
        });
    }

    if model_matches(&normalized, "gpt-5.3") || model_matches(&normalized, "gpt-5.2") {
        return Some(Pricing {
            input_per_million: 1.75,
            cached_input_per_million: 0.175,
            output_per_million: 14.00,
        });
    }

    if model_matches(&normalized, "gpt-5-mini") {
        return Some(Pricing {
            input_per_million: 0.25,
            cached_input_per_million: 0.025,
            output_per_million: 2.00,
        });
    }

    if model_matches(&normalized, "gpt-5-nano") {
        return Some(Pricing {
            input_per_million: 0.05,
            cached_input_per_million: 0.005,
            output_per_million: 0.40,
        });
    }

    if model_matches(&normalized, "gpt-5.1") || model_matches(&normalized, "gpt-5") {
        return Some(Pricing {
            input_per_million: 1.25,
            cached_input_per_million: 0.125,
            output_per_million: 10.00,
        });
    }

    if model_matches(&normalized, "gpt-4.1-mini") {
        return Some(Pricing {
            input_per_million: 0.40,
            cached_input_per_million: 0.10,
            output_per_million: 1.60,
        });
    }

    if model_matches(&normalized, "gpt-4.1-nano") {
        return Some(Pricing {
            input_per_million: 0.10,
            cached_input_per_million: 0.025,
            output_per_million: 0.40,
        });
    }

    if model_matches(&normalized, "gpt-4.1") {
        return Some(Pricing {
            input_per_million: 2.00,
            cached_input_per_million: 0.50,
            output_per_million: 8.00,
        });
    }

    if model_matches(&normalized, "gpt-4o-mini") {
        return Some(Pricing {
            input_per_million: 0.15,
            cached_input_per_million: 0.075,
            output_per_million: 0.60,
        });
    }

    if model_matches(&normalized, "gpt-4o") {
        return Some(Pricing {
            input_per_million: 2.50,
            cached_input_per_million: 1.25,
            output_per_million: 10.00,
        });
    }

    if model_matches(&normalized, "o4-mini") {
        return Some(Pricing {
            input_per_million: 1.10,
            cached_input_per_million: 0.275,
            output_per_million: 4.40,
        });
    }

    None
}

fn model_matches(model: &str, base: &str) -> bool {
    model == base
        || model
            .strip_prefix(base)
            .is_some_and(|suffix| suffix.starts_with('-'))
}

fn tokens_to_millions(tokens: u64) -> f64 {
    tokens as f64 / 1_000_000.0
}

fn parse_upstream_body(body: &str) -> Option<Value> {
    if body.trim().is_empty() {
        return None;
    }

    serde_json::from_str(body)
        .ok()
        .or_else(|| Some(json!({ "raw": body })))
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_cost_with_cached_prompt_tokens() {
        let usage = OpenAiUsage {
            prompt_tokens: 1_000,
            completion_tokens: 500,
            total_tokens: 1_500,
            prompt_tokens_details: PromptTokensDetails { cached_tokens: 400 },
        };

        let cost = calculate_cost_usd("gpt-4o-mini-2024-07-18", &usage);

        assert!(cost.pricing_known);
        assert!((cost.cost_usd - 0.00042).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_model_logs_zero_cost_and_marks_pricing_unknown() {
        let usage = OpenAiUsage {
            prompt_tokens: 1_000,
            completion_tokens: 500,
            total_tokens: 1_500,
            prompt_tokens_details: PromptTokensDetails::default(),
        };

        let cost = calculate_cost_usd("custom-provider-model", &usage);

        assert!(!cost.pricing_known);
        assert_eq!(cost.cost_usd, 0.0);
    }
}
