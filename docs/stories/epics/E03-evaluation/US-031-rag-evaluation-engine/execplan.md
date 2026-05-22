# Exec Plan

## Goal

Implement US-031 RAG Evaluation Engine as a CLI evaluation mode that calculates
an average Faithfulness score for a local JSON dataset.

## Scope

In scope:

- Add an `eval --dataset <path>` CLI subcommand.
- Add `src/eval.rs`.
- Parse local JSON dataset records with `question`, `context`, and `answer`.
- Call OpenAI Chat Completions using `OPENAI_API_KEY`.
- Parse binary judge scores.
- Print total evaluated records and average Faithfulness score.

Out of scope:

- Stored reports.
- Dashboard reporting.
- Provider abstraction.
- Retry and rate-limit orchestration.

## Risk Classification

Risk flags:

- External systems.
- Public contracts.
- Existing behavior.
- Weak proof.

Hard gates:

- External provider behavior.

Lane: high-risk.

## Work Phases

1. Discovery.
2. Design.
3. Validation planning.
4. Implementation.
5. Verification.
6. Harness update.

## Stop Conditions

Pause for human confirmation if:

- Evaluation results must be persisted.
- The evaluator must use a provider other than OpenAI Chat Completions.
- The proxy HTTP API must change.
- Validation requirements need to be weakened.
