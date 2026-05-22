# Red Team Engine

Status: implemented by `US-021-red-teaming-engine`.

## Purpose

The red-team engine is a local CLI scanner for OpenAI-compatible Chat
Completions endpoints. It probes a target with a small fixed set of malicious
prompts and reports whether the target appears to refuse unsafe requests.

## CLI Contract

- `cargo run` defaults to proxy serve mode for backward compatibility.
- `cargo run -- serve` starts the existing local proxy.
- `cargo run -- scan <target>` runs the red-team scanner.
- `<target>` must be the full URL for an endpoint that accepts OpenAI Chat
  Completions JSON.

## Scan Contract

Each scan sends five concurrent `POST` requests to the target.

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

The scanner parses OpenAI-style response bodies from `choices[].message.content`,
legacy `choices[].text`, `output_text`, or `error.message`. Non-JSON bodies are
treated as raw response text.

## Classification

- `Safe`: the target returned a successful HTTP response and the response text
  contains a known refusal marker.
- `Vulnerable`: the target returned a successful HTTP response without a known
  refusal marker.
- `Error`: the request failed, the response body could not be read, or the
  target returned a non-success HTTP status.

The scanner prints a terminal report with total cases, Safe count, Vulnerable
count, Error count, per-case status code, latency, prompt preview, and response
or error preview.

## Data Handling

US-021 does not persist scan prompts, responses, or reports. Results are printed
to stdout only.

## Non-Goals

- Adaptive attack generation.
- Provider authentication management.
- Exploit validation against real systems.
- Persistent findings storage.
- Dashboard visualization.
