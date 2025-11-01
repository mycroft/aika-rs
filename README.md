# aika-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A command-line tool that leverages multiple AI providers (Anthropic Claude, OpenAI, Mistral) to generate git commit messages, code reviews, and process text-based outputs. Built in Rust, aika-rs provides a seamless interface for interacting with AI models from your terminal.

## Features

- 🤖 Multiple AI provider support (Anthropic Claude, OpenAI, Mistral)
- 📝 Generate AI-powered commit messages from git diffs
- 🔍 Code review generation with specialized Rust review support
- 📋 List available models for each provider
- 🔧 Flexible input sources (files, directories, commands)
- 💬 Multiple prompt templates (commit messages, code reviews, README generation)
- 🔄 Support for streaming responses
- 📦 Text wrapping with paragraph support
- 🎯 Debug mode for detailed execution information

## Installation

### Prerequisites

- Rust toolchain (1.56 or later)
- API key for at least one supported provider:
  - Anthropic API key (for Claude models)
  - OpenAI API key (for GPT models)
  - Mistral API key (for Mistral models)
- Git (for commit message generation)

### Building from Source

```bash
git clone https://github.com/mycroft/aika-rs
cd aika-rs
cargo build --release
```

The binary will be available at `target/release/aika`

## Configuration

Create a configuration file at `~/.config/aika-rs/config.toml` (or specify custom location with `--config`):

```toml
[credentials]
anthropic_api_key = "your_anthropic_api_key"
openai_api_key = "your_openai_api_key"
mistral_api_key = "your_mistral_api_key"

[providers.claude]
model = "claude-sonnet-3-latest"

[inputs.git-diff-cached]
command = "git diff --cached"

[inputs.git-diff]
command = "git diff"

[prompts.commit-message]
prompt = """
Generate a concise and descriptive git commit message for the following changes:
- Use conventional commit style (type: short summary)
- Start with a lowercase imperative verb
- Keep the summary under 72 characters

{input}
"""

[prompts.review]
prompt = "Review the following code changes and provide feedback on correctness, readability, security, and performance.\n\n{input}"

[prompts.review-rust]
prompt = "As a senior Rust engineer, review the following code changes with focus on idiomatic Rust, ownership/borrowing, and safety.\n\n{input}"
```

See `contrib/config.toml` for a complete example configuration.

### Environment Variables

You can also set API keys via environment variables:

- `ANTHROPIC_API_KEY`: Your Anthropic API key
- `OPENAI_API_KEY`: Your OpenAI API key
- `MISTRAL_API_KEY`: Your Mistral API key

## Usage

### List Available Models

```bash
# List models for default provider (Claude)
aika list-models

# List models for specific provider
aika list-models --provider openai
aika list-models --provider mistral
```

### Generate Commit Message

```bash
# Using staged changes
aika query

# Using specific files
aika query --input "file:src/main.rs,README.md"

# Using a directory
aika query --input "dir:src"

# Using a specific model
aika query --model "claude-3-5-opus-latest"

# Using a specific provider
aika query --provider openai

# Using a custom prompt
aika query --prompt "commit-message"
```

### Code Review

```bash
# Review changes with default prompt
aika query --prompt review

# Rust-specific code review
aika query --prompt review-rust

# Review specific git diff
aika query --input "cmd:git diff HEAD~1" --prompt review
```

### Streaming Output

Enable streaming for real-time responses:
```bash
aika query --stream
```

### Debug Mode

Add `--debug` flag to see detailed execution information:

```bash
aika query --debug
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Dependencies

- AI Provider APIs:
  - [Anthropic Claude API](https://anthropic.com/) - Claude models
  - [OpenAI API](https://openai.com/) - GPT models
  - [Mistral API](https://mistral.ai/) - Mistral models
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [ureq](https://github.com/algesten/ureq) - HTTP client
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [toml](https://github.com/toml-rs/toml) - Configuration file parsing
- Additional dependencies can be found in `Cargo.toml`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
