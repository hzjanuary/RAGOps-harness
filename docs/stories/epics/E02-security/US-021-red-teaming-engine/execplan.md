# Exec Plan

## Goal

Implement US-021 Red Teaming Engine as a CLI scan mode while preserving existing
proxy startup behavior.

## Scope

In scope:

- Add `clap`.
- Refactor `main.rs` to dispatch `serve` and `scan`.
- Move server startup behind `proxy::run()`.
- Add `red_team::run_scan(target)`.
- Add five concurrent malicious prompt requests.
- Parse response text and print Safe/Vulnerable/Error counts.

Out of scope:

- Stored findings.
- Dashboard reporting.
- Live target credentials.
- New HTTP routes.

## Risk Classification

Risk flags:

- Audit/security.
- External systems.
- Public contracts.
- Existing behavior.
- Weak proof.

Hard gates:

- Audit/security.

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

- Scanner output must persist prompts or responses.
- Target authentication is required.
- The proxy HTTP API must change.
- Validation requirements need to be weakened.
