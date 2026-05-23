# Dashboard

Status: implemented by `US-042-pure-cli-dashboard`.

## Purpose

The dashboard is a pure CLI report for inspecting FinOps usage stored in
SQLite. It gives developers a fast terminal view of request volume, estimated
cost, average latency, and recent proxy logs without running a browser or web
asset server.

## Runtime Contract

- `cargo run -- dashboard` prints the report once and exits.
- `cargo run` and `cargo run -- serve` continue to run the proxy.
- The proxy does not expose dashboard HTTP routes.
- The product no longer serves static files from `ui/`.

## Data Contract

The dashboard opens the configured SQLite database path and queries
`llm_request_logs`.

Aggregate stats:

- Total requests.
- Total estimated cost in USD.
- Average latency in milliseconds.

Recent logs:

- The five most recent records ordered by `created_at DESC`.
- Display columns: Request ID, Model, Tokens, Latency (ms), Cost (USD).

## UI Contract

Output is written to stdout:

- Header: `=== RAGOps FinOps CLI Report ===`
- Summary lines for aggregate stats.
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
