# 0005 CLI Red Team Scanner

Date: 2026-05-22

## Status

Accepted

## Context

US-021 introduces the first command-line workflow and the first security-testing
surface. The existing binary already runs the Axum proxy when launched without
arguments, so the new interface must preserve that behavior while adding an
explicit scanner command.

## Decision

Use `clap` in the root Rust binary with two subcommands:

- `serve` starts the existing proxy.
- `scan <target>` runs a bounded red-team scanner against an OpenAI-compatible
  Chat Completions endpoint.

No separate worker process, persistent report table, or dashboard surface is
introduced in US-021.

## Alternatives Considered

1. Create a separate scanner binary. Rejected for now because the project has
   one small Rust crate and shared dependency needs are minimal.
2. Add scanner HTTP routes to the proxy. Rejected because US-021 is a local CLI
   workflow and should not widen the public HTTP API.
3. Store findings in SQLite. Deferred until there is a product contract for
   report retention, privacy, and dashboard reads.

## Consequences

Positive:

- `cargo run` remains backward compatible.
- Operators can run `serve` and `scan` from one binary.
- Scanner output is immediate and does not retain malicious prompts or model
  responses on disk.

Tradeoffs:

- Refusal detection is heuristic and can produce false positives or false
  negatives.
- Live scan proof depends on a running target endpoint.
- Scan reports are not queryable after the process exits.

## Follow-Up

- Add deterministic mock-target integration tests before expanding scanner
  classifications.
- Define a retention policy before storing scan reports.
