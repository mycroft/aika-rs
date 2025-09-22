use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Provider {
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, Provider>,
    pub inputs: HashMap<String, Input>,
}

pub fn load_config(config_file: &str) -> Result<Config> {
    let default_config_path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("aika-rs")
        .join("config.toml");

    let config_path: PathBuf = if !config_file.is_empty() {
        config_file.into()
    } else {
        default_config_path.to_str().unwrap().to_string().into()
    };

    if !config_path.exists() {
        // Returning default config if file does not exist
        eprintln!("Config file not found at {:?}, using default configuration.", config_path);
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

    Config { providers, inputs }
}