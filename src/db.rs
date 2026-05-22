use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use tokio::sync::Mutex;

pub type SharedConnection = Arc<Mutex<Connection>>;

#[derive(Clone)]
pub struct Database {
    connection: SharedConnection,
}

#[derive(Debug, Clone)]
pub struct UsageLogEntry {
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub upstream_id: Option<String>,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub cost_usd: f64,
    pub latency_ms: u64,
    pub status_code: u16,
    pub pricing_known: bool,
}

#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    IntegerOverflow(&'static str),
    InvalidCost(f64),
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref();

        if path != Path::new(":memory:") {
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                std::fs::create_dir_all(parent).map_err(DbError::Io)?;
            }
        }

        let connection = Connection::open(path).map_err(DbError::Sqlite)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(DbError::Sqlite)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(DbError::Sqlite)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(DbError::Sqlite)?;
        connection
            .pragma_update(None, "synchronous", "NORMAL")
            .map_err(DbError::Sqlite)?;

        initialize_schema(&connection)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub async fn log_usage(&self, entry: UsageLogEntry) -> Result<(), DbError> {
        if !entry.cost_usd.is_finite() || entry.cost_usd < 0.0 {
            return Err(DbError::InvalidCost(entry.cost_usd));
        }

        let prompt_tokens = to_i64(entry.prompt_tokens, "prompt_tokens")?;
        let completion_tokens = to_i64(entry.completion_tokens, "completion_tokens")?;
        let total_tokens = to_i64(entry.total_tokens, "total_tokens")?;
        let cached_prompt_tokens = to_i64(entry.cached_prompt_tokens, "cached_prompt_tokens")?;
        let latency_ms = to_i64(entry.latency_ms, "latency_ms")?;
        let status_code = i64::from(entry.status_code);
        let pricing_known = if entry.pricing_known { 1_i64 } else { 0_i64 };
        let created_at = entry
            .created_at
            .to_rfc3339_opts(SecondsFormat::Millis, true);

        let connection = self.connection.lock().await;
        connection
            .execute(
                r#"
                INSERT INTO llm_request_logs (
                    request_id,
                    created_at,
                    provider,
                    endpoint,
                    model,
                    upstream_id,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    cached_prompt_tokens,
                    cost_usd,
                    latency_ms,
                    status_code,
                    pricing_known
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
                params![
                    entry.request_id,
                    created_at,
                    entry.provider,
                    entry.endpoint,
                    entry.model,
                    entry.upstream_id,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                    cached_prompt_tokens,
                    entry.cost_usd,
                    latency_ms,
                    status_code,
                    pricing_known,
                ],
            )
            .map_err(DbError::Sqlite)?;

        Ok(())
    }

    #[cfg(test)]
    pub async fn logged_count(&self) -> Result<u64, DbError> {
        let connection = self.connection.lock().await;
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM llm_request_logs", [], |row| {
                row.get(0)
            })
            .map_err(DbError::Sqlite)?;

        u64::try_from(count).map_err(|_| DbError::IntegerOverflow("logged_count"))
    }
}

impl Display for DbError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "database filesystem error: {error}"),
            Self::Sqlite(error) => write!(formatter, "sqlite error: {error}"),
            Self::IntegerOverflow(field) => {
                write!(formatter, "value for {field} exceeds SQLite INTEGER range")
            }
            Self::InvalidCost(cost) => write!(formatter, "invalid cost_usd value: {cost}"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::IntegerOverflow(_) | Self::InvalidCost(_) => None,
        }
    }
}

fn initialize_schema(connection: &Connection) -> Result<(), DbError> {
    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS llm_request_logs (
                request_id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                provider TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                model TEXT NOT NULL,
                upstream_id TEXT,
                prompt_tokens INTEGER NOT NULL CHECK (prompt_tokens >= 0),
                completion_tokens INTEGER NOT NULL CHECK (completion_tokens >= 0),
                total_tokens INTEGER NOT NULL CHECK (total_tokens >= 0),
                cached_prompt_tokens INTEGER NOT NULL DEFAULT 0 CHECK (cached_prompt_tokens >= 0),
                cost_usd REAL NOT NULL CHECK (cost_usd >= 0),
                latency_ms INTEGER NOT NULL CHECK (latency_ms >= 0),
                status_code INTEGER NOT NULL CHECK (status_code BETWEEN 100 AND 599),
                pricing_known INTEGER NOT NULL DEFAULT 0 CHECK (pricing_known IN (0, 1))
            );

            CREATE INDEX IF NOT EXISTS idx_llm_request_logs_created_at
                ON llm_request_logs (created_at);

            CREATE INDEX IF NOT EXISTS idx_llm_request_logs_model_created_at
                ON llm_request_logs (model, created_at);
            "#,
        )
        .map_err(DbError::Sqlite)
}

fn to_i64(value: u64, field: &'static str) -> Result<i64, DbError> {
    i64::try_from(value).map_err(|_| DbError::IntegerOverflow(field))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn persists_usage_log_entry() {
        let db = Database::open(":memory:").expect("database opens");

        db.log_usage(UsageLogEntry {
            request_id: "test-request-id".to_owned(),
            created_at: Utc::now(),
            provider: "openai".to_owned(),
            endpoint: "/v1/chat/completions".to_owned(),
            model: "gpt-4o-mini".to_owned(),
            upstream_id: Some("chatcmpl-test".to_owned()),
            prompt_tokens: 100,
            completion_tokens: 25,
            total_tokens: 125,
            cached_prompt_tokens: 10,
            cost_usd: 0.00003,
            latency_ms: 250,
            status_code: 200,
            pricing_known: true,
        })
        .await
        .expect("usage log persists");

        assert_eq!(db.logged_count().await.expect("count query succeeds"), 1);
    }
}
