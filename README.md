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
- [prek](https://github.com/j178/prek) (pre-commit hooks, auto-installed by setup script)

### From Source

```bash
cargo install --path .
```

Or using the Makefile wrapper:

```bash
make install
```

### Development Setup

This installs `prek` (if missing) and installs git hooks:

```bash
./.wt/setup
```

## Quick Start

1. Clone the repository:
```bash
git clone <repository-url>
cd xcom-rs
```

2. Build the project:
```bash
make build
```

3. Run a command:

```bash
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
