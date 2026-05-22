# Validation

## Proof Strategy

US-041 proof covers Rust compilation, existing unit tests, CLI help, and a
runtime smoke test for the Axum-hosted dashboard API and static frontend.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | Existing database insertion and usage parsing tests remain active. Dedicated stats fixture test is a follow-up. |
| Integration | `GET /api/stats` returns aggregate JSON from the server state database. |
| E2E | Not covered; no browser automation exists yet. |
| Platform | `GET /` serves `ui/index.html`; `GET /health` remains healthy. |
| Performance | Dashboard reads aggregate queries and five recent rows only. |
| Logs/Audit | Dashboard reads metadata only and does not persist new records. |

## Commands

```text
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
cargo run --target-dir /tmp/ragops-harness-target -- --help
RAGOPS_DB_PATH=/tmp/ragops-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- serve
curl -sS http://127.0.0.1:8000/api/stats
curl -sS http://127.0.0.1:8000/
curl -sS http://127.0.0.1:8000/health
```

## Acceptance Evidence

- `cargo fmt --all` completed with no output.
- `cargo check --target-dir /tmp/ragops-harness-target` passed.
- `cargo test --target-dir /tmp/ragops-harness-target` passed: 9 tests, 0
  failures.
- `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed
  `serve`, `scan`, and `eval`.
- `RAGOPS_DB_PATH=/tmp/ragops-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- serve`
  started the Axum server on `0.0.0.0:8000`.
- `curl -sS http://127.0.0.1:8000/api/stats` returned dashboard JSON with
  zero counts for an empty smoke-test database.
- `curl -sS http://127.0.0.1:8000/` returned `ui/index.html`.
- `curl -sS http://127.0.0.1:8000/health` returned healthy JSON.

Not attempted:

- Browser rendering smoke test.
