# Design

## Domain Model

- `FinopsStats` contains aggregate dashboard metrics.
- `LatestLog` contains the fields needed to render the latest usage list and
  cost chart.

## Application Flow

1. `main.rs` parses CLI arguments with `clap`.
2. `serve` calls `proxy::run()`.
3. `proxy::run()` builds the Axum router with existing proxy routes,
   `GET /api/stats`, and `ServeDir::new("ui")` as the fallback service.
4. `ui/index.html` calls `fetch("/api/stats")`.
5. `api_stats` reads the database from `crate::proxy::AppState`.
6. If `llm_request_logs` exists, it aggregates totals and loads the five latest
   rows.
7. The UI parses the JSON response and renders stat cards, latest logs, and a
   bar chart.

## Interface Contract

HTTP:

- `GET /`
- `GET /api/stats`
- `GET /health`
- `POST /v1/chat/completions`

Stats response:

```json
{
  "total_requests": 0,
  "total_cost_usd": 0.0,
  "avg_latency_ms": 0.0,
  "latest_logs": []
}
```

## Data Model

No tables or migrations are added. The dashboard reads existing
`llm_request_logs` rows from the proxy database connection.

## UI / Platform Impact

The new surface is a browser UI served by Axum. Existing proxy routes and CLI
subcommands are unchanged.

## Observability

Dashboard data is rendered in the local UI. No new audit or operational logs
are introduced.

## Alternatives Considered

1. Keep Tauri. Rejected because the current Linux environment no longer
   provides the WebKitGTK 4.0 package set required by Tauri 1.x.
2. Frontend package build. Rejected to keep the first GUI slice static.
3. Separate dashboard server. Rejected because the existing Axum server can
   serve the static UI and API.
