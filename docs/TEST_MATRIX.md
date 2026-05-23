# Test Matrix

This file maps product behavior to proof. Do not mark a row implemented until
tests or validation evidence exist.

## Status Values

| Status | Meaning |
| --- | --- |
| planned | Accepted as intended behavior, not implemented |
| in_progress | Actively being built |
| implemented | Implemented and proof exists |
| changed | Contract changed after earlier implementation |
| retired | No longer part of the product contract |

## Matrix

| Story | Contract | Unit | Integration | E2E | Platform | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| US-011 Core Proxy & FinOps Logger | Local OpenAI Chat Completions proxy loads `.env`, logs usage, and estimates cost to SQLite | yes | no | no | yes | implemented | `cargo check --target-dir /tmp/ragops-harness-target`; `cargo test --target-dir /tmp/ragops-harness-target` passed 3 tests; `cargo run --target-dir /tmp/ragops-harness-target` logged `.env` configuration loaded from the project root; `curl -sS http://127.0.0.1:8000/health` returned healthy JSON on 2026-05-22. Live OpenAI integration not run. |
| US-021 Red Teaming Engine | CLI scan mode sends five concurrent malicious OpenAI-format chat requests to a target endpoint and reports Safe, Vulnerable, and Error counts | yes | no | no | yes | implemented | `cargo check --target-dir /tmp/ragops-harness-target` passed; `cargo test --target-dir /tmp/ragops-harness-target` passed 6 tests; `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed `serve` and `scan`; `cargo run --target-dir /tmp/ragops-harness-target -- scan http://127.0.0.1:9/v1/chat/completions` printed a 5-case terminal report with 5 expected connection errors. Live target scan not run. |
| US-031 RAG Evaluation Engine | CLI eval mode reads a local JSON dataset, sends each answer/context pair to OpenAI as a Faithfulness judge request, and reports total records plus average score | yes | no | no | yes | implemented | `cargo check --target-dir /tmp/ragops-harness-target` passed; `cargo test --target-dir /tmp/ragops-harness-target` passed 9 tests; `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed `eval`; `cargo run --target-dir /tmp/ragops-harness-target -- eval --help` listed `--dataset`. Live OpenAI evaluation not run. |
| US-041 Web GUI Dashboard | Axum serve mode exposes `GET /api/stats`, serves `ui/index.html`, and renders FinOps stat cards plus a bar chart from the proxy database | no | yes | no | yes | retired | Superseded by US-042 on 2026-05-23. `/api/stats`, `tower-http`, and tracked `ui/` assets were removed from the product surface. Historical validation remains in the US-041 story packet. |
| US-042 Pure CLI Dashboard | CLI dashboard mode reads local SQLite FinOps metadata, prints aggregate stats, and renders the five latest logs as a terminal table | no | yes | no | yes | implemented | `cargo fmt --all` passed; `cargo check --target-dir /tmp/ragops-harness-target` passed; `cargo test --target-dir /tmp/ragops-harness-target` passed 9 tests; `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed `dashboard`; `RAGOPS_DB_PATH=/tmp/ragops-cli-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- dashboard` printed the CLI report with zero-count stats and a table header. |

## Evidence Rules

- Unit proof covers pure domain and application rules.
- Integration proof covers backend enforcement, data integrity, provider
  behavior, jobs, or service contracts.
- E2E proof covers user-visible browser flows.
- Platform proof covers only shell, deployment, mobile, desktop, or runtime
  behavior that cannot be proven in lower layers.
- A story can be implemented without every proof column if the story packet
  explains why.
