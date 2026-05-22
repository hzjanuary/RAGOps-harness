# RAG Evaluation Engine

Status: implemented by `US-031-rag-evaluation-engine`.

## Purpose

The RAG evaluation engine is a local CLI workflow for judging whether generated
answers are faithful to retrieved context. It reads a JSON dataset from disk,
sends each record to an OpenAI Chat Completions judge, and prints an aggregate
faithfulness score.

## CLI Contract

- `cargo run -- eval --dataset <path>` runs the evaluator.
- `<path>` must point to a local JSON file.
- The JSON file must be an array of records.

Dataset record shape:

```json
{
  "question": "What is the capital of France?",
  "context": "France's capital city is Paris.",
  "answer": "Paris."
}
```

## Evaluation Contract

Each record is sent to `https://api.openai.com/v1/chat/completions` with
`Authorization: Bearer $OPENAI_API_KEY`.

The judge prompt is:

```text
Evaluate if answer is fully supported by context. Return ONLY 1 for yes, 0 for no. Context: {context}. Answer: {answer}.
```

The evaluator parses the first returned assistant message as an integer score:

- `1`: answer is fully supported by the supplied context.
- `0`: answer is not fully supported by the supplied context.

Any other judge output is treated as an evaluation error.

## Report Contract

The evaluator prints:

- Total evaluated records.
- Average Faithfulness score.

Empty datasets produce `0` evaluated records and an average score of `0.000`.

## Configuration

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `OPENAI_API_KEY` | yes | none | OpenAI API key used for judge calls. |

The evaluator attempts to load `.env` before reading `OPENAI_API_KEY`.

## Data Handling

US-031 does not persist questions, contexts, answers, judge responses, or
scores. Evaluation results are printed to stdout only.

## Non-Goals

- Persistent evaluation history.
- Dashboard visualization.
- Multi-metric evaluation.
- Multi-provider judging.
- Retry policy or rate-limit scheduling.
