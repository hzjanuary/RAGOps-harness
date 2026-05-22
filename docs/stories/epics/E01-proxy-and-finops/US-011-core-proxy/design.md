# Design

## Domain Model

- `UsageLogEntry` captures the durable FinOps record: request id, provider,
  endpoint, model, upstream id, token counts, cached token count, estimated cost,
  latency, status code, and whether pricing was known.
- `OpenAiUsage` parses the OpenAI `usage` object and supports optional
  `prompt_tokens_details.cached_tokens`.
- `CostEstimate` records the calculated USD cost and whether the model matched a
  local pricing rule.
- `ApiError` is the HTTP-facing error shape and always serializes as JSON.

## Application Flow

1. Startup initializes tracing, loads `.env` if present, and then reads
   `RAGOPS_DB_PATH` and `OPENAI_API_KEY`.
2. Axum receives `POST /v1/chat/completions`.
3. The handler generates a local UUID request id and parses the body into a
   strict request shape for `model` and `stream`.
4. Missing or malformed request fields return a JSON `400`.
5. Missing `OPENAI_API_KEY` returns a JSON `500` configuration error.
6. Non-streaming JSON payloads are forwarded to
   `https://api.openai.com/v1/chat/completions` using `reqwest`.
7. Upstream non-success responses are wrapped in the proxy JSON error envelope
   while preserving the upstream status code.
8. Successful upstream JSON is parsed for `usage`, cost is estimated, and the
   SQLite log write is attempted.
9. If logging succeeds, the original upstream JSON is returned to the client.
   If logging fails, the client receives a JSON `500` explaining that the
   upstream call succeeded but local FinOps logging failed.

## Interface Contract

Routes:

- `GET /health`
- `POST /v1/chat/completions`

Successful chat response:

- Status: `200`.
- Body: OpenAI Chat Completions JSON.

Error response:

```json
{
  "error": {
    "code": "invalid_chat_completion_request",
    "message": "Request body must be valid Chat Completions JSON: ...",
    "request_id": "uuid"
  }
}
```

OpenAI error responses include `upstream_status` and `upstream_body` when
available.

## Data Model

SQLite table: `llm_request_logs`

Indexes:

- `idx_llm_request_logs_created_at`
- `idx_llm_request_logs_model_created_at`

The database connection is wrapped in `Arc<tokio::sync::Mutex<rusqlite::Connection>>`
to keep the single local SQLite connection safe across async handlers. The
connection enables a 5-second busy timeout, foreign keys, WAL journal mode, and
normal synchronous mode.

No migration framework is introduced in US-011. The schema is created with
`CREATE TABLE IF NOT EXISTS` during startup.

## UI / Platform Impact

No GUI, CLI, or platform shell is introduced. The runtime surface is a local
HTTP server only.

## Observability

Tracing is initialized in `main.rs`. The proxy emits structured tracing fields
for successful requests, upstream errors, unknown pricing matches, missing API
key at boot, and DB logging failures.

The SQLite record is product audit/FinOps data. Tracing logs are operational
logs and do not replace the SQLite record.

## Alternatives Considered

1. Support streaming immediately. Rejected for US-011 because streaming requires
   SSE handling and delayed usage aggregation, which would widen the story.
2. Store prompt and completion payloads. Rejected for US-011 to keep local
   privacy risk lower while still proving cost logging.
3. Use a connection pool. Deferred because US-011 explicitly requires
   `Arc<Mutex<rusqlite::Connection>>` and a single local SQLite file.
