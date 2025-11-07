use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::provider::Provider;

pub fn run_repl(provider: Box<dyn Provider>, model: Option<String>, debug: bool) -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    let model_name = model.as_deref().unwrap_or(&provider.model()).to_string();

    println!("Aika REPL - Interactive mode");
    println!("Provider: {}", provider.name());
    println!("Model: {}", model_name);
    println!("Type 'exit', 'quit', or press Ctrl+D to exit");
    println!("Type '/help' for available commands");
    println!();

    let mut conversation_history: Vec<(String, String)> = Vec::new();

    loop {
        let readline = rl.readline("aika> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                rl.add_history_entry(trimmed)?;

                // Handle special commands
                match trimmed {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "/help" => {
                        print_help();
                        continue;
                    }
                    "/clear" => {
                        conversation_history.clear();
                        println!("Conversation history cleared.");
                        continue;
                    }
                    "/history" => {
                        print_history(&conversation_history);
                        continue;
                    }
                    "/models" => {
                        provider.list_models()?;
                        continue;
                    }
                    _ if trimmed.starts_with("/") => {
                        println!(
                            "Unknown command: {}. Type '/help' for available commands.",
                            trimmed
                        );
                        continue;
                    }
                    _ => {}
                }

                // Send query to AI provider
                if debug {
                    println!("Sending query to {}...", provider.name());
                }

                match provider.query(&model_name, trimmed, false) {
                    Ok(response) => {
                        println!("\n{}\n", response);
                        conversation_history.push((trimmed.to_string(), response));
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  /help     - Show this help message");
    println!("  /clear    - Clear conversation history");
    println!("  /history  - Show conversation history");
    println!("  /models   - List available models");
    println!("  exit/quit - Exit the REPL");
    println!();
    println!("Just type your message to interact with the AI.");
}

fn print_history(history: &[(String, String)]) {
    if history.is_empty() {
        println!("No conversation history.");
        return;
    }

    println!("\nConversation History:");
    println!("━━━━━━━━━━━━━━━━━━━━");
    for (i, (prompt, response)) in history.iter().enumerate() {
        println!("\n[{}] User: {}", i + 1, prompt);
        println!("Assistant: {}", response);
    }
    println!();
}
