# 0004 Rust Axum SQLite Core Proxy

Date: 2026-05-22

## Status

Accepted

## Context

US-011 is the first implementation story for RAGOps Harness. The repository's
generic architecture document intentionally had no selected application stack,
but the selected story and user request require a local Rust proxy that forwards
OpenAI Chat Completions traffic and logs FinOps usage data locally.

The implementation must handle external provider behavior, local audit/FinOps
records, and a public local HTTP contract while keeping the first slice small
enough for hackathon delivery.

## Decision

Use a root Rust crate with:

- Axum for the local HTTP server.
- Reqwest for outbound OpenAI requests.
- Rusqlite with a single `Arc<tokio::sync::Mutex<rusqlite::Connection>>` for
  SQLite access.
- Tokio for async runtime.
- Tracing for operational logs.

The first route surface is:

- `GET /health`
- `POST /v1/chat/completions`

US-011 rejects streaming requests, stores usage/cost metadata only, and returns
JSON errors for local and upstream failures.

## Alternatives Considered

1. Start with a CLI command shell before the proxy. Rejected because US-011 is
   explicitly the core proxy and logger.
2. Add a SQLite pool or migration framework. Deferred because the story requires
   `Arc<Mutex<>>` and only one schema is needed.
3. Store full request/response bodies for richer debugging. Rejected for the
   first slice to reduce sensitive-data retention.

## Consequences

Positive:

- The project now has a compilable backend implementation.
- Future dashboard work can read from a stable SQLite table.
- Provider credentials can come from the process environment or local `.env`,
  remain process-local, and are not persisted.
- The local proxy can fail with machine-readable JSON errors.

Tradeoffs:

- One mutex-protected SQLite connection serializes log writes.
- Streaming completions are unsupported until a later story.
- Pricing constants must be maintained as provider pricing changes.
- Live provider proof requires external credentials and may incur cost.

## Follow-Up

- Add a mock-provider integration test before widening provider support.
- Add a pricing update policy or configurable pricing table.
- Add streaming support only after the logging strategy for streamed usage is
  defined.
