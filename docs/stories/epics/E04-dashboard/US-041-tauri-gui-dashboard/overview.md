# Overview

## Current Behavior

The project has CLI and proxy surfaces, but no graphical way to inspect local
FinOps usage.

## Target Behavior

RAGOps Harness serves a web dashboard from the existing Axum server. Running
`cargo run` or `cargo run -- serve` exposes `ui/index.html` at
`http://localhost:8000/` and exposes `GET /api/stats` for local FinOps metrics.

## Affected Users

- Developers inspecting local LLM usage cost and latency.
- Future agents expanding local reporting workflows.

## Affected Product Docs

- `docs/product/dashboard.md`
- `docs/ARCHITECTURE.md`
- `docs/TEST_MATRIX.md`
- `docs/decisions/0007-tauri-dashboard.md`

## Non-Goals

- Tauri or native desktop packaging.
- Authentication.
- Persisted dashboard settings.
- Separate frontend dev server.
