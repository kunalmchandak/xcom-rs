# Contributing

## Development

### Available Make Commands

```bash
make help              # Show all available commands
make build             # Build debug version
make release           # Build optimized release version
make test              # Run all tests
make fmt               # Format code with rustfmt
make lint              # Run clippy linter
make check             # Run all checks (fmt, lint, test)
make clean             # Clean build artifacts
make install           # Install binary to ~/.cargo/bin
make setup             # Setup development environment
make pre-commit-hooks  # Install git pre-commit hooks
```

### Version Management

```bash
make bump-patch        # Bump patch version (0.1.0 -> 0.1.1)
make bump-minor        # Bump minor version (0.1.0 -> 0.2.0)
make bump-major        # Bump major version (0.1.0 -> 1.0.0)
make publish           # Publish to crates.io
make publish-tag       # Publish specific git tag
```

### Running

```bash
cargo run
cargo run -- --name "Alice"  # With arguments
```

### Testing

```bash
cargo test              # Run all tests
cargo test --verbose    # Verbose output
```

## Pre-commit Hooks

This project uses [prek](https://github.com/j178/prek) for pre-commit hooks:

- **trailing-whitespace**: Removes trailing whitespace
- **end-of-file-fixer**: Ensures files end with a newline
- **check-yaml**: Validates YAML files
- **check-toml**: Validates TOML files
- **fmt**: Runs `cargo fmt` to format code
- **cargo-check**: Runs `cargo check` to verify compilation
- **clippy**: Runs `cargo clippy` for linting

Hooks are automatically installed by the `.wt/setup` script.

## Project Structure

```
xcom-rs/
├── .wt/                    # Worktree setup scripts
│   └── setup               # Development environment setup
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library code with tests
│   ├── cli.rs              # CLI argument parsing
│   ├── context.rs          # Execution context
│   ├── output.rs           # Output formatting
│   ├── protocol.rs         # Protocol definitions
│   ├── auth/               # Authentication module
│   │   ├── mod.rs          # Module exports
│   │   ├── models.rs       # Auth data models
│   │   └── storage.rs      # Auth storage & persistence
│   ├── billing/            # Billing and cost tracking module
│   │   ├── mod.rs          # Module exports
│   │   ├── models.rs       # Billing data models
│   │   └── storage.rs      # Budget tracking & cost estimation
│   └── tweets/             # Tweets operations module
│       ├── mod.rs          # Module exports
│       ├── models.rs       # Tweet data models
│       ├── commands.rs     # Tweet command operations
│       └── ledger.rs       # Idempotency ledger storage
├── Cargo.toml              # Project manifest
├── Makefile                # Build and development tasks
├── .pre-commit-config.yaml # Pre-commit hooks configuration
├── README.md               # User-facing project overview
└── CONTRIBUTING.md         # Developer guide
```

### Module Organization

Each domain module (`auth`, `billing`, `tweets`) follows a consistent structure:

- `mod.rs` - Public API and re-exports
- `models.rs` - Data structures and types
- `storage.rs` or `ledger.rs` - Persistence and storage logic
- `commands.rs` - Business logic and operations (when applicable)

This organization keeps related functionality together while maintaining clear separation of concerns.
