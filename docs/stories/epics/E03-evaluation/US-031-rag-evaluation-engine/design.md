# Design

## Domain Model

- `EvalRecord` represents one dataset row with `question`, `context`, and
  `answer`.
- The Faithfulness score is a binary integer returned by the judge: `1` for
  supported, `0` for unsupported.

## Application Flow

1. `main.rs` parses CLI arguments with `clap`.
2. `eval --dataset <path>` calls `eval::run_eval(&dataset)`.
3. `run_eval` reads the local JSON dataset and parses it into `Vec<EvalRecord>`.
4. The evaluator loads `OPENAI_API_KEY` from `.env` or the process
   environment.
5. For each record, the evaluator sends a Chat Completions request to OpenAI
   with the required faithfulness prompt.
6. The evaluator parses the first assistant message as `0` or `1`.
7. Scores are averaged and printed to stdout.

## Interface Contract

Command:

- `eval --dataset <path>`

Dataset:

```json
[
  {
    "question": "Question text",
    "context": "Retrieved context text",
    "answer": "Generated answer text"
  }
]
```

## Data Model

No tables or migrations are added. US-031 does not persist evaluation inputs,
judge responses, or scores.

## UI / Platform Impact

The new surface is CLI only. Existing proxy routes and red-team scan behavior
are unchanged.

## Observability

Evaluation output is stdout terminal reporting. No new tracing or audit records
are introduced.

## Alternatives Considered

1. Store per-record scores in SQLite. Deferred until retention and dashboard
   contracts exist.
2. Reuse the local proxy for judge calls. Rejected for this slice because the
   requirement targets direct OpenAI Chat Completions calls from the CLI.
3. Add multiple quality metrics. Deferred to keep US-031 scoped to
   Faithfulness.
