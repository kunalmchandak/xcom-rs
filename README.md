# xcom-rs

`xcom-rs` is an experimental, agent-friendly CLI for X.com-style operations.

The project is designed around predictable, machine-readable responses (JSON/YAML/NDJSON) while still
supporting a human-readable text mode.

## Features

- Output envelope with consistent `ok`/`error`/`meta` fields
- Output formats: `text`, `json`, `yaml`, `ndjson`
- Introspection helpers: `commands`, `schema`, `help`
- Tweet operations (demo): `tweets create`, `tweets list`
- Auth and billing helpers (local state)
- Embedded skill installer for agent toolchains (`install-skills`)

## Install

### Prerequisites

- Rust (1.70+)

### From crates.io

```bash
cargo install xcom-rs
```

### From Source

```bash
cargo install --path .
```

For development setup and Makefile workflows, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Quick Start

```bash
cargo install xcom-rs
xcom-rs commands
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

`xcom-rs` currently supports importing/exporting a bearer token for local use. OAuth login flows are
not implemented yet.

Check current auth status:

```bash
xcom-rs auth status --output json
```

Import credentials (expects a base64-encoded JSON token):

```bash
XCOM_AUTH_DATA="$(python - <<'PY'
import base64, json

token = {
  "accessToken": "YOUR_TOKEN",
  "tokenType": "bearer",
  "expiresAt": None,
  "scopes": ["tweet.read", "tweet.write"],
}

print(base64.b64encode(json.dumps(token).encode()).decode())
PY
)"

xcom-rs auth import "$XCOM_AUTH_DATA" --output json
```

By default, credentials are stored at `$XDG_DATA_HOME/xcom-rs/auth.json` or
`~/.local/share/xcom-rs/auth.json`.

Security note: the export/import payload is base64 (not encrypted). Treat it like a secret.

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

## Contributing

For developer setup, workflows, and repository layout, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
