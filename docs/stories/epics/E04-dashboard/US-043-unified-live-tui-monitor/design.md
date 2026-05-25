# Design

## Domain Model

- `DashboardReport` contains total requests, total USD cost, average latency,
  flagged entry count, and the latest log rows.
- `LatestLog` contains request id, timestamp, model, total tokens, latency,
  cost, and status code.

## Application Flow

1. `main.rs` parses CLI arguments with `clap`.
2. No arguments and `serve` both enter the unified live monitor flow.
3. The serve flow opens one `Database`.
4. The Axum `Router` receives a clone of that `Database` through
   `proxy::AppState`.
5. `main.rs` captures `axum::serve(listener, app)` and spawns it with
   `tokio::spawn`.
6. The foreground task loops forever, calling
   `dashboard::run_dashboard(&db)` and sleeping for two seconds.
7. `dashboard` remains one-shot by opening `Database` and calling the same
   dashboard renderer once.

## Interface Contract

Commands:

- `serve`
- `dashboard`

The proxy route surface remains:

- `GET /health`
- `POST /v1/chat/completions`

No dashboard HTTP route is added.

## Data Model

No schema changes are introduced. The dashboard reads from `llm_request_logs`.

## UI / Platform Impact

The live monitor uses `crossterm` to enter the terminal alternate screen, hide
the cursor, clear the screen in place before each render, and restore the
terminal on Ctrl-C. `dashboard.rs` only prints the dashboard content: an ASCII
header, fixed summary cards, and a `comfy-table` table with the five latest
requests.

## Observability

Dashboard render failures are logged with `tracing::error` and do not stop the
proxy task.

## Alternatives Considered

1. Keep `serve` proxy-only and add a separate watch command. Rejected because
   the selected operational flow is a unified local monitor.
2. Reopen SQLite independently on every dashboard refresh. Rejected because the
   existing shared `Database` handle already coordinates access.
3. Add a dashboard HTTP API. Rejected because US-042 retired that surface.
