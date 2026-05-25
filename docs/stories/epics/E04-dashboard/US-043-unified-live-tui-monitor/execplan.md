# Exec Plan

## Goal

Implement US-043 Unified Live TUI Monitor so `serve` runs the proxy and live
dashboard concurrently in one process.

## Scope

In scope:

- Rewrite `main.rs` serve dispatch.
- Spawn `axum::serve` with `tokio::spawn`.
- Keep dashboard rendering on the foreground task.
- Change `dashboard::run_dashboard` to accept `&Database`.
- Render aggregate metrics and five latest request logs every two seconds.
- Keep `dashboard.rs` free of Axum types.
- Update product, story, decision, and test-matrix docs.

Out of scope:

- HTTP dashboard API.
- Keyboard-interactive TUI.
- Schema migrations.
- Provider integration smoke with real OpenAI credentials.

## Risk Classification

Risk flags:

- Existing behavior.
- Weak proof.

Hard gates:

- None.

Lane: normal.

## Work Phases

1. Discovery.
2. Design.
3. Implementation.
4. Verification.
5. Harness update.

## Stop Conditions

Pause for human confirmation if:

- The proxy route contract must change.
- Dashboard refresh must become interactive.
- Validation requirements need to be weakened.
