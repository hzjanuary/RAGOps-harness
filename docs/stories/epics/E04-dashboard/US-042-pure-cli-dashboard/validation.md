# Validation

## Proof Strategy

US-042 proof must show the crate compiles without the web dashboard dependency,
existing tests still pass, CLI help exposes `dashboard`, and the dashboard can
open a local SQLite database and print a report.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | Existing FinOps, red-team, and eval tests remain green. |
| Integration | Dashboard opens SQLite through `Database` and queries initialized `llm_request_logs`. |
| E2E | Not covered; no browser flow remains. |
| Platform | CLI help lists `dashboard`; dashboard command prints report and table. |
| Performance | Not covered; report is limited to five latest logs. |
| Logs/Audit | Dashboard is read-only and writes no new audit records. |

## Fixtures

- Empty smoke database at `/tmp/ragops-cli-dashboard-smoke.sqlite`.

## Commands

```text
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
cargo run --target-dir /tmp/ragops-harness-target -- --help
RAGOPS_DB_PATH=/tmp/ragops-cli-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- dashboard
```

## Acceptance Evidence

- `cargo check --target-dir /tmp/ragops-harness-target` passed.
- `cargo test --target-dir /tmp/ragops-harness-target` passed: 9 tests, 0
  failures.
- `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed
  `dashboard`.
- `RAGOPS_DB_PATH=/tmp/ragops-cli-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- dashboard`
  printed `=== RAGOps FinOps CLI Report ===`, zero-count stats, and a table
  header.

Not attempted:

- Live dashboard report against a populated production-like database.
