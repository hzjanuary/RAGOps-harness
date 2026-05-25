# RAGOps Harness

RAGOps Harness is a local-first Rust tool for inspecting RAG and LLM application
behavior. It currently provides a local OpenAI Chat Completions proxy, FinOps
usage logging, a terminal dashboard, a basic red-team scanner, and a simple RAG
faithfulness evaluator.

The system is intentionally small and CLI-first. Product behavior is tracked in
`docs/product/`, story evidence in `docs/stories/`, and validation status in
`docs/TEST_MATRIX.md`.

## Current Status

Current milestone: `US-043 Unified Live TUI Monitor`.

Implemented:

- Local Axum proxy on `0.0.0.0:8000`.
- `GET /health`.
- `POST /v1/chat/completions` forwarding to OpenAI Chat Completions.
- SQLite FinOps logging for token usage, latency, model, upstream id, and
  estimated cost.
- Live CLI dashboard in `serve` mode that refreshes aggregate FinOps stats and
  the five latest logs every two seconds.
- One-shot CLI dashboard command for snapshot reports.
- CLI red-team scanner for OpenAI-compatible Chat Completions endpoints.
- CLI RAG faithfulness evaluator using OpenAI as a judge.

Retired:

- The Axum-served web dashboard from `US-041`.
- `GET /api/stats`.
- Static `ui/` assets and direct `tower-http` usage.

## Requirements

- Rust toolchain.
- OpenAI API key for proxy forwarding and RAG evaluation.
- SQLite is provided through the bundled `rusqlite` dependency.

## Configuration

Create a local `.env` file when using OpenAI-backed commands:

```bash
OPENAI_API_KEY=sk-your-key-here
RAGOPS_DB_PATH=ragops_harness.sqlite3
```

Environment variables:

| Variable | Required | Default | Used by |
| --- | --- | --- | --- |
| `OPENAI_API_KEY` | Yes for proxy requests and eval runs | none | `serve`, `eval` |
| `RAGOPS_DB_PATH` | No | `ragops_harness.sqlite3` | `serve`, `dashboard` |

Exported environment variables take precedence over `.env`.

## Commands

Show the command surface:

```bash
cargo run -- --help
```

Run the proxy with the live CLI dashboard:

```bash
cargo run
# or
cargo run -- serve
```

Print the CLI dashboard:

```bash
cargo run -- dashboard
```

Run a red-team scan:

```bash
cargo run -- scan --target http://127.0.0.1:8000/v1/chat/completions
```

Run RAG faithfulness evaluation:

```bash
cargo run -- eval --dataset ./dataset.json
```

## Proxy

The proxy accepts OpenAI Chat Completions JSON at:

```text
POST http://127.0.0.1:8000/v1/chat/completions
```

Example:

```bash
curl -sS http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [
      { "role": "user", "content": "Say hello in one short sentence." }
    ]
  }'
```

Health check:

```bash
curl -sS http://127.0.0.1:8000/health
```

Expected response:

```json
{"status":"ok","service":"ragops-harness"}
```

The proxy loads `.env`, reads `OPENAI_API_KEY`, forwards non-streaming requests
to OpenAI, parses the `usage` block, estimates cost, and writes metadata to
SQLite. Streaming requests are rejected because usage logging requires a
complete JSON response.

## CLI Dashboard

The `serve` command starts the Axum proxy in a background Tokio task and keeps
the foreground terminal on a live dashboard loop:

```bash
cargo run -- serve
```

The dashboard enters the terminal alternate screen, hides the cursor, and
redraws in place every two seconds. It shows total request count, total
estimated cost, average latency, flagged entries, and the five latest logged
proxy requests. Ctrl-C exits the live view and restores the terminal.

The dashboard can also print a one-shot terminal report:

```text
+======================================================================+
|                    RAGOps Live FinOps Monitor                       |
|              Local OpenAI Proxy + SQLite Usage Telemetry            |
+======================================================================+
```

It then renders summary cards and a `comfy-table` table with:

- Request ID.
- Timestamp.
- Model.
- Tokens.
- Latency in milliseconds.
- Cost in USD.
- Status code.

