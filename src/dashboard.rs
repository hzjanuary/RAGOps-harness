use comfy_table::{presets::ASCII_FULL, Cell, CellAlignment, Color, Row, Table};

use crate::db::{Database, DbError};

type DashboardResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct DashboardReport {
    total_requests: u64,
    total_cost_usd: f64,
    avg_latency_ms: f64,
    flagged_entries: u64,
    latest_logs: Vec<LatestLog>,
}

#[derive(Debug)]
struct LatestLog {
    request_id: String,
    created_at: String,
    model: String,
    total_tokens: u64,
    latency_ms: u64,
    cost_usd: f64,
    status_code: u16,
}

pub async fn run_dashboard(db: &Database) -> DashboardResult<()> {
    let report = db.with_connection(load_dashboard_report).await?;

    print_header();
    print_summary_cards(&report);
    print_latest_logs_table(&report.latest_logs);

    Ok(())
}

fn print_header() {
    println!("+======================================================================+");
    println!("|                    RAGOps Live FinOps Monitor                       |");
    println!("|              Local OpenAI Proxy + SQLite Usage Telemetry            |");
    println!("+======================================================================+");
    println!();
}

fn print_summary_cards(report: &DashboardReport) {
    let total_requests = report.total_requests.to_string();
    let total_cost = format!("${:.6}", report.total_cost_usd);
    let avg_latency = format!("{:.2} ms", report.avg_latency_ms);
    let flagged_entries = report.flagged_entries.to_string();

    println!("+------------------+------------------+------------------+------------------+");
    println!("| Total Requests   | Total Cost       | Avg Latency      | Flagged Entries  |");
    println!(
        "| {:>16} | {:>16} | {:>16} | {:>16} |",
        total_requests, total_cost, avg_latency, flagged_entries
    );
    println!("+------------------+------------------+------------------+------------------+");
    println!();
}

fn print_latest_logs_table(latest_logs: &[LatestLog]) {
    let mut table = Table::new();
    table.load_preset(ASCII_FULL);
    table.set_header(Row::from(vec![
        Cell::new("Request ID").fg(Color::Cyan),
        Cell::new("Timestamp").fg(Color::Cyan),
        Cell::new("Model").fg(Color::Cyan),
        Cell::new("Tokens")
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
        Cell::new("Latency")
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
        Cell::new("Cost USD")
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
        Cell::new("Status")
            .fg(Color::Cyan)
            .set_alignment(CellAlignment::Right),
    ]));

    if latest_logs.is_empty() {
        table.add_row(Row::from(vec![
            Cell::new("-"),
            Cell::new("No requests logged yet"),
            Cell::new("-"),
            Cell::new("0").set_alignment(CellAlignment::Right),
            Cell::new("0 ms").set_alignment(CellAlignment::Right),
            Cell::new("0.000000").set_alignment(CellAlignment::Right),
            Cell::new("-").set_alignment(CellAlignment::Right),
        ]));
    } else {
        for log in latest_logs {
            table.add_row(Row::from(vec![
                Cell::new(truncate_middle(&log.request_id, 24)),
                Cell::new(&log.created_at),
                Cell::new(&log.model),
                Cell::new(log.total_tokens).set_alignment(CellAlignment::Right),
                Cell::new(format!("{} ms", log.latency_ms)).set_alignment(CellAlignment::Right),
                Cell::new(format!("{:.6}", log.cost_usd)).set_alignment(CellAlignment::Right),
                Cell::new(log.status_code).set_alignment(CellAlignment::Right),
            ]));
        }
    }

    println!("Latest Requests");
    println!("{table}");
}

fn load_dashboard_report(connection: &rusqlite::Connection) -> Result<DashboardReport, DbError> {
    let (total_requests, total_cost_usd, avg_latency_ms, flagged_entries) = connection
        .query_row(
            r#"
            SELECT
                COUNT(*),
                COALESCE(SUM(cost_usd), 0.0),
                COALESCE(AVG(latency_ms), 0.0),
                COALESCE(SUM(
                    CASE
                        WHEN status_code >= 400
                            OR LOWER(endpoint) LIKE '%vulnerable%'
                            OR LOWER(endpoint) LIKE '%prompt-injection%'
                            OR LOWER(endpoint) LIKE '%jailbreak%'
                        THEN 1
                        ELSE 0
                    END
                ), 0)
            FROM llm_request_logs
            "#,
            [],
            |row| {
                let total_requests: i64 = row.get(0)?;
                let total_cost_usd: f64 = row.get(1)?;
                let avg_latency_ms: f64 = row.get(2)?;
                let flagged_entries: i64 = row.get(3)?;

                Ok((
                    u64::try_from(total_requests).unwrap_or(0),
                    total_cost_usd,
                    avg_latency_ms,
                    u64::try_from(flagged_entries).unwrap_or(0),
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
                total_tokens,
                latency_ms,
                cost_usd,
                status_code
            FROM llm_request_logs
            ORDER BY created_at DESC
            LIMIT 5
            "#,
        )
        .map_err(DbError::Sqlite)?;

    let latest_logs = statement
        .query_map([], |row| {
            let total_tokens: i64 = row.get(3)?;
            let latency_ms: i64 = row.get(4)?;
            let status_code: i64 = row.get(6)?;

            Ok(LatestLog {
                request_id: row.get(0)?,
                created_at: row.get(1)?,
                model: row.get(2)?,
                total_tokens: u64::try_from(total_tokens).unwrap_or(0),
                latency_ms: u64::try_from(latency_ms).unwrap_or(0),
                cost_usd: row.get(5)?,
                status_code: u16::try_from(status_code).unwrap_or(0),
            })
        })
        .map_err(DbError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DbError::Sqlite)?;

    Ok(DashboardReport {
        total_requests,
        total_cost_usd,
        avg_latency_ms,
        flagged_entries,
        latest_logs,
    })
}

fn truncate_middle(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_owned();
    }

    if max_len <= 3 {
        return ".".repeat(max_len);
    }

    let side_len = (max_len - 3) / 2;
    let tail_len = max_len - 3 - side_len;
    let head = &value[..side_len];
    let tail = &value[value.len() - tail_len..];

    format!("{head}...{tail}")
}
