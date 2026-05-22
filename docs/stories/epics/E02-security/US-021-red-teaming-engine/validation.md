# Validation

## Proof Strategy

US-021 proof covers deterministic local behavior: CLI parsing, scanner response
text parsing, refusal detection, compilation, and the terminal report path for
an unreachable local target. A live scan requires a running OpenAI-compatible
target and is not required for this story.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | Refusal marker detection; non-refusal vulnerable signal; OpenAI chat content extraction. |
| Integration | Not covered; mock-target integration is a follow-up before expanding classifications. |
| E2E | Not covered; no browser flow exists. |
| Platform | CLI help lists `serve` and `scan`; scan command prints a report for a local unreachable target. |
| Performance | Five scan requests are spawned concurrently; no load target exists. |
| Logs/Audit | No persistent audit data is added; scan reports print to stdout only. |

## Fixtures

- OpenAI-style JSON response fixture with `choices[].message.content`.
- Local unreachable endpoint `http://127.0.0.1:9/v1/chat/completions` for report
  smoke proof without external network or credentials.

## Commands

```text
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
cargo run --target-dir /tmp/ragops-harness-target -- --help
cargo run --target-dir /tmp/ragops-harness-target -- scan http://127.0.0.1:9/v1/chat/completions
```

## Acceptance Evidence

- `cargo fmt --all` completed with no output.
- `cargo check --target-dir /tmp/ragops-harness-target` passed.
- `cargo test --target-dir /tmp/ragops-harness-target` passed: 6 tests, 0
  failures.
- `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed `serve`
  and `scan`.
- `cargo run --target-dir /tmp/ragops-harness-target -- scan http://127.0.0.1:9/v1/chat/completions`
  printed a five-case report with 0 Safe, 0 Vulnerable, and 5 expected
  connection Errors.

Not attempted:

- Live scan against a working LLM target.
