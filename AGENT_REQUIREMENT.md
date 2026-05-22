# TARGET STORY: US-041 Web GUI Dashboard (Pivot from Tauri)

## 1. ARCHITECTURE BOUNDARIES
- Remove `tauri` and `tauri-build` completely.
- Add `tower-http` to serve static files.
- Run UI via default browser at `http://localhost:8000/`.
- Backend: Axum handles both `/v1/chat/completions` (Proxy) and `/api/stats` (FinOps DB Query).
- Frontend: `ui/index.html` uses standard `fetch('/api/stats')` instead of Tauri API.

## 2. EXECUTION PLAN
1. Update `Cargo.toml`: Remove `tauri`, `tauri-build`, `build-dependencies`. Add `tower-http = { version = "0.5", features = ["fs", "cors"] }`.
2. Delete `build.rs` and `tauri.conf.json`.
3. Update `src/dashboard.rs`: Remove Tauri imports. Implement Axum handler `pub async fn api_stats(State(state): State<AppState>) -> impl IntoResponse`. Aggregate total_requests, total_cost_usd, avg_latency_ms, and fetch last 5 logs. Return JSON.
4. Update `src/main.rs`: In `Serve` command router, add `.route("/api/stats", get(dashboard::api_stats))` and `.fallback_service(tower_http::services::ServeDir::new("ui"))`.
5. Update `ui/index.html`: Replace `window.__TAURI__.invoke(...)` with `fetch('/api/stats').then(res => res.json())`.