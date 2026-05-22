use rusqlite::OptionalExtension;
use serde::Serialize;
use serde_json::json;

use crate::db::DbError;
use crate::proxy::AppState;

#[derive(Debug, Serialize)]
struct FinopsStats {
    total_requests: u64,
    total_cost_usd: f64,
    avg_latency_ms: f64,
    latest_logs: Vec<LatestLog>,
}

#[derive(Debug, Serialize)]
struct LatestLog {
    request_id: String,
    created_at: String,
    model: String,
    cost_usd: f64,
    latency_ms: u64,
    status_code: u16,
}

pub async fn api_stats(
    axum::extract::State(state): axum::extract::State<crate::proxy::AppState>,
) -> axum::Json<serde_json::Value> {
    match load_finops_stats(state).await {
        Ok(stats) => axum::Json(json!(stats)),
        Err(error) => axum::Json(json!({
            "error": {
                "code": "finops_stats_failed",
                "message": error.to_string()
            }
        })),
    }
}

async fn load_finops_stats(state: AppState) -> Result<FinopsStats, DbError> {
    state
        .db
        .with_connection(|connection| {
            if !has_usage_table(connection)? {
                return Ok(FinopsStats {
                    total_requests: 0,
                    total_cost_usd: 0.0,
                    avg_latency_ms: 0.0,
                    latest_logs: Vec::new(),
                });
            }

            let (total_requests, total_cost_usd, avg_latency_ms) = connection
                .query_row(
                    r#"
                    SELECT
                        COUNT(*),
                        COALESCE(SUM(cost_usd), 0.0),
                        COALESCE(AVG(latency_ms), 0.0)
                    FROM llm_request_logs
                    "#,
                    [],
                    |row| {
                        let total_requests: i64 = row.get(0)?;
                        let total_cost_usd: f64 = row.get(1)?;
                        let avg_latency_ms: f64 = row.get(2)?;

                        Ok((
                            u64::try_from(total_requests).unwrap_or(0),
                            total_cost_usd,
                            avg_latency_ms,
                        ))
                    },
                )
                .map_err(DbError::Sqlite)?;

            let mut statement = connection
                .prepare(
                    r#"
                    SELECT
                        request_id,
                        created_at,
                        model,
                        cost_usd,
                        latency_ms,
                        status_code
                    FROM llm_request_logs
                    ORDER BY created_at DESC
                    LIMIT 5
                    "#,
                )
                .map_err(DbError::Sqlite)?;

            let latest_logs = statement
                .query_map([], |row| {
                    let latency_ms: i64 = row.get(4)?;
                    let status_code: i64 = row.get(5)?;

                    Ok(LatestLog {
                        request_id: row.get(0)?,
                        created_at: row.get(1)?,
                        model: row.get(2)?,
                        cost_usd: row.get(3)?,
                        latency_ms: u64::try_from(latency_ms).unwrap_or(0),
                        status_code: u16::try_from(status_code).unwrap_or(0),
                    })
                })
                .map_err(DbError::Sqlite)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(DbError::Sqlite)?;

            Ok(FinopsStats {
                total_requests,
                total_cost_usd,
                avg_latency_ms,
                latest_logs,
            })
        })
        .await
}

fn has_usage_table(connection: &rusqlite::Connection) -> Result<bool, DbError> {
    connection
        .query_row(
            r#"
            SELECT 1
            FROM sqlite_master
            WHERE type = 'table' AND name = 'llm_request_logs'
            LIMIT 1
            "#,
            [],
            |_| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
        .map_err(DbError::Sqlite)
}
