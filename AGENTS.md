# Agent Development Guide for xcom-rs

This document provides essential information for AI coding agents working in this Rust CLI project.

## Project Overview

- **Language**: Rust (edition 2021, requires Rust 1.70+)
- **Project Type**: CLI application using `clap` for argument parsing
- **Dependencies**: `clap` (v4.5 with derive features), `anyhow` (v1.0)
- **Build System**: Cargo with Make wrapper for common tasks

## Build Commands

### Basic Operations

```bash
# Build debug version
cargo build
make build

# Build release version (optimized)
cargo build --release
make release

# Install to ~/.cargo/bin
cargo install --path .
make install
```

### Testing

```bash
# Run all tests
cargo test
make test

# Run all tests with verbose output
cargo test --verbose

# Run a single test by name
cargo test test_greet_with_name

# Run tests in a specific module
cargo test tests::

# Run tests matching a pattern
cargo test greet
```

### Code Quality

```bash
# Format code (auto-fix)
cargo fmt
make fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run clippy linter (treats warnings as errors)
cargo clippy -- -D warnings
make lint

# Run all checks: format, lint, test
make check

# Clean build artifacts
cargo clean
make clean
```

## Code Style Guidelines

### Formatting

- **Edition**: 2021
- **Line width**: 100 characters maximum
- **Indentation**: 4 spaces (no hard tabs)
- **Newline style**: Unix (LF)
- **Import ordering**: Imports are automatically reordered by `rustfmt`
- **Module ordering**: Modules are automatically reordered by `rustfmt`

Configuration is enforced via `rustfmt.toml`.

### Imports

```rust
// Standard library imports first
use std::collections::HashMap;
use std::fs;

// External crate imports
use anyhow::Result;
use clap::Parser;

// Internal crate imports
use crate::module_name;
```

Order: std → external crates → crate-internal. Let `rustfmt` handle ordering automatically.

### Naming Conventions

- **Functions/variables**: `snake_case`
- **Types/traits/enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Lifetimes**: short lowercase letters (`'a`, `'b`)
- **Type parameters**: single uppercase letter or `PascalCase` (`T`, `ParseError`)

### Types and Error Handling

- Use `anyhow::Result<T>` for functions that may error in `main.rs` and application code
- Return concrete error types in library code when callers need to handle specific errors
- Use `?` operator for error propagation
- Prefer `Option<T>` pattern matching over `.unwrap()` or `.expect()` unless you can prove the value exists
- Add meaningful context to errors: `context("failed to read config")?`

```rust
// Good: Application code
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    process(&args)?;
    Ok(())
}

// Good: Library code with pattern matching
pub fn greet(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("Hello, {}!", n),
        None => String::from("Hello, world!"),
    }
}
```

### Documentation

- Add doc comments (`///`) for all public items
- Include examples in doc comments for non-trivial functions
- Use `//!` for module-level documentation

```rust
/// Greet a person by name, or greet the world if no name is provided
///
/// # Examples
///
/// ```
/// use xcom_rs::greet;
/// assert_eq!(greet(Some("Alice")), "Hello, Alice!");
/// ```
pub fn greet(name: Option<&str>) -> String {
    // implementation
}
```

### Code Organization

- Keep `main.rs` minimal - parse args and delegate to library code
- Put business logic in `lib.rs` or separate modules
- Group related functionality into modules
- Write tests in the same file as the code they test using `#[cfg(test)]`

## Linting Rules

Clippy configuration (`clippy.toml`):
- **Cognitive complexity threshold**: 30

All clippy warnings are treated as errors in CI/pre-commit hooks.

## Testing Best Practices

- Place unit tests in a `tests` module within the same file as the code
- Use descriptive test names: `test_function_name_condition_expected_result`
- Test both success and error cases
- Use `assert_eq!` for equality checks
- Use `assert!` for boolean conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_with_name() {
        assert_eq!(greet(Some("Alice")), "Hello, Alice!");
    }

    #[test]
    fn test_greet_without_name() {
        assert_eq!(greet(None), "Hello, world!");
    }
}
```

## Pre-commit Hooks

This project uses pre-commit hooks (via `prek` or native git hooks) that run:
1. `cargo fmt -- --check` - formatting verification
2. `cargo clippy -- -D warnings` - linting
3. `cargo test --quiet` - test suite

Install hooks with: `make setup` or `make pre-commit-hooks`

## Version Management

```bash
make bump-patch  # 0.1.0 → 0.1.1
make bump-minor  # 0.1.0 → 0.2.0
make bump-major  # 0.1.0 → 1.0.0
make publish     # Publish to crates.io (requires confirmation)
```

## Common Patterns

- Use `clap` derive macros for CLI argument parsing
- Prefer `&str` over `String` for function parameters when possible
- Use `as_deref()` to convert `Option<String>` to `Option<&str>`
- Return `anyhow::Result<()>` from `main()`
