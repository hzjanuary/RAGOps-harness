# Overview

## Current Behavior

US-041 serves a web dashboard from the Axum proxy with `GET /api/stats` and
static files under `ui/`.

## Target Behavior

RAGOps Harness replaces the web dashboard with `cargo run -- dashboard`, a
one-shot terminal report that reads local SQLite FinOps metadata and prints
aggregate stats plus the five latest logs in a table.

## Affected Users

- Developers inspecting local LLM usage cost and latency from a terminal.
- Hackathon operators who need a low-dependency demo surface.

## Affected Product Docs

- `docs/product/dashboard.md`
- `docs/ARCHITECTURE.md`
- `docs/TEST_MATRIX.md`
- `docs/decisions/0008-pure-cli-dashboard.md`

## Non-Goals

- Browser UI.
- Static frontend assets.
- Dashboard HTTP API.
- Live updating or watch mode.
