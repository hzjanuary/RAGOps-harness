# Overview

## Current Behavior

The harness had no complete root Rust crate for the selected proxy story. The
US-011 story packet existed but its story files were empty, and the generic
architecture document still described stack discovery rather than an accepted
implementation shape.

## Target Behavior

RAGOps Harness runs a local Rust proxy on `0.0.0.0:8000` with:

- `.env` loading before runtime configuration reads.
- `POST /v1/chat/completions` forwarding to OpenAI Chat Completions.
- Usage extraction from OpenAI JSON responses.
- USD cost estimation from prompt, cached prompt, and completion tokens.
- SQLite persistence through a shared `Arc<Mutex<rusqlite::Connection>>`.
- JSON error responses for client, config, upstream, parsing, and logging
  failures.
- `GET /health` for a simple runtime health check.

## Affected Users

- Developers running RAG applications through a local proxy.
- Hackathon/demo operators who need local cost and token usage evidence.
- Future dashboard agents that will read the SQLite usage records.

## Affected Product Docs

- `docs/product/finops-proxy.md`
- `docs/ARCHITECTURE.md`
- `docs/TEST_MATRIX.md`
- `docs/decisions/0004-rust-axum-sqlite-core-proxy.md`

## Non-Goals

- Streaming Chat Completions.
- Multi-provider routing.
- Dashboard or Tauri UI.
- CLI command surface.
- Prompt/response content retention.
- Live OpenAI integration proof without a configured API key.
