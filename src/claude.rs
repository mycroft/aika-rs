use std::io::{BufRead as _, BufReader, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config::Config, provider::Provider as ProviderTrait};

#[derive(Debug, Serialize, Deserialize)]
struct Model {
    id: String,
    display_name: String,
    #[serde(rename = "type")]
    model_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelsResponse {
    data: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentItem {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentItem>,
}

// Streaming response structures for Claude
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(flatten)]
    data: ClaudeStreamData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ClaudeStreamData {
    ContentBlockDelta {
        index: u32,
        delta: ClaudeContentDelta,
    },
    MessageStart {
        message: ClaudeResponse,
    },
    MessageStop,
    Other(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeContentDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

pub struct ClaudeProvider {
    api_key: String,
    base_url: String,
    model: String,
}

const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";

impl ClaudeProvider {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let api_key: String = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| {
                config.credentials
                    .as_ref()
                    .and_then(|creds| creds.anthropic_api_key.clone())
                    .ok_or(std::env::VarError::NotPresent)
            })
            .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable is not set and no API key found in config"))?;

        let model = config
            .providers
            .get("anthropic")
            .map(|provider| provider.model.clone())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    
        Ok(Self {
            api_key,
            base_url: "https://api.anthropic.com".into(),
            model,
        })
    }
}

impl ProviderTrait for ClaudeProvider {
    fn model(&self) -> String {
        self.model.clone()
    }

    fn list_models(&self) -> anyhow::Result<()> {
        let url = format!("{}/v1/models", self.base_url);
        let response: ModelsResponse = ureq::get(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .call()?
            .body_mut()
            .read_json::<ModelsResponse>()?;

        println!("Available Claude models:");
        for model in response.data {
            println!("  {} - {}", model.id, model.display_name);
        }

        Ok(())
    }

    fn query(&self, model: &str, prompt: &str, streaming: bool) -> Result<String> {
        let url = format!("{}/v1/messages", self.base_url);
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

        let mut result = String::new();

        let config: ureq::config::Config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build();

        let agent: ureq::Agent = config.into();

        let response = agent
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .send_json(query);

        let mut response = match response {
            Ok(resp) => resp,
            Err(e) => {
                return Err(anyhow::anyhow!("Claude request failed: {}", e));
            }
        };

        if response.status() != 200 {
            let status = response.status();

            let error_body = response
                .body_mut()
                .read_to_string()
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            return Err(anyhow::anyhow!(
                "Claude API error ({}): {}",
                status,
                error_body
            ));
        }

        if !streaming {
            let response = response.body_mut().read_json::<ClaudeResponse>()?;

            for item in response.content {
                if item.content_type == "text" {
                    result.push_str(&item.text);
                }
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
                    match serde_json::from_str::<ClaudeStreamEvent>(data) {
                        Ok(stream_event) => {
                            match stream_event.data {
                                ClaudeStreamData::ContentBlockDelta { delta, .. } => {
                                    if let Some(text) = delta.text {
                                        // Call the callback with the chunk
                                        print!("{}", text);
                                        std::io::stdout().flush().unwrap();
                                    }
                                }
                                _ => {
                                    // Handle other event types if needed
                                }
                            }
                        }
                        Err(e) => {
                            // Log parse errors but continue processing
                            eprintln!("Failed to parse Claude streaming response: {}", e);
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::ServerGuard;

    fn setup_mock_server() -> ServerGuard {
        mockito::Server::new()
    }

    #[test]
    fn test_claude_query_success() {
        let mut server = setup_mock_server();

        let mock = server
            .mock("POST", "/v1/messages")
            .match_header("x-api-key", "test-key")
            .match_header("anthropic-version", "2023-06-01")
            .with_status(200)
            .with_body(
                r#"{
                  "content": [{"type": "text", "text": "Fix typo in readme"}],
                  "role": "assistant"
              }"#,
            )
            .create();

        let provider = ClaudeProvider {
            api_key: "test-key".to_string(),
            base_url: server.url(), // Point to mock server
            model: DEFAULT_MODEL.to_string(),
        };

        let result = provider.query("test input", DEFAULT_MODEL, false);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_claude_handles_api_error() {
        let mut server = setup_mock_server();

        let mock = server
            .mock("POST", "/v1/messages")
            .with_status(401)
            .with_body(r#"{"error": {"message": "Invalid API key"}}"#)
            .create();

        let provider = ClaudeProvider {
            api_key: "bad-key".to_string(),
            base_url: server.url(),
            model: DEFAULT_MODEL.to_string(),
        };

        let result = provider.query("test", DEFAULT_MODEL, false);

        mock.assert();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("401"));
    }

    #[test]
    fn test_claude_streaming_response() {
        let mut server = setup_mock_server();

        // Mock SSE stream
        let mock = server.mock("POST", "/v1/messages")
              .with_status(200)
              .with_header("content-type", "text/event-stream")
              .with_body("data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\ndata: {\"type\":\"message_stop\"}\n\n")
              .create();

        let provider = ClaudeProvider {
            api_key: "test-key".to_string(),
            base_url: server.url(),
            model: DEFAULT_MODEL.to_string(),
        };

        let result = provider.query("test", DEFAULT_MODEL, true);

        mock.assert();
        assert!(result.is_ok());
    }
}
