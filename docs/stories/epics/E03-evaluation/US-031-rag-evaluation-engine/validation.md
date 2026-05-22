# Validation

## Proof Strategy

US-031 proof covers deterministic local behavior: prompt construction, binary
score parsing, compilation, unit tests, and CLI help. Live OpenAI evaluation
requires real credentials and may incur provider cost, so it is manual.

## Test Plan

| Layer | Cases |
| --- | --- |
| Unit | Required faithfulness prompt construction; `0` and `1` score parsing; non-binary score rejection. |
| Integration | Not covered; mock OpenAI integration is a follow-up before expanding judge parsing or retry behavior. |
| E2E | Not covered; no browser flow exists. |
| Platform | CLI help lists `eval` with `--dataset`. |
| Performance | Serial judge calls only; batch scheduling is out of scope. |
| Logs/Audit | No persistent audit or evaluation data is added; reports print to stdout only. |

## Fixtures

- Unit prompt fixture with context and answer strings.
- Unit score fixtures for `1`, `0`, `yes`, and `2`.

## Commands

```text
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
cargo run --target-dir /tmp/ragops-harness-target -- --help
cargo run --target-dir /tmp/ragops-harness-target -- eval --help
```

## Acceptance Evidence

- `cargo fmt --all` completed with no output.
- `cargo check --target-dir /tmp/ragops-harness-target` passed.
- `cargo test --target-dir /tmp/ragops-harness-target` passed: 9 tests, 0
  failures.
- `cargo run --target-dir /tmp/ragops-harness-target -- --help` listed
  `serve`, `scan`, and `eval`.
- `cargo run --target-dir /tmp/ragops-harness-target -- eval --help` listed
  `--dataset`.

Not attempted:

- Live evaluation against OpenAI.
