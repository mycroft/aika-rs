use serde::{
    Deserialize,
    Serialize,
};

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
    
    let response: ClaudeResponse = ureq::post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .send_json(query)?
            .body_mut()
            .read_json::<ClaudeResponse>()?;

    for item in response.content {
        if item.content_type == "text" {
            println!("{}", item.text);
        }
    }

    Ok(())
}
