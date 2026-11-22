---
name: secrets-and-commands
description: API keys and the commands aika runs
applies_to: ["src/**/*.rs", "contrib/**"]
---
aika holds API keys for three providers and runs commands named in its
configuration to build its inputs. Review the change for:

- API keys that reach output, debug output (`--debug` included), error
  messages, or files, or that are sent to a provider other than their own.
- Commands run through a shell, or built from anything other than the
  user's own configuration and arguments, such as model output or file
  contents.
- Model output written to files or executed without the user asking for it.
- Paths from inputs or configuration that are followed outside what the user
  named (symlinks, `..`).

Report only concrete, reachable problems in the changed code.
