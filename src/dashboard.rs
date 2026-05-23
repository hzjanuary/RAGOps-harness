use comfy_table::{Cell, Row, Table};

use crate::db::{Database, DbError};

type DashboardResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct DashboardReport {
    total_requests: u64,
    total_cost_usd: f64,
    avg_latency_ms: f64,
    latest_logs: Vec<LatestLog>,
}

#[derive(Debug)]
struct LatestLog {
    request_id: String,
    model: String,
    total_tokens: u64,
    latency_ms: u64,
    cost_usd: f64,
}

pub async fn run_dashboard(db_path: &str) -> DashboardResult<()> {
    let db = Database::open(db_path)?;
    let report = db.with_connection(load_dashboard_report).await?;

    println!("=== RAGOps FinOps CLI Report ===");
    println!("Database: {db_path}");
    println!("Total Requests: {}", report.total_requests);
    println!("Total Cost (USD): {:.6}", report.total_cost_usd);
    println!("Average Latency (ms): {:.2}", report.avg_latency_ms);
    println!();

    let mut table = Table::new();
    table.set_header(Row::from(vec![
        Cell::new("Request ID"),
        Cell::new("Model"),
        Cell::new("Tokens"),
        Cell::new("Latency (ms)"),
        Cell::new("Cost (USD)"),
    ]));

    for log in report.latest_logs {
        table.add_row(Row::from(vec![
            Cell::new(log.request_id),
            Cell::new(log.model),
            Cell::new(log.total_tokens),
            Cell::new(log.latency_ms),
            Cell::new(format!("{:.6}", log.cost_usd)),
        ]));
    }

    println!("{table}");

    Ok(())
}

fn load_dashboard_report(connection: &rusqlite::Connection) -> Result<DashboardReport, DbError> {
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
                model,
                total_tokens,
                latency_ms,
                cost_usd
            FROM llm_request_logs
            ORDER BY created_at DESC
            LIMIT 5
            "#,
        )
        .map_err(DbError::Sqlite)?;

    let latest_logs = statement
        .query_map([], |row| {
            let total_tokens: i64 = row.get(2)?;
            let latency_ms: i64 = row.get(3)?;

            Ok(LatestLog {
                request_id: row.get(0)?,
                model: row.get(1)?,
                total_tokens: u64::try_from(total_tokens).unwrap_or(0),
                latency_ms: u64::try_from(latency_ms).unwrap_or(0),
                cost_usd: row.get(4)?,
            })
        })
        .map_err(DbError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DbError::Sqlite)?;

    Ok(DashboardReport {
        total_requests,
        total_cost_usd,
        avg_latency_ms,
        latest_logs,
    })
}
