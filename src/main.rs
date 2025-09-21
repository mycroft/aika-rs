use std::path::PathBuf;

use clap::{Parser, Subcommand};

use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

#[derive(Parser)]
#[command(name = "aika")]
#[command(about = "A tool to use Claude AI from the command line", long_about = None)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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

fn list_anthropic_models() -> anyhow::Result<()> {
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

fn query_anthropic(model: &str, prompt: &str) -> anyhow::Result<()> {
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

    match &cli.command {
        Some(Commands::ListModels) => {
            list_anthropic_models()
        }
        Some(Commands::Query { model, input }) => {
            let default_prompt = get_commit_msg_prompt(&PathBuf::from("."));
            query_anthropic(model.as_deref().unwrap(), input.as_deref().unwrap_or(&default_prompt))
        }
        None => {
            Err(anyhow::anyhow!("No command provided. Use --help for usage information."))
        }
    }
}
