use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod config;
use crate::config::{    
    load_config,
    Provider,
};

pub mod claude;
use crate::claude::{
    list_anthropic_models,
    query_anthropic,
};

#[derive(Parser)]
#[command(name = "aika")]
#[command(about = "A tool to use Claude AI from the command line", long_about = None)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    ListModels,
    Query {
        /// Input type: prompt to use; if empty, using a generic prompt using git diff --cached
        #[arg(short, long)]
        input: Option<String>,

        /// Model to use (default: claude-3-5-sonnet-latest)
        #[arg(short, long, default_value = "claude-3-5-sonnet-latest")]
        model: Option<String>,
    },
}

fn get_command_output(cmd: &Vec<&str>) -> anyhow::Result<String> {
    let output = std::process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute command {:?}: {}", cmd, e))?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Git command failed with status: {}",
            output.status
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse git output: {}", e))?;

    Ok(stdout)
}

fn get_commit_msg_prompt(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    let command = vec!["git", "diff", "--cached", &path_str];
    format!(
        "Generate a concise and descriptive git commit message for the following changes in {}:\n\n{}",
        path_str,
        get_command_output(&command).unwrap_or_else(|_| "No changes found.".to_string())
    )
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = match load_config(cli.config.as_deref().unwrap_or("")) {
        Ok(config)  => config,
        Err(e) => {
            eprintln!("Warning: Failed to load config file: {}", e);
            return Err(e);
        }
    };

    match &cli.command {
        Some(Commands::ListModels) => {
            list_anthropic_models()
        }
        Some(Commands::Query { model, input }) => {
            let default_model = "claude-3-5-sonnet-latest";
            let default_prompt = get_commit_msg_prompt(&PathBuf::from("."));
            query_anthropic(
                model.as_deref().unwrap_or(
                config.providers.get("claude").unwrap_or(&Provider{model: default_model.to_string()}).model.as_str()),
                input.as_deref().unwrap_or(&default_prompt),
            )
        }
        None => {
            Err(anyhow::anyhow!("No command provided. Use --help for usage information."))
        }
    }
}
