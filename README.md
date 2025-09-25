# aika-rs

A command-line tool to interact with Claude AI, primarily focused on generating git commit messages and other text-based outputs using Anthropic's Claude models.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Generate commit messages from git diffs using Claude AI
- List available Claude models
- Configurable input sources (files, directories, commands)
- Custom prompts support
- TOML-based configuration

## Installation

### Prerequisites

- Rust toolchain (1.56 or later)
- Anthropic API key
- Git (for commit message generation)

### Building from Source

```bash
git clone https://github.com/yourusername/aika-rs
cd aika-rs
cargo build --release
```

The binary will be available at `target/release/aika`

## Configuration

Create a configuration file at `~/.config/aika-rs/config.toml` (or specify custom location with `--config`):

```toml
[providers.claude]
model = "claude-3-5-sonnet-latest"

[inputs.git-diff-cached]
command = "git diff --cached"

[prompts.commit-message]
prompt = "Generate a concise and descriptive git commit message for the following changes:\n\n```\n{input}\n```"
```

### Environment Variables

- `ANTHROPIC_API_KEY`: Your Anthropic API key (required)

## Usage

### List Available Models

```bash
aika list-models
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

# Using a custom prompt
aika query --prompt "custom-prompt-name"
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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Anthropic's Claude API](https://anthropic.com/)
- Uses [clap](https://github.com/clap-rs/clap) for CLI argument parsing
- Uses [ureq](https://github.com/algesten/ureq) for HTTP requests
