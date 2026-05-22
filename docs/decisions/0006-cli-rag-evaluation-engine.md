# 0006 CLI RAG Evaluation Engine

Date: 2026-05-22

## Status

Accepted

## Context

US-031 introduces the first answer-quality evaluation workflow. The selected
story requires a local JSON dataset, OpenAI Chat Completions as an LLM judge,
and a terminal Faithfulness report.

The existing binary already owns local CLI workflows for `serve` and `scan`, so
the evaluation surface should fit that command model without widening the proxy
HTTP API or adding persistence before retention rules exist.

## Decision

Add `eval --dataset <path>` to the root Rust binary.

The evaluator:

- Reads a local JSON array of `{ question, context, answer }` records.
- Calls OpenAI Chat Completions directly with `reqwest` and `OPENAI_API_KEY`.
- Uses a binary Faithfulness prompt that asks the judge to return only `1` or
  `0`.
- Prints total evaluated records and average Faithfulness score.
- Does not persist dataset contents, judge responses, or scores.

## Alternatives Considered

1. Add a proxy route for evaluation. Rejected because US-031 is a local CLI
   workflow and should not expand the HTTP API.
2. Store evaluation results in SQLite. Deferred until report retention,
   privacy, and dashboard read contracts exist.
3. Add a provider abstraction. Deferred because the selected story targets
   OpenAI Chat Completions only.

## Consequences

Positive:

- RAG answer faithfulness can be checked from the existing binary.
- No new storage schema or retention policy is required.
- The evaluator follows the existing OpenAI and `reqwest` dependency choices.

Tradeoffs:

- Live evaluation depends on external credentials and provider availability.
- Scores depend on judge behavior and are not deterministic offline.
- Large datasets run serially in this slice.

## Follow-Up

- Add a mock OpenAI integration test before expanding evaluator behavior.
- Define persistence and privacy rules before storing evaluation reports.
- Add batching, retries, or rate-limit handling only after dataset-size
  expectations are known.
