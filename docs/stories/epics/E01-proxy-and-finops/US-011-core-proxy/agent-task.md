# Agent Task

## Request

Implement the complete Rust codebase for US-011 Core Proxy & FinOps Logger.

Required files:

- `Cargo.toml`
- `src/db.rs`
- `src/proxy.rs`
- `src/main.rs`

## Intake Result

Input type: spec slice.

Lane: high-risk.

Reason: the task introduces provider forwarding, API key handling, local
FinOps/audit records, a SQLite schema, and a public local HTTP contract.

## Implementation Notes

- The root crate is the implementation target for the requested `src/*.rs`
  files.
- The nested untracked `rag-harness/` directory was not used or modified.
- The proxy supports non-streaming OpenAI Chat Completions only.
- The FinOps logger stores usage/cost metadata, not prompt or completion
  content.
- Unknown model pricing does not block proxying; it logs `cost_usd = 0.0` and
  `pricing_known = 0`.

## Validation

Completed:

- `cargo fmt --all`
- `cargo check --target-dir /tmp/ragops-harness-target`
- `cargo test --target-dir /tmp/ragops-harness-target`
- `cargo run --target-dir /tmp/ragops-harness-target`
- `curl -sS http://127.0.0.1:8000/health`

Not attempted:

- Live OpenAI call, because no API key was provided and it would incur external
  cost.
