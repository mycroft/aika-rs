use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

pub mod config;
use crate::config::{Provider, load_config};

pub mod provider;
use crate::provider::create_provider;

pub mod claude;
pub mod mistral;
pub mod openai;

pub mod input;
use crate::input::{Input, from_config, get_input};

pub mod output;
use crate::output::wrap_text;

#[derive(Parser)]
#[command(name = "aika")]
#[command(about = "A tool to use Claude AI from the command line", long_about = None)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    config: Option<String>,

    #[arg(long, default_value_t = false)]
    debug: bool,

    #[arg(short, long, default_value = "anthropic")]
    provider: String,
}

#[derive(Subcommand)]
enum Commands {
    ListModels,
    Query {
        /// Input type: prompt to use; if empty, using a generic prompt using git diff --cached
        #[arg(short, long, default_value = "git-diff-cached")]
        input: String,

        /// Model to use (default: claude-3-5-sonnet-latest)
        #[arg(short, long, default_value = "claude-3-5-sonnet-latest")]
        model: Option<String>,

        /// Prompt to use; if empty, using a generic prompt
        #[arg(short, long, default_value = "commit-message")]
        prompt: Option<String>,

        /// Output style
        #[arg(short, long, default_value = "none")]
        output: String,

        /// Enable streaming output
        #[arg(short, long, default_value_t = false)]
        stream: bool,
    },
}

const DEFAULT_MODEL: &str = "claude-3-5-sonnet-latest";
const DEFAULT_PROMPT: &str = "commit-message";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = match load_config(cli.config.as_deref().unwrap_or("")) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Failed to load config file: {}", e);
            return Err(e);
        }
    };

    let provider = create_provider(&cli.provider, &config)?;

    match &cli.command {
        Some(Commands::ListModels) => provider.list_models(),
        Some(Commands::Query {
            stream: _,
            model: _,
            prompt: _,
            input: _,
            output: _,
        })
        | None => {
            // Use default values when no command is provided
            let (stream, model, prompt, input, output) = match &cli.command {
                Some(Commands::Query {
                    stream,
                    model,
                    prompt,
                    input,
                    output,
                }) => (*stream, model.clone(), prompt.clone(), input.clone(), output.clone()),
                None => (
                    false,                            // default stream
                    Some(DEFAULT_MODEL.to_string()),  // default model
                    Some(DEFAULT_PROMPT.to_string()), // default prompt
                    "git-diff-cached".to_string(),    // default input
                    "none".to_string(),               // default output
                ),
                _ => unreachable!(),
            };

            let input = if let Some(input) = input.strip_prefix("file:") {
                let files = &input
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                get_input(&Input::Files(files.clone()), &PathBuf::from("."), cli.debug)
                    .context("Failed to get input from files")?
            } else if let Some(dir) = input.strip_prefix("dir:") {
                get_input(&Input::Dir(dir.to_string()), &PathBuf::from("."), cli.debug)
                    .context("Failed to get input from directory")?
            } else {
                let input = config.inputs.get(&input.clone()).unwrap_or_else(|| {
                    eprintln!(
                        "Input '{}' not found in config, using default command.",
                        &input
                    );
                    config.inputs.get("git-diff-cached").unwrap()
                });

                get_input(&from_config(input), &PathBuf::from("."), cli.debug)
                    .context("Failed to get input from config")?
            };

            let prompt = config.prompts.get(&prompt.clone().unwrap_or(DEFAULT_PROMPT.to_string()))
                .map(|prompt| prompt.prompt.clone())
                .unwrap_or_else(|| "Generate a concise and descriptive git commit message for the following changes:\n\n```\n{input}\n```".to_string())
                .replace("{input}", &input);

            let default_provider = Provider {
                model: DEFAULT_MODEL.to_string(),
            };

            let model = model.as_deref().unwrap_or(
                config
                    .providers
                    .get("claude")
                    .unwrap_or(&default_provider)
                    .model
                    .as_str(),
            );

            let response = provider.query(model, &prompt, stream);
            if let Ok(response) = response {
                if !stream {
                    match output.as_str() {
                        "json" => {
                            let json_output = serde_json::json!({
                                "model": model,
                                "response": response,
                            });
                            println!("{}", json_output);
                        }
                        "wrapped" => {
                            let wrapped_response = wrap_text(&response, 80);
                            println!("{}", wrapped_response);
                        }
                        _ => println!("{}", &response),
                    }
                }
            } else {
                eprintln!("Error querying provider: {}", response.unwrap_err());
            }

            Ok(())
        }
    }
}
