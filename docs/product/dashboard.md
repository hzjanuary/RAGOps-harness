# Dashboard

Status: implemented by `US-042-pure-cli-dashboard` and extended by
`US-043-unified-live-tui-monitor`.

## Purpose

The dashboard is a pure CLI surface for inspecting FinOps usage stored in
SQLite. It gives developers a fast terminal view of request volume, estimated
cost, average latency, flagged entries, and recent proxy logs without running a
browser or web asset server.

## Runtime Contract

- `cargo run` and `cargo run -- serve` start the proxy and render a live
  dashboard in the foreground every two seconds.
- `cargo run -- dashboard` prints the dashboard once and exits.
- The proxy does not expose dashboard HTTP routes.
- The product no longer serves static files from `ui/`.
- Serve-mode proxy writes and dashboard reads share the same `Database` handle.

## Data Contract

The dashboard opens the configured SQLite database path and queries
`llm_request_logs`.

Aggregate stats:

- Total requests.
- Total estimated cost in USD.
- Average latency in milliseconds.
- Flagged entries where stored status or endpoint metadata suggests a failed or
  vulnerable trace.

Recent logs:

- The five most recent records ordered by `created_at DESC`.
- Display columns: Request ID, Timestamp, Model, Tokens, Latency, Cost USD,
  Status.

## UI Contract

Output is written to stdout:

- Serve mode enters the terminal alternate screen and hides the cursor.
- Serve mode clears and redraws the dashboard in place before each render.
- Ctrl-C exits serve mode and restores the cursor and primary terminal screen.
- ASCII bordered header.
- Summary cards for aggregate stats.
- A `comfy-table` table containing the five latest logs.

## Data Handling

US-042 reads local FinOps metadata only. It does not persist new records, store
prompt or response content, or send local usage data to an external service.

## Non-Goals

- Web dashboard.
- Static frontend assets.
- Authentication.
- Editable settings.
- Report export.
- Persistent dashboard preferences.
- Keyboard-interactive controls.
