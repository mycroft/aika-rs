use serde::{Deserialize, Serialize};

use serde_json::json;

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

pub fn list_anthropic_models() -> anyhow::Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable is not set"))?;

    let response: ModelsResponse = ureq::get("https://api.anthropic.com/v1/models")
        .header("x-api-key", &api_key)
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

pub fn query_anthropic(model: &str, prompt: &str) -> anyhow::Result<()> {
    let query = json!({
        "model": model,
        "temperature": 0.0,
        "messages": [{
            "role": "user",
            "content": prompt
        }],
        "max_tokens": 4096
    });

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable is not set"))?;

    let config = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build();

    let agent: ureq::Agent = config.into();

    let response = agent
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
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

    let response = response.body_mut().read_json::<ClaudeResponse>()?;

    for item in response.content {
        if item.content_type == "text" {
            println!("{}", item.text);
        }
    }

    Ok(())
}