The dashboard reads the database configured by `RAGOPS_DB_PATH`.

## Red-Team Scanner

The scanner sends five concurrent malicious prompts to an OpenAI-compatible
Chat Completions endpoint.

Payload shape:

```json
{
  "model": "test",
  "messages": [
    {
      "role": "user",
      "content": "malicious prompt"
    }
  ]
}
```

Each result is classified as:

- `Safe`: successful response includes a known refusal marker.
- `Vulnerable`: successful response does not include a known refusal marker.
- `Error`: request failed, body read failed, or HTTP status was non-success.

The scanner prints counts plus per-case status, latency, prompt preview, and
response or error preview. Results are not persisted.

## RAG Evaluation

The evaluator reads a local JSON dataset:

```json
[
  {
    "question": "What is the capital of France?",
    "context": "France's capital city is Paris.",
    "answer": "Paris."
  }
]
```

For each row, it asks OpenAI to return `1` when the answer is fully supported by
the context and `0` otherwise. The command prints total evaluated records and
average Faithfulness score. Dataset rows and judge responses are not persisted.

## Data Storage

FinOps records are stored in SQLite table `llm_request_logs`.

Stored fields include:

- Local request id.
- Timestamp.
- Provider and endpoint.
- Model.
- Upstream response id.
- Prompt, completion, total, and cached prompt tokens.
- Estimated USD cost.
- Latency in milliseconds.
- Upstream success status code.
- Whether pricing was known.

Prompt and completion content are not stored by the proxy. Red-team prompts,
scan responses, evaluation datasets, judge responses, and dashboard reports are
printed or processed in memory only.

## Validation

Run local checks:

```bash
cargo fmt --all
cargo check --target-dir /tmp/ragops-harness-target
cargo test --target-dir /tmp/ragops-harness-target
```

Useful smoke checks:

```bash
cargo run --target-dir /tmp/ragops-harness-target -- --help
RAGOPS_DB_PATH=/tmp/ragops-cli-dashboard-smoke.sqlite \
  cargo run --target-dir /tmp/ragops-harness-target -- dashboard
```

Live OpenAI proxy and evaluation proof requires real credentials and may incur
provider cost.

## Project Layout

```text
src/
  main.rs       CLI dispatch and serve-mode proxy/dashboard concurrency
  proxy.rs      Axum proxy, OpenAI forwarding, usage parsing, cost estimation
  db.rs         SQLite setup, schema, and FinOps usage persistence
  dashboard.rs  Terminal FinOps dashboard rendering using comfy-table
  red_team.rs   Concurrent malicious-prompt scanner
  eval.rs       Local JSON RAG faithfulness evaluation

docs/
  HARNESS.md        Human-agent operating model
  FEATURE_INTAKE.md Task classification and risk lanes
  ARCHITECTURE.md   Architecture rules and selected stack
  TEST_MATRIX.md    Behavior-to-proof status
  product/          Current product contracts
  stories/          Story packets and validation evidence
  decisions/        Architecture and product decision records
```

## Architecture Notes

Runtime stack:

- Rust.
- Tokio.
- Axum.
- Reqwest.
- Rusqlite with bundled SQLite.
- Clap.
- Comfy Table.
- Crossterm.

Primary flow:

```text
RAG app
  -> local RAGOps proxy
      -> OpenAI Chat Completions
      -> SQLite FinOps log
      -> live CLI dashboard reads SQLite
```

See `docs/ARCHITECTURE.md` and `docs/decisions/` for accepted architectural
choices and pivots.

## Current Limitations

- OpenAI Chat Completions only.
- Streaming proxy requests are not supported.
- No incoming proxy authentication.
- No provider retry policy beyond returning structured errors.
- Pricing constants are maintained in code.
- Dashboard has no keyboard-interactive controls.
- RAG evaluation requires live OpenAI credentials and has no offline judge mode.

## Security Notes

- Keep `.env` local.
- Do not paste provider keys into prompts, docs, issues, or shared logs.
- Rotate any key that was committed or shared.
- The proxy stores operational metadata, not prompt or completion content.

## License

No project license has been selected yet.
