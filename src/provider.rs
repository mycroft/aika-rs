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
    fn list_models(&self) -> Result<()>;
    fn query(&self, message: &str, model: &str, streaming: bool) -> Result<()>;
}

/// Factory function to create AI providers
pub fn create_provider(provider_name: &str, config: &Config) -> Result<Box<dyn Provider>> {
    match provider_name {
        "anthropic" => Ok(Box::new(crate::claude::ClaudeProvider::new(config)?)),
        "openai" => Ok(Box::new(crate::openai::OpenAIProvider::new(config)?)),
        _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider_name)),
    }
}
