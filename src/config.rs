use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Provider {
    pub model: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, Provider>,
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
        return Err(anyhow::anyhow!(
            "Config file not found at {}",
            config_path.display()
        ));
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}