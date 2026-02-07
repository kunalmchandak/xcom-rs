# xcom-rs

A CLI application written in Rust.

## Setup

### Prerequisites

- Rust (1.70+)
- pre-commit

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd xcom-rs
```

2. Install pre-commit hooks:
```bash
pre-commit install
```

3. Build the project:
```bash
cargo build
```

## Development

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality:

- **trailing-whitespace**: Removes trailing whitespace
- **end-of-file-fixer**: Ensures files end with a newline
- **check-yaml**: Validates YAML files
- **check-toml**: Validates TOML files
- **fmt**: Runs `cargo fmt` to format code
- **cargo-check**: Runs `cargo check` to verify compilation
- **clippy**: Runs `cargo clippy` for linting

## License

MIT
