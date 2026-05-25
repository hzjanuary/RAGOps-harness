# Overview

## Current Behavior

`serve` runs the Axum proxy as a blocking foreground process, and `dashboard`
prints a one-shot CLI report in a separate command.

## Target Behavior

RAGOps Harness unifies local operations in `cargo run -- serve`:

- The Axum proxy binds to `0.0.0.0:8000` and runs on a background Tokio task.
- The foreground task continuously renders a live CLI dashboard every two
  seconds.
- The server and dashboard share the same `Database` handle so proxy writes and
  dashboard reads coordinate through the existing SQLite connection.
- `cargo run -- dashboard` remains a one-shot report for users who only need a
  snapshot.

## Affected Users

- Developers running the local proxy while watching cost and latency.
- Hackathon operators demonstrating proxy traffic in one terminal.
- Future agents extending terminal-first operations workflows.

## Affected Product Docs

- `docs/product/dashboard.md`
- `docs/product/finops-proxy.md`
- `docs/ARCHITECTURE.md`
- `docs/TEST_MATRIX.md`
- `docs/decisions/0009-unified-live-tui-monitor.md`

## Non-Goals

- Browser UI.
- Dashboard HTTP API.
- Interactive keyboard controls.
- Persistent dashboard preferences.
