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

## Evidence Rules

- Unit proof covers pure domain and application rules.
- Integration proof covers backend enforcement, data integrity, provider
  behavior, jobs, or service contracts.
- E2E proof covers user-visible browser flows.
- Platform proof covers only shell, deployment, mobile, desktop, or runtime
  behavior that cannot be proven in lower layers.
- A story can be implemented without every proof column if the story packet
  explains why.
