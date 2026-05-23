# 0008 Pure CLI Dashboard

Date: 2026-05-23

## Status

Accepted

## Context

US-041 provided a browser dashboard by adding `tower-http`, `GET /api/stats`,
and static files under `ui/`. The hackathon direction changed to a terminal-only
experience similar to CLI tools that print dense operational tables.

## Decision

Replace the web dashboard with a pure CLI dashboard command.

The dashboard:

- Runs as `ragops-harness dashboard`.
- Reads the configured SQLite database directly through `Database`.
- Prints aggregate FinOps stats to stdout.
- Uses `comfy-table` for the five most recent request logs.
- Removes `tower-http`, `/api/stats`, and tracked `ui/` assets from the product
  surface.

## Alternatives Considered

1. Keep both web and CLI dashboards. Rejected because the pivot explicitly
   removes the web GUI surface.
2. Keep `/api/stats` for future use. Rejected because there is no active client
   for that HTTP contract after the pivot.
3. Store rendered reports. Rejected because US-042 is a one-shot read-only CLI
   report.

## Consequences

Positive:

- The dashboard runs entirely in the terminal.
- No browser, static asset server, CDN, or frontend files are required.
- The proxy route surface returns to health and Chat Completions only.

Tradeoffs:

- The report is not interactive or live-updating.
- There is no dashboard HTTP API for external consumers.
- Historical US-041 web proof is retired rather than extended.

## Follow-Up

- Add filters or watch mode only after a CLI reporting contract is selected.
