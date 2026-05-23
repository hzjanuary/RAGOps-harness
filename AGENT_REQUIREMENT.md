# TARGET STORY: US-042 Pure CLI Dashboard (Pivot from Web GUI)

## 1. ARCHITECTURE BOUNDARIES
- Remove Web GUI completely. Delete the `ui/` directory.
- Remove `tower-http` from `Cargo.toml`.
- Add `comfy-table` to `Cargo.toml` to render beautiful terminal tables (like docker or gh cli).
- CLI Command: `ragops-harness dashboard` (runs once, prints the report to stdout, and exits).

## 2. DOMAIN MODEL (CLI Dashboard)
- Connect to SQLite `harness.db`.
- Aggregate Statistics: Total Requests, Total Cost (USD), Average Latency (ms).
- Fetch Latest Logs: Retrieve the 5 most recent requests (ID, Model, Latency, Total Tokens, Cost).
- Output: Print a summary section, followed by a formatted `comfy-table` showing the recent logs.

## 3. EXECUTION PLAN
1. Update `Cargo.toml`: Remove `tower-http`. Add `comfy-table = "7.1"`.
2. Delete the `ui/` folder entirely.
3. Update `src/main.rs`: 
   - Add `Dashboard` back to the `Commands` enum in `clap`.
   - Remove `tower_http` and `ServeDir` fallback from the Axum proxy router in the `Serve` arm. 
   - Route the `Dashboard` command to `crate::dashboard::run_dashboard(&db_path).await`.
4. Rewrite `src/dashboard.rs`:
   - Remove all Axum-related imports (`IntoResponse`, `Json`, `State`).
   - Implement `pub async fn run_dashboard(db_path: &str)`.
   - Query the database directly.
   - Print the "RAGOps FinOps Report" using `println!` and `comfy-table::Table`.