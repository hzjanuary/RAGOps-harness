# Exec Plan

## Goal

Implement the first complete Rust vertical slice for the RAGOps Harness core
proxy and FinOps logger.

## Scope

In scope:

- Root `Cargo.toml`.
- `src/main.rs` Axum server startup on `0.0.0.0:8000`.
- `src/proxy.rs` request validation, OpenAI forwarding, usage parsing, cost
  calculation, JSON error responses, and DB logging.
- `src/db.rs` SQLite connection setup, schema creation, and usage log insert.
- Product/story/test-matrix evidence updates.

Out of scope:

- Tauri dashboard.
- CLI command parser.
- Multi-provider adapters.
- Streaming completions.
- Live provider integration run without user-provided API credentials.

## Risk Classification

Lane: high-risk.

Risk flags:

- Data model: creates durable SQLite usage records.
- Audit/security: logs FinOps data and handles provider credentials.
- External systems: forwards traffic to OpenAI.
- Public contracts: exposes local HTTP routes and JSON error shape.
- Weak proof: live OpenAI integration depends on external credentials/network.

Hard gates:

- Audit/security.
- External provider behavior.

Human confirmation was not required before implementation because the user
explicitly selected US-011 and specified Rust, Axum-compatible behavior, OpenAI
Chat Completions, SQLite, and the required files.

## Work Phases

1. Discovery: read harness intake, architecture, test matrix, product docs, and
   the empty US-011 story packet.
2. Design: define local HTTP, SQLite, cost, and error contracts.
3. Validation planning: add unit proof for pricing and SQLite logging; use
   `cargo check` as type proof.
4. Implementation: add root Rust crate and complete `main`, `proxy`, and `db`
   modules.
5. Verification: run formatting, typecheck, and unit tests.
6. Harness update: update product doc, story packet, test matrix, and decision
   log.

## Stop Conditions

Pause for human confirmation if:

- Streaming support becomes mandatory for US-011.
- Prompt/response content storage is requested.
- Multi-provider support is added to this story.
- Validation requirements need to be weakened.
- A live OpenAI integration run should be performed with real credentials.
