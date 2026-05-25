# 0009 Unified Live TUI Monitor

Date: 2026-05-25

## Status

Accepted

## Context

US-042 moved dashboard reporting from a retired web UI into a one-shot CLI
command. The selected next step is a single local operations mode where the
proxy keeps serving requests while the foreground terminal shows live FinOps
telemetry.

## Decision

`serve` now starts the Axum proxy as a background Tokio task and keeps the main
foreground task in a two-second live dashboard render loop.

The live monitor:

- Opens one `Database` handle during serve startup.
- Passes a clone of that handle into Axum proxy state.
- Calls `dashboard::run_dashboard(&db)` from the foreground loop.
- Uses `crossterm` alternate-screen rendering to avoid writing each refresh to
  terminal scrollback.
- Restores the cursor and primary terminal screen on Ctrl-C.
- Logs dashboard render failures without stopping the proxy task.
- Keeps dashboard rendering independent of Axum web structures.

## Alternatives Considered

1. Add a separate `watch` command. Rejected because the selected operator flow
   is `serve` plus foreground monitoring.
2. Keep `serve` proxy-only. Rejected because users would need a second process
   to observe local traffic.
3. Reintroduce a dashboard HTTP API. Rejected because US-042 explicitly retired
   the web dashboard surface.

## Consequences

Positive:

- One terminal can run the proxy and show live usage.
- Proxy writes and dashboard reads share the existing SQLite connection guard.
- The dashboard module remains CLI-only.

Tradeoffs:

- `serve` no longer leaves proxy logs as the main foreground output.
- The live dashboard has no keyboard controls or alternate refresh interval.

## Follow-Up

- Add interactive controls only after a terminal interaction contract is
  selected.
