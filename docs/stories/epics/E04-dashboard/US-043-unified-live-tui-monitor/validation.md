# Validation

## Proof Strategy

US-043 proof must show the crate compiles, existing tests still pass, CLI help
still exposes the commands, one-shot dashboard rendering still works, and the
serve command can start the proxy while the foreground dashboard loop renders.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | Existing FinOps, red-team, and eval tests remain green. |
| Integration | Serve mode shares one `Database` between proxy state and dashboard reads. |
| E2E | Not covered; no browser flow exists. |
| Platform | CLI help lists `serve` and `dashboard`; serve prints the live dashboard while the proxy responds to `GET /health`. |
| Performance | Dashboard refresh is bounded to aggregate queries plus five latest rows every two seconds. |
| Logs/Audit | Dashboard is read-only; render failures are operational logs only. |

## Fixtures

- Empty smoke database at `/tmp/ragops-cli-dashboard-smoke.sqlite`.
- Empty live monitor smoke database at `/tmp/ragops-live-dashboard-smoke.sqlite`.

## Commands

```text
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
cargo run --target-dir /tmp/ragops-harness-target -- --help
RAGOPS_DB_PATH=/tmp/ragops-cli-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- dashboard
RAGOPS_DB_PATH=/tmp/ragops-live-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- serve
curl -sS http://127.0.0.1:8000/health
```

## Acceptance Evidence

- `cargo fmt --all` passed.
- Initial sandboxed `cargo check --target-dir /tmp/ragops-harness-target`
  failed because Cargo needed network access to resolve the newly added
  `crossterm` dependency. The escalated retry passed and updated `Cargo.lock`
  with `crossterm v0.27.0`.
- `cargo test --target-dir /tmp/ragops-harness-target` passed: 9 tests, 0
  failures.
- `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed
  `serve` and `dashboard`.
- `RAGOPS_DB_PATH=/tmp/ragops-cli-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- dashboard`
  printed the live monitor header, zero-count summary cards, and latest
  requests table without emitting an ANSI clear-screen sequence from
  `dashboard.rs`.
- `RAGOPS_DB_PATH=/tmp/ragops-live-dashboard-smoke.sqlite cargo run --target-dir /tmp/ragops-harness-target -- serve`
  started the proxy, logged `RAGOps Harness proxy listening`, entered the
  alternate screen, hid the cursor, and rendered the live dashboard repeatedly.
  The first sandboxed attempt failed to bind `0.0.0.0:8000` with
  `Operation not permitted`; the escalated local smoke passed.
- `curl -sS http://127.0.0.1:8000/health` returned
  `{"status":"ok","service":"ragops-harness"}` during the serve smoke.
- Ctrl-C exited the serve smoke with code 0 and emitted cursor restore plus
  leave-alternate-screen terminal control sequences.

Not attempted:

- Live OpenAI proxy request, because it requires credentials and may incur
  provider cost.
