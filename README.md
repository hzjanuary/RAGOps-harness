# RAGOps Harness

RAGOps Harness is a local-first LLMOps harness for Retrieval-Augmented
Generation applications. The current implementation provides a Rust HTTP proxy
that forwards OpenAI Chat Completions requests, records token usage and latency,
estimates cost, and stores FinOps records in SQLite.

The project is intentionally small right now. The harness docs define how future
agents should turn product ideas into validated story-sized work before adding
larger CLI, evaluation, red teaming, or desktop features.

## Status

Current milestone: US-021 Red Teaming Engine.

Implemented:

- Rust backend with Axum.
- CLI dispatch with `serve` and `scan` subcommands.
- Local proxy server on `0.0.0.0:8000`.
- `GET /health` runtime health check.
- `POST /v1/chat/completions` forwarding to OpenAI Chat Completions.
- `.env` loading before runtime configuration reads.
- SQLite usage logging for token counts, latency, model, upstream id, and
  estimated USD cost.
- JSON error envelopes for client, configuration, upstream, parsing, and local
  logging failures.
- Red-team scanner that sends five concurrent malicious OpenAI-compatible chat
  requests to a target endpoint and prints Safe/Vulnerable/Error counts.

Planned product areas:

- RAG evaluation pipeline.
- Additional CLI workflows.
- Tauri desktop dashboard for local reports.
- Multi-provider support.

## Why This Exists

RAG systems often fail quietly. They can spend more than expected, respond too
slowly, drift in quality, or become vulnerable to prompt injection. RAGOps
Harness is designed to sit between a RAG application and an LLM provider so
developers can inspect cost, latency, quality, and security behavior locally.

The first slice focuses on FinOps visibility because it gives immediate runtime
evidence without storing prompt or completion content.

## Architecture

```text
RAG app
  -> RAGOps Harness local proxy
      -> OpenAI Chat Completions
      -> SQLite FinOps log
```

Runtime stack:

- Rust
- Axum
- Reqwest
- Rusqlite
- Tokio
- SQLite

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and
[docs/decisions/0004-rust-axum-sqlite-core-proxy.md](docs/decisions/0004-rust-axum-sqlite-core-proxy.md)
for the accepted implementation shape.

## Quick Start

Prerequisites:

- Rust toolchain.
- An OpenAI API key.

Create a local `.env` file:

```bash
OPENAI_API_KEY=sk-your-key-here
# Optional:
RAGOPS_DB_PATH=ragops_harness.sqlite3
```

Run the proxy:

```bash
cargo run
```

The explicit proxy command is also supported:

```bash
cargo run -- serve
```

Check health:

```bash
curl -sS http://127.0.0.1:8000/health
```

Expected response:

```json
{"status":"ok","service":"ragops-harness"}
```

Send a non-streaming Chat Completions request through the local proxy:

```bash
curl -sS http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [
      { "role": "user", "content": "Say hello in one short sentence." }
    ]
  }'
```

The proxy uses `OPENAI_API_KEY` from the process environment or `.env` and sends
it upstream. The local client request does not need to include the provider key.

Run the red-team scanner against an OpenAI-compatible endpoint:

```bash
cargo run -- scan http://127.0.0.1:8000/v1/chat/completions
```

The scanner sends five concurrent malicious prompts using the OpenAI Chat
Completions JSON shape and prints a terminal report with Safe, Vulnerable, and
Error counts.

## Configuration

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `OPENAI_API_KEY` | Yes for proxy requests | None | OpenAI API key used for upstream calls. |
| `RAGOPS_DB_PATH` | No | `ragops_harness.sqlite3` | SQLite database path for local usage logs. |

Configuration rules:

- `.env` is loaded at startup before configuration is read.
- Exported environment variables take precedence over `.env` values.
- Missing `OPENAI_API_KEY` does not stop the server from booting, but proxy
  requests return a JSON configuration error until the key is set and the
  process is restarted.
- `.env` is ignored by git and should never be committed.

## Data Logged

RAGOps Harness stores usage metadata only:

- Local request id.
- Timestamp.
- Provider.
- Endpoint.
- Model.
- Upstream response id.
- Prompt tokens.
- Completion tokens.
- Total tokens.
- Cached prompt tokens when reported.
- Estimated cost in USD.
- Latency in milliseconds.
- Upstream success status code.
- Whether the model matched a known pricing rule.

Prompt and completion content are not persisted by the proxy. Red-team scan
prompts and responses are printed to stdout only and are not persisted.

## Validation

Run the local checks:

```bash
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
```

Current test coverage includes:

- Cost calculation with cached prompt tokens.
- Unknown model pricing behavior.
- SQLite usage log insertion.
- Red-team refusal marker detection.
- OpenAI-style response text extraction.
- CLI help and scan-report smoke checks are run manually as platform proof.

Live OpenAI integration is intentionally manual because it requires real
credentials and may incur provider cost.

## Project Layout

```text
src/
  main.rs      CLI dispatch for serve and scan modes
  proxy.rs     Chat Completions proxy, usage parsing, cost estimation, errors
  db.rs        SQLite connection setup, schema, and usage log persistence
  red_team.rs  Concurrent malicious-prompt scanner and terminal report

docs/
  HARNESS.md        Human-agent operating model
  FEATURE_INTAKE.md Task classification and risk lanes
  ARCHITECTURE.md   Architecture rules and selected stack
  TEST_MATRIX.md    Behavior-to-proof status
  product/          Product contracts
  stories/          Story packets and evidence
  decisions/        Architecture and product decision records
```

## Current Limitations

- OpenAI Chat Completions only.
- Streaming requests are rejected.
- No incoming proxy authentication yet.
- No dashboard yet.
- Pricing constants must be maintained as provider pricing changes.
- No provider retry policy beyond returning a graceful JSON error.

## Security Notes

- Keep `.env` local.
- Do not paste provider keys into prompts, docs, issues, or shared logs.
- Rotate the API key if it was ever committed or shared.
- The proxy logs operational metadata and FinOps records, not prompt or
  completion bodies.

## License

No project license has been selected yet.
