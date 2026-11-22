---
name: rust-correctness
description: Runtime bugs in the aika CLI
applies_to: ["**/*.rs"]
---
aika is a command-line tool that sends inputs (files, directories, command
output) to Anthropic, OpenAI or Mistral and prints the answer, streamed or
not. Review the changed Rust code for bugs that show up when it runs:

- Panics outside tests: `unwrap` and `expect` on API responses, config
  values or command output; indexing and slicing that can go out of bounds
  (an empty `choices` or `content` array, a short line in a stream).
- Streaming: server-sent event lines split across reads, `data: [DONE]`,
  events of other types, and partial UTF-8 at a chunk boundary.
- Provider differences: a request or response shape copied from one
  provider to another where they differ (system prompt placement, model
  listing, error bodies).
- Errors that are swallowed or lose their cause, so a failed request looks
  like an empty answer.
- Text wrapping that splits inside a multi-byte character or drops lines.

Do not report formatting, naming or Clippy lints: CI runs `cargo fmt` and
Clippy. Only report what is visible in the diff and the files given.
