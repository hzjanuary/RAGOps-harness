use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

type EvalResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const OPENAI_CHAT_COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";
const EVAL_MODEL: &str = "gpt-4o-mini";

#[derive(Debug, Deserialize)]
struct EvalRecord {
    question: String,
    context: String,
    answer: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

#[derive(Debug)]
struct EvalError {
    message: String,
}

impl EvalError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for EvalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for EvalError {}

pub async fn run_eval(dataset_path: &str) -> EvalResult<()> {
    let records = read_dataset(dataset_path).await?;

    if records.is_empty() {
        print_report(0, 0.0);
        return Ok(());
    }

    let api_key = openai_api_key()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("ragops-harness-eval/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut total_score = 0_u64;
    for (index, record) in records.iter().enumerate() {
        let score = evaluate_record(&client, &api_key, index + 1, record).await?;
        total_score += u64::from(score);
    }

    let average = total_score as f64 / records.len() as f64;
    print_report(records.len(), average);

    Ok(())
}

async fn read_dataset(dataset_path: &str) -> EvalResult<Vec<EvalRecord>> {
    let dataset = tokio::fs::read_to_string(dataset_path)
        .await
        .map_err(|error| {
            EvalError::new(format!(
                "failed to read evaluation dataset `{dataset_path}`: {error}"
            ))
        })?;

    serde_json::from_str(&dataset).map_err(|error| {
        EvalError::new(format!(
            "failed to parse evaluation dataset `{dataset_path}` as a JSON array: {error}"
        ))
        .into()
    })
}

fn openai_api_key() -> EvalResult<String> {
    let _ = dotenvy::dotenv();

    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| EvalError::new("OPENAI_API_KEY is required to run evaluation"))?;

    if api_key.trim().is_empty() {
        return Err(EvalError::new("OPENAI_API_KEY must not be empty").into());
    }

    Ok(api_key)
}

async fn evaluate_record(
    client: &Client,
    api_key: &str,
    record_index: usize,
    record: &EvalRecord,
) -> EvalResult<u8> {
    let prompt = build_prompt(&record.context, &record.answer);
    let payload = json!({
        "model": EVAL_MODEL,
        "temperature": 0,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = client
        .post(OPENAI_CHAT_COMPLETIONS_URL)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|error| {
            EvalError::new(format!(
                "OpenAI evaluation request failed for record {record_index}: {error}"
            ))
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        EvalError::new(format!(
            "failed to read OpenAI evaluation response for record {record_index}: {error}"
        ))
    })?;

    if !status.is_success() {
        return Err(EvalError::new(format!(
            "OpenAI evaluation failed for record {record_index} with status {status}: {}",
            truncate(&body, 500)
        ))
        .into());
    }

    let response: ChatCompletionResponse = serde_json::from_str(&body).map_err(|error| {
        EvalError::new(format!(
            "failed to parse OpenAI evaluation response for record {record_index}: {error}"
        ))
    })?;
    let content = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| {
            EvalError::new(format!(
                "OpenAI evaluation response for record {record_index} did not include message content"
            ))
        })?;

    parse_score(content).map_err(|error| {
        EvalError::new(format!(
            "OpenAI evaluation response for record {record_index} ({}) was not a binary score: {error}",
            truncate(&record.question, 100)
        ))
        .into()
    })
}

fn build_prompt(context: &str, answer: &str) -> String {
    format!(
        "Evaluate if answer is fully supported by context. Return ONLY 1 for yes, 0 for no. Context: {context}. Answer: {answer}."
    )
}

fn parse_score(value: &str) -> Result<u8, EvalError> {
    let score: u8 = value
        .trim()
        .parse()
        .map_err(|_| EvalError::new(format!("expected 0 or 1, got `{value}`")))?;

    match score {
        0 | 1 => Ok(score),
        _ => Err(EvalError::new(format!("expected 0 or 1, got `{value}`"))),
    }
}

fn print_report(total_records: usize, average_faithfulness: f64) {
    println!("Total evaluated records: {total_records}");
    println!("Average Faithfulness score: {average_faithfulness:.3}");
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let mut truncated: String = value.chars().take(max_chars.saturating_sub(3)).collect();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_required_faithfulness_prompt() {
        assert_eq!(
            build_prompt("The context", "The answer"),
            "Evaluate if answer is fully supported by context. Return ONLY 1 for yes, 0 for no. Context: The context. Answer: The answer."
        );
    }

    #[test]
    fn parses_binary_score() {
        assert_eq!(parse_score("1").expect("score parses"), 1);
        assert_eq!(parse_score(" 0\n").expect("score parses"), 0);
    }

    #[test]
    fn rejects_non_binary_score() {
        assert!(parse_score("yes").is_err());
        assert!(parse_score("2").is_err());
    }
}
