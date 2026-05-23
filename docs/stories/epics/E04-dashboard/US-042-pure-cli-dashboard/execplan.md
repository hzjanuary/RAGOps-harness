# Exec Plan

## Goal

Replace the Axum-served web dashboard with a pure CLI FinOps dashboard report.

## Scope

In scope:

- Remove `tower-http`.
- Add `comfy-table`.
- Add `dashboard` CLI command.
- Remove dashboard routes and static file serving from `serve`.
- Rewrite `src/dashboard.rs` as a SQLite-backed terminal report.
- Remove tracked `ui/` assets.

Out of scope:

- Watch mode.
- Report export.
- Stored dashboard preferences.
- Browser or desktop UI.

## Risk Classification

Risk flags:

- Public contracts.
- Cross-platform.
- Existing behavior.
- Weak proof.

Hard gates:

- None.

Lane: high-risk.

## Work Phases

1. Discovery.
2. Design.
3. Validation planning.
4. Implementation.
5. Verification.
6. Harness update.

## Stop Conditions

Pause for human confirmation if:

- The web dashboard must coexist with CLI output.
- The dashboard needs authentication or persistent report storage.
- Validation requirements need to be weakened.
