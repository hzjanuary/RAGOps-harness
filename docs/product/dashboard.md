# Dashboard

Status: implemented by `US-041-web-gui-dashboard`.

## Purpose

The dashboard is a local web surface for inspecting FinOps usage stored in
SQLite. It gives developers a quick view of request volume, estimated cost,
average latency, and recent proxy logs without requiring desktop WebKitGTK
packages.

## Runtime Contract

- `cargo run` defaults to serve mode and hosts the dashboard.
- `cargo run -- serve` hosts the dashboard explicitly.
- The dashboard is available at `http://localhost:8000/`.
- The stats API is available at `GET /api/stats`.
- Existing `GET /health` and `POST /v1/chat/completions` routes remain
  unchanged.

## Data Contract

`GET /api/stats` returns JSON:

```json
{
  "total_requests": 0,
  "total_cost_usd": 0.0,
  "avg_latency_ms": 0.0,
  "latest_logs": []
}
```

Each latest log includes:

- `request_id`
- `created_at`
- `model`
- `cost_usd`
- `latency_ms`
- `status_code`

The dashboard reads from the proxy `AppState` database connection and table
`llm_request_logs`.

## UI Contract

`ui/index.html` is a static HTML dashboard using Tailwind CDN and Chart.js CDN.
It calls `fetch("/api/stats")`, renders three stat cards, renders the five
latest logs, and renders a bar chart for latest request costs.

## Data Handling

US-041 reads local FinOps metadata only. It does not persist new records, store
prompt or response content, or send local usage data to an external service.

## Non-Goals

- Tauri or native desktop packaging.
- Authentication.
- Editable settings.
- Multi-database selection.
- Report export.
- Persistent dashboard preferences.
