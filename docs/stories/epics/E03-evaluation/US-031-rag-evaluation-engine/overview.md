# Overview

## Current Behavior

The project has a proxy and red-team scanner, but no workflow for evaluating RAG
answer quality against retrieved context.

## Target Behavior

RAGOps Harness exposes `cargo run -- eval --dataset <path>`. The command reads a
local JSON array of `{ question, context, answer }` records, asks OpenAI Chat
Completions to judge whether each answer is fully supported by its context, and
prints the total evaluated records plus the average Faithfulness score.

## Affected Users

- Developers validating RAG answer quality locally.
- Future agents expanding evaluation metrics and reporting.

## Affected Product Docs

- `docs/product/rag-evaluation-engine.md`
- `docs/ARCHITECTURE.md`
- `docs/TEST_MATRIX.md`
- `docs/decisions/0006-cli-rag-evaluation-engine.md`

## Non-Goals

- Stored evaluation reports.
- Dashboard reads.
- Additional metrics beyond Faithfulness.
- Provider abstraction.
