# Validation

## Proof Strategy

US-011 proof is split into local deterministic checks and deferred live provider
proof. Local proof must show the Rust crate compiles, cost calculation handles
known and unknown pricing, and SQLite logging writes a record. Live OpenAI proof
requires a valid `OPENAI_API_KEY` and is not run by default.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | GPT-4o mini cached-token cost calculation; unknown model marks pricing unknown. |
| Integration | In-memory SQLite schema creation and usage log insert. |
| E2E | Not covered in US-011; no browser or CLI flow exists yet. |
| Platform | Server binds to `0.0.0.0:8000`; `/health` returns JSON. |
| Performance | Not covered; no load target exists yet. |
| Logs/Audit | SQLite `llm_request_logs` write covered by test; tracing initialized by startup path. |
| Configuration | Startup loads `.env` before reading `OPENAI_API_KEY` and does not log secret values. |

## Fixtures

- In-memory SQLite database.
- `gpt-4o-mini-2024-07-18` pricing fixture with prompt, cached prompt, and
  completion token counts.
- Unknown model fixture to verify non-blocking `pricing_known = false`.

## Commands

```text
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
cargo run --target-dir /tmp/ragops-harness-target
curl -sS http://127.0.0.1:8000/health
```

## Acceptance Evidence

- `cargo fmt --all` completed with no output.
- `cargo check --target-dir /tmp/ragops-harness-target` passed.
- `cargo test --target-dir /tmp/ragops-harness-target` passed: 3 tests, 0
  failures.
- `cargo run --target-dir /tmp/ragops-harness-target` started the server on
  `0.0.0.0:8000` and logged that `.env` configuration loaded from the project
  root.
- `curl -sS http://127.0.0.1:8000/health` returned
  `{"status":"ok","service":"ragops-harness"}`.

Not attempted:

- Live `POST /v1/chat/completions` against OpenAI. This requires a real
  `OPENAI_API_KEY` and would incur provider cost.
