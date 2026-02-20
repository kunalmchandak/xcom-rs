# xcom-rs

`xcom-rs` is an experimental, agent-friendly CLI for X.com-style operations.

The project is designed around predictable, machine-readable responses (JSON/YAML/NDJSON) while still
supporting a human-readable text mode.

## Features

- Output envelope with consistent `ok`/`error`/`meta` fields
- Output formats: `text`, `json`, `yaml`, `ndjson`
- Introspection helpers: `commands`, `schema`, `help`
- Tweet operations: `tweets create`, `tweets list`, `tweets like`, `tweets unlike`, `tweets retweet`,
  `tweets reply`, `tweets thread`, `tweets show`, `tweets conversation`
- Search: `search recent`, `search users`
- Timeline: `timeline home`, `timeline mentions`, `timeline user`
- Media: `media upload`
- Bookmarks: `bookmarks add`, `bookmarks remove`, `bookmarks list`
- Auth and billing helpers (local state): `auth status`
- Diagnostics: `doctor` (with optional `--probe` for API connectivity check)
- Embedded skill installer for agent toolchains (`install-skills`)

## Install

### Prerequisites

- Rust (1.70+)

### From crates.io

```bash
cargo install xcom-rs
```

### Homebrew

This repository includes a Homebrew formula at `Formula/xcom_rs.rb`.

To install via a tap (recommended for now):

```bash
brew tap tumf/tap
brew install xcom_rs
```

Notes:

- The formula name is `xcom_rs`, but the installed binary is `xcom-rs`.
- If you don't have a tap yet, copy `Formula/xcom_rs.rb` into your tap repo under `Formula/`.

### From Source

```bash
cargo install --path .
```

For development setup and Makefile workflows, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Quick Start

```bash
# 1. Install
cargo install xcom-rs

# 2. Set your bearer token as an environment variable
export XCOM_RS_BEARER_TOKEN="your_bearer_token_here"
# Optionally, set scopes for scope diagnostics
export XCOM_RS_SCOPES="tweet.read tweet.write users.read"

# 3. Verify setup with auth status
xcom-rs auth status --output json

# 4. Verify setup with doctor
xcom-rs doctor --output json

# 5. Create your first tweet
xcom-rs tweets create "Hello from xcom-rs!" --output json

# 6. Browse your home timeline
xcom-rs timeline home --limit 5 --output json
```

## Usage

### Global Flags

- `--output {text|json|yaml|ndjson}`
- `--log-format {text|json}`
- `--trace-id <id>`
- `--non-interactive`
- `--dry-run`
- `--max-cost-credits <n>`
- `--budget-daily-credits <n>`

### Authentication

`xcom-rs` uses environment variables for authentication. Set the following environment variables:

- **`XCOM_RS_BEARER_TOKEN`** (required): Your bearer token. Can be in `Bearer <token>` format or raw token.
- **`XCOM_RS_SCOPES`** (optional): Space-separated or comma-separated list of OAuth scopes for diagnostics.
- **`XCOM_RS_EXPIRES_AT`** (optional): UNIX epoch timestamp for token expiration.

Example:

```bash
export XCOM_RS_BEARER_TOKEN="your_bearer_token_here"
export XCOM_RS_SCOPES="tweet.read tweet.write users.read"
```

Check current auth status:

```bash
xcom-rs auth status --output json
```

Security note: Keep your bearer token secure. Never commit it to version control or expose it in logs.

### Examples

List agent-facing command metadata:

```bash
xcom-rs commands --output json
```

Get a JSON schema for a command response envelope:

```bash
xcom-rs schema --command commands --output json
```

Create a tweet (demo):

```bash
xcom-rs tweets create "Hello from xcom-rs" --output json
```

List tweets (demo):

```bash
xcom-rs tweets list --limit 5 --output text
```

Install embedded skills:

```bash
xcom-rs install-skills --yes
```

## Shell Completions

`xcom-rs` can generate shell completion scripts for Bash, Zsh, and Fish.

### Bash

```bash
# Generate and source immediately
source <(xcom-rs completion --shell bash)

# Or persist to a file (reload your shell afterwards)
xcom-rs completion --shell bash > ~/.local/share/bash-completion/completions/xcom-rs
```

### Zsh

```zsh
# Generate and source immediately
source <(xcom-rs completion --shell zsh)

# Or add to your fpath (e.g. ~/.zsh/completions/)
mkdir -p ~/.zsh/completions
xcom-rs completion --shell zsh > ~/.zsh/completions/_xcom-rs

# Ensure the directory is in your fpath (add to ~/.zshrc if not already present)
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

### Fish

```fish
xcom-rs completion --shell fish | source

# Or persist to the completions directory
xcom-rs completion --shell fish > ~/.config/fish/completions/xcom-rs.fish
```

## Contributing

For developer setup, workflows, and repository layout, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Docs

- CLI examples: [docs/examples.md](docs/examples.md)
- Implementation notes: `docs/implementation/`
- Validation reports: `docs/validation/`

## License

MIT
