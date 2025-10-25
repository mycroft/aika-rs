use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub anthropic_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    pub openai_api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Provider {
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub credentials: Option<Credentials>,
    pub providers: HashMap<String, Provider>,
    pub inputs: HashMap<String, Input>,
    pub prompts: HashMap<String, Prompt>,
}

impl Default for Config {
    fn default() -> Self {
        get_default_config()
    }
}

pub fn load_config(config_file: &str) -> Result<Config> {
    let default_config_path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("aika-rs")
        .join("config.toml");

    let config_path: PathBuf = if !config_file.is_empty() {
        config_file.into()
    } else {
        default_config_path
    };

    if !config_path.exists() {
        // Returning default config if file does not exist
        eprintln!(
            "Config file not found at {:?}, using default configuration.",
            config_path
        );
        return Ok(get_default_config());
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}

pub fn get_default_config() -> Config {
    let mut providers = HashMap::new();
    providers.insert(
        "claude".to_string(),
        Provider {
            model: "claude-3-5-sonnet-latest".to_string(),
        },
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "git-diff-cached".to_string(),
        Input {
            command: "git diff --cached".to_string(),
        },
    );

    let mut prompts = HashMap::new();
    prompts.insert(
        "commit-message".to_string(),
        Prompt {
            prompt: "Generate a concise and descriptive git commit message for the following changes:\n\n```\n{input}\n```".to_string(),
        },
    );

    Config {
        credentials: None,
        providers,
        inputs,
        prompts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let toml = r#"
              [credentials]
              anthropic_api_key = "test-key"

              [inputs.git-diff-cached]
              command = "git diff --cached"

              [providers.claude]
              model = "claude-3-5-sonnet-latest"

              [prompts.custom]
              prompt = "Custom prompt with {input}"
          "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.credentials.unwrap().anthropic_api_key,
            Some("test-key".into())
        );
    }

    #[test]
    fn test_missing_config_uses_defaults() {
        //let temp = TempDir::new().unwrap();
        let config = load_config("non-existent-file").unwrap();

        // Should have default prompts
        assert!(config.prompts.contains_key("commit-message"));
    }

    #[test]
    fn test_malformed_toml_returns_error() {
        let bad_toml = "this is { not valid toml";
        let result: Result<Config, _> = toml::from_str(bad_toml);
        assert!(result.is_err());
    }
}
