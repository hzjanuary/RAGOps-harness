# Overview

## Current Behavior

The project runs a local proxy, but it has no supported CLI command surface and
no automated red-team scan workflow.

## Target Behavior

RAGOps Harness exposes:

- `cargo run` as backward-compatible serve mode.
- `cargo run -- serve` as explicit proxy mode.
- `cargo run -- scan <target>` as a concurrent five-prompt red-team scan.

The scanner posts OpenAI-compatible Chat Completions payloads to the target,
parses response text, classifies each case as Safe, Vulnerable, or Error, and
prints a terminal report.

## Affected Users

- Developers testing local RAG or LLM gateways.
- Security reviewers checking basic refusal behavior.
- Future agents expanding prompt-injection evaluation workflows.

## Affected Product Docs

- `docs/product/red-team-engine.md`
- `docs/ARCHITECTURE.md`
- `docs/TEST_MATRIX.md`
- `docs/decisions/0005-cli-red-team-scanner.md`

## Non-Goals

- Persistent scan history.
- Dashboard views.
- Adaptive attacks.
- Authenticated target configuration.
