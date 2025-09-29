use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod config;
use crate::config::{Provider, load_config};

pub mod claude;
use crate::claude::{list_anthropic_models, query_anthropic, query_anthropic_stream};

pub mod input;
use crate::input::{Input, from_config, get_input};

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

    match &cli.command {
        Some(Commands::ListModels) => list_anthropic_models(),
        Some(Commands::Query {
            stream,
            model,
            prompt,
            input,
        }) => {
            let input = if input.starts_with("file:") {
                let files = &input[5..]
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                get_input(&Input::Files(files.clone()), &PathBuf::from("."), cli.debug)
                    .expect("Failed to get input from file(s)")
            } else if input.starts_with("dir:") {
                let dir = &input[4..];
                get_input(&Input::Dir(dir.to_string()), &PathBuf::from("."), cli.debug)
                    .expect("Failed to get input from directory")
            } else {
                let input = config
                    .inputs
                    .get(&input.clone())
                    .map(|input| input)
                    .unwrap_or_else(|| {
                        eprintln!(
                            "Input '{}' not found in config, using default command.",
                            &input
                        );
                        &config.inputs.get("git-diff-cached").unwrap()
                    });

                get_input(&from_config(input), &PathBuf::from("."), cli.debug)
                    .expect("Failed to get input")
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

            if *stream {
                query_anthropic_stream(model, &prompt, Box::new(|text| print!("{}", text)))
            } else {
                query_anthropic(model, &prompt)
            }
        }
        None => Err(anyhow::anyhow!(
            "No command provided. Use --help for usage information."
        )),
    }
}
