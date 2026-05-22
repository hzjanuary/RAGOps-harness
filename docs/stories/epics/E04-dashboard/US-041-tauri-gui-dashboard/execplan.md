# Exec Plan

## Goal

Implement US-041 Web GUI Dashboard as an Axum-served local UI for FinOps
metrics.

## Scope

In scope:

- Remove Tauri dependencies and build files.
- Add `tower-http` static file serving.
- Add `GET /api/stats`.
- Serve `ui/index.html` from the existing Axum process.
- Aggregate total requests, total cost, average latency, and five latest logs.
- Replace Tauri frontend invocation with `fetch("/api/stats")`.

Out of scope:

- Native desktop packaging.
- New database schema.
- Authentication.
- Dashboard settings.

## Risk Classification

Risk flags:

- Public contracts.
- Existing behavior.
- Weak proof.

Hard gates:

- None.

Lane: normal with stronger validation.

## Work Phases

1. Discovery.
2. Design.
3. Validation planning.
4. Implementation.
5. Verification.
6. Harness update.

## Stop Conditions

Pause for human confirmation if:

- Dashboard data must include prompt or response content.
- The dashboard must write to the database.
- A separate frontend runtime must be introduced.
- Validation requirements need to be weakened.
