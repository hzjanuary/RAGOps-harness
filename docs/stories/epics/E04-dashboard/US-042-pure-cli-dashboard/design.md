# Design

## Domain Model

- `DashboardReport` contains total requests, total USD cost, average latency,
  and the latest log rows.
- `LatestLog` contains request id, model, total tokens, latency, and cost.

## Application Flow

1. `main.rs` parses `dashboard` with `clap`.
2. The command reads `RAGOPS_DB_PATH`, defaulting to `ragops_harness.sqlite3`.
3. `dashboard::run_dashboard(db_path)` opens the SQLite database with
   `Database::open`.
4. The report queries aggregate stats and the five latest logs.
5. The command prints a header, summary stats, and a `comfy-table` table.

## Interface Contract

Command:

```text
ragops-harness dashboard
```

There are no dashboard HTTP routes.

## Data Model

No schema changes are introduced. The report reads from `llm_request_logs`.

## UI / Platform Impact

The new surface is terminal output only. `ui/` static assets are removed from
the tracked product and `tower-http` is no longer a direct dependency.

## Observability

The dashboard writes the report to stdout and does not write audit records or
operational logs.

## Alternatives Considered

1. Keep the web dashboard alongside CLI output. Rejected by the pivot.
2. Keep `/api/stats`. Rejected because no active dashboard client remains.
3. Add watch mode. Deferred until a live CLI contract is selected.
