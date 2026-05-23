# 0007 Web Dashboard Pivot

Date: 2026-05-22

## Status

Superseded by `0008-pure-cli-dashboard`

## Context

US-041 originally targeted a Tauri desktop dashboard. The Linux environment for
this project no longer provides the WebKitGTK 4.0-era pkg-config packages
required by Tauri 1.x. The available baseline provides WebKitGTK 4.1-era
packages instead.

The hackathon goal is a working local dashboard, so the desktop shell is being
dropped in favor of serving the static dashboard from the existing Axum server.

## Decision

Serve the dashboard natively through Axum.

The dashboard:

- Uses `tower-http` `ServeDir` to serve static files from `ui/`.
- Adds `GET /api/stats` to the existing proxy server.
- Reads FinOps metadata from the existing proxy `AppState` database connection.
- Keeps `GET /health` and `POST /v1/chat/completions` unchanged.
- Removes Tauri dependencies, `build.rs`, and `tauri.conf.json`.

## Alternatives Considered

1. Keep Tauri and install compatibility packages. Rejected because the target
   environment removed the required WebKitGTK 4.0 packages.
2. Move to Tauri 2. Deferred because the user selected an Axum web pivot for
   hackathon speed.
3. Add a separate frontend dev server. Rejected because static files served by
   Axum are enough for this slice.

## Consequences

Positive:

- The dashboard compiles on the current Linux environment.
- No native desktop shell dependencies are needed.
- Operators use the existing server URL: `http://localhost:8000/`.

Tradeoffs:

- The dashboard is browser-based rather than a desktop window.
- CDN assets require network access when the dashboard loads.
- Static file serving now shares the proxy server process.

## Follow-Up

- Add a deterministic database fixture test for `/api/stats`.
- Decide whether dashboard assets should be vendored instead of loaded from
  CDNs.
- Add browser-level visual smoke checks if the dashboard grows beyond the first
  FinOps view.
