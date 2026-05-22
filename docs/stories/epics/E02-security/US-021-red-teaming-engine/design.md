# Design

## Domain Model

- `ScanCase` identifies a fixed malicious prompt by index.
- `ScanResult` records per-case outcome, HTTP status, response preview, error
  preview, and latency.
- `Outcome` is one of Safe, Vulnerable, or Error.

## Application Flow

1. `main.rs` parses CLI arguments with `clap`.
2. No arguments default to `serve`.
3. `serve` calls `proxy::run()`.
4. `scan <target>` calls `red_team::run_scan(&target)`.
5. `run_scan` builds one `reqwest::Client`, spawns five concurrent tasks, and
   sends an OpenAI-format JSON payload for each malicious prompt.
6. Each task parses the target response text and checks for refusal markers.
7. Results are sorted by prompt index and printed as a terminal report.

## Interface Contract

Commands:

- `serve`
- `scan <target>`

Scan payload:

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

## Data Model

No tables or migrations are added. US-021 does not store prompt or response
content.

## UI / Platform Impact

The new surface is CLI only. The HTTP proxy routes are unchanged.

## Observability

Scan output is stdout terminal reporting. Proxy tracing remains unchanged.

## Alternatives Considered

1. Separate scanner binary. Rejected to keep the first CLI surface small.
2. HTTP-triggered scanner route. Rejected to avoid widening proxy API scope.
3. SQLite-backed findings. Deferred pending retention and privacy rules.
