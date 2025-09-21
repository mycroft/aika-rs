use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

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

fn main() -> anyhow::Result<()> {
    let model = "claude-3-5-sonnet-latest";
    let prompt = "Explain the theory of relativity in simple terms.";

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
