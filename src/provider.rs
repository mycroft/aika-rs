//! Provider trait for AI service integrations.
//!
//! This module defines the core trait that all AI service providers must implement
//! to be compatible with the aika CLI tool. It provides a unified interface for
//! interacting with different AI services like Anthropic's Claude, OpenAI's GPT, etc.
//!
//! # Example
//!
//! ```rust
//! use crate::provider::Provider;
//!
//! impl Provider for MyAIProvider {
//!     fn list_models(&self) -> Result<()> {
//!         // Implementation to fetch available models
//!         Ok(())
//!     }
//!
//!     fn query(&self, message: &str, model: &str, streaming: bool) -> Result<()> {
//!         // Implementation to send a message and get response
//!         Ok(())
//!     }
//! }
//! ```

use anyhow::Result;

use crate::config::Config;

pub trait Provider {
    fn model(&self) -> String;
    fn list_models(&self) -> Result<()>;
    fn query(&self, message: &str, model: &str, streaming: bool) -> Result<String>;
}

/// Factory function to create AI providers
pub fn create_provider(provider_name: &str, config: &Config) -> Result<Box<dyn Provider>> {
    match provider_name {
        "anthropic" => Ok(Box::new(crate::claude::ClaudeProvider::new(config)?)),
        "mistral" => Ok(Box::new(crate::mistral::MistralProvider::new(config)?)),
        "openai" => Ok(Box::new(crate::openai::OpenAIProvider::new(config)?)),
        _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider_name)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{Config, Credentials},
        provider::create_provider,
    };

    #[test]
    fn test_create_anthropic_provider() {
        let config = Config {
            credentials: Some(Credentials {
                anthropic_api_key: Some("test-anthropic-key".to_string()),
                openai_api_key: None,
                mistral_api_key: None,
            }),
            ..Default::default()
        };

        let provider = create_provider("anthropic", &config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_unsupported_provider_returns_error() {
        let config = Config::default();
        let provider = create_provider("invalid-provider", &config);
        assert!(provider.is_err());
        if let Err(err) = &provider {
            assert!(err.to_string().contains("Unsupported provider"));
        }
    }
}
