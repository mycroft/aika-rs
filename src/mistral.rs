use std::io::{BufRead as _, BufReader, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config::Config, provider::Provider as ProviderTrait};

pub struct MistralProvider {
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Model {
    id: String,
    #[serde(rename = "type")]
    model_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelsResponse {
    data: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralMessage {
    content: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralChoice {
    message: MistralMessage,
    finish_reason: String,
    index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralResponse {
    choices: Vec<MistralChoice>,
    id: String,
    object: String,
    created: u64,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralStreamChoice {
    delta: MistralStreamDelta,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralStreamDelta {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralStreamResponse {
    choices: Vec<MistralStreamChoice>,
    id: String,
    object: String,
    created: u64,
    model: String,
}

impl MistralProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key: String = std::env::var("MISTRAL_API_KEY")
            .or_else(|_| {
                config
                    .credentials
                    .as_ref()
                    .and_then(|creds| creds.mistral_api_key.clone())
                    .ok_or(std::env::VarError::NotPresent)
            })
            .map_err(|_| {
                anyhow::anyhow!(
                    "MISTRAL_API_KEY environment variable is not set and no API key found in config"
                )
            })?;
        Ok(Self { api_key })
    }
}

impl ProviderTrait for MistralProvider {
    fn list_models(&self) -> Result<()> {
        let response: ModelsResponse = ureq::get("https://api.mistral.ai/v1/models")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .call()?
            .body_mut()
            .read_json()?;

        println!("Available Mistral models:");
        for model in response.data {
            println!("  {}", model.id);
        }

        Ok(())
    }

    fn query(&self, model: &str, prompt: &str, streaming: bool) -> Result<String> {
        let mut result = String::new();

        let query = json!({
            "model": model,
            "temperature": 0.0,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "max_tokens": 4096,
            "stream": streaming,
        });

        let config: ureq::config::Config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build();

        let agent: ureq::Agent = config.into();

        let response = agent
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .send_json(query);

        let mut response = match response {
            Ok(resp) => resp,
            Err(e) => {
                return Err(anyhow::anyhow!("Mistral request failed: {}", e));
            }
        };

        if response.status() != 200 {
            let status = response.status();
            let error_body = response
                .body_mut()
                .read_to_string()
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            return Err(anyhow::anyhow!(
                "Mistral API error ({}): {}",
                status,
                error_body
            ));
        }

        if !streaming {
            let response = response.body_mut().read_json::<MistralResponse>()?;
            if let Some(response) = response.choices.first() {
                result.push_str(response.message.content.as_str());
            } else {
                println!("No response from Mistral.");
            }
        } else {
            let reader = BufReader::new(response.body_mut().with_config().reader());

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                // Parse SSE format: "data: {...}"
                if let Some(data) = line.strip_prefix("data: ") {
                    // Parse JSON response
                    match serde_json::from_str::<MistralStreamResponse>(data) {
                        Ok(stream_event) => {
                            if let Some(choice) = stream_event.choices.first()
                                && let Some(content) = &choice.delta.content
                            {
                                print!("{}", content);
                                std::io::stdout().flush().unwrap();
                            }
                        }
                        Err(e) => {
                            // Log parse errors but continue processing
                            eprintln!("Failed to parse Mistral streaming response: {}", e);
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
