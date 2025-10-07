use std::io::{BufRead as _, BufReader};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::provider::Provider as ProviderTrait;

#[derive(Debug, Serialize, Deserialize)]
struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelsResponse {
    data: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    content: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: String,
    index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    id: String,
    object: String,
    created: u64,
    model: String,
}

// Streaming response structures
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
    id: String,
    object: String,
    created: u64,
    model: String,
}

pub struct OpenAIProvider {
    api_key: String,
}

impl OpenAIProvider {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable is not set"))?;
        Ok(Self { api_key })
    }
}

impl ProviderTrait for OpenAIProvider {
    fn list_models(&self) -> Result<()> {
        let models = ureq::get("https://api.openai.com/v1/models")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .call()?
            .body_mut()
            .read_json::<ModelsResponse>()?;

        println!("Available OpenAI GPT models:");
        for model in models.data {
            if model.id.starts_with("gpt-") && !model.id.contains("instruct") {
                println!("  {}", model.id);
            }
        }

        Ok(())
    }

    fn query(&self, model: &str, prompt: &str, streaming: bool) -> Result<()> {
        let query = json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_completion_tokens": 4096,
            "stream": streaming,
        });

        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build();

        let agent: ureq::Agent = config.into();

        let response = agent
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(query);

        let mut response = match response {
            Ok(resp) => resp,
            Err(e) => {
                return Err(anyhow::anyhow!("OpenAI request failed: {}", e));
            }
        };

        if response.status() != 200 {
            let status = response.status();

            let error_body = response
                .body_mut()
                .read_to_string()
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            return Err(anyhow::anyhow!(
                "OpenAI API error ({}): {}",
                status,
                error_body
            ));
        }

        if streaming {
            let reader = BufReader::new(response.body_mut().with_config().reader());

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                // Parse SSE format: "data: {...}"
                if let Some(data) = line.strip_prefix("data: ") {
                    // Check for end of stream
                    if data == "[DONE]" {
                        break;
                    }

                    // Parse JSON response
                    match serde_json::from_str::<OpenAIStreamResponse>(data) {
                        Ok(stream_response) => {
                            if let Some(choice) = stream_response.choices.first()
                                && let Some(content) = &choice.delta.content
                            {
                                print!("{}", content.as_str());
                            }
                        }
                        Err(e) => {
                            // Log parse errors but continue processing
                            eprintln!("Failed to parse streaming response: {}", e);
                        }
                    }
                }
            }
        } else {
            let response = response.body_mut().read_json::<OpenAIResponse>()?;

            for item in response.choices {
                if item.message.role == "assistant" {
                    println!("{}", item.message.content);
                }
            }
        }

        Ok(())
    }
}
