# Agent Guide for xcom-rs

This guide is for coding agents working in this repository and focuses on practical Rust CLI work.

## Repository Facts (Source of Truth)

- Language: Rust edition 2021 (`Cargo.toml`, `rustfmt.toml`)
- Crate name: `xcom-rs` (binary CLI)
- Core dependencies:
  - `clap` with `derive`
  - `anyhow`
  - `serde`, `serde_json`, `serde_yaml`
  - `tracing`, `tracing-subscriber` (`json`, `env-filter`)
  - `uuid` (`v4`, `serde`)
  - `rusqlite` with `bundled`
  - `sha2`, `hex`, `dirs`
- Dev dependencies: `mockito`, `tempfile`
- Build toolchain: Cargo, with a Makefile wrapper for common tasks

## Fast Command Reference

```bash
# Build
cargo build
make build

# Optimized build
cargo build --release
make release

# Format and lint
cargo fmt
cargo fmt -- --check
cargo clippy -- -D warnings
make fmt
make lint

# Test and full checks
cargo test --verbose
make test
make check

# Cleanup and indexing
cargo install --path .
make install
cargo clean
make clean
make index
```

## Single-Test Execution Patterns (Use These First)

Use targeted tests before running the full suite.

```bash
# Exact test function name
cargo test test_parse_valid_input

# Test name pattern (substring match)
cargo test parse_valid

# Module-scoped pattern
cargo test parser::tests::

# Show println!/debug output while developing
cargo test parse_valid -- --nocapture

# Re-run one failing test with backtrace
RUST_BACKTRACE=1 cargo test test_parse_valid_input -- --nocapture
```

Workflow: run the narrowest test target, fix and re-run, then run `cargo test --verbose` and `make check`.

## Style and Formatting Rules

Formatting is defined in `rustfmt.toml`:
- `edition = "2021"`
- `max_width = 100`
- `hard_tabs = false`, `tab_spaces = 4`
- `newline_style = "Unix"`
- `reorder_imports = true`, `reorder_modules = true`

Always run `cargo fmt` after edits.

## Import Conventions

Prefer this grouping order:
1. `std` imports
2. external crate imports
3. `crate::...` imports

Let rustfmt reorder within groups. Avoid manual micro-formatting.

## Naming Conventions

- Functions, modules, variables: `snake_case`
- Structs, enums, traits: `PascalCase`
- Constants/statics: `SCREAMING_SNAKE_CASE`
- Lifetimes: short lowercase (`'a`, `'b`)
- Test names: explicit behavior format like
  `test_parse_config_with_missing_file_returns_error`

## Types and Error Handling

- In app/CLI flow, prefer `anyhow::Result<T>`.
- Propagate errors with `?`; do not swallow failures.
- Add context on fallible I/O or parse boundaries.
- Avoid `unwrap()` / `expect()` in production code.
- Use concrete error enums in library-style modules when callers need branching.

## Testing Guidelines

- Keep unit tests near code using `#[cfg(test)] mod tests`.
- Use integration tests in `tests/` for CLI or multi-module behavior.
- Cover happy path + failure path for any behavior change.
- For parser/IO changes, add regression tests for previous failures.
- Prefer deterministic tests; avoid clock/network dependencies unless mocked.

## Module Organization

- Keep `main.rs` focused on argument parsing and top-level orchestration.
- Put business logic in `lib.rs` and feature modules.
- Keep functions small and composable.
- Extract helpers when branching becomes hard to scan.
- Respect clippy cognitive complexity threshold (`30` in `clippy.toml`).

## Documentation Expectations

- Public APIs should have `///` doc comments.
- Non-trivial modules should have `//!` module docs.
- Include concise examples for externally consumable behavior.
- Keep docs implementation-accurate; update docs with code changes.

## Lint, Hooks, and Quality Gates

- Lint policy: `cargo clippy -- -D warnings` (warnings are errors).
- Pre-commit hook (installed by `make pre-commit-hooks`) runs:
  1. `cargo fmt -- --check`
  2. `cargo clippy -- -D warnings`
  3. `cargo test --quiet`
- `make setup` installs dev prerequisites and hooks.

## Agent Operating Rules for This Repo

- Prefer minimal, targeted diffs over broad rewrites.
- Verify behavior with focused tests first, then full checks.
- Do not add new dependencies unless justified by clear need.
- Follow existing patterns for `clap` derives, `anyhow` results, and tracing usage.
- Keep comments only where intent is not obvious from the code.

## External Agent Rule Scan Result

Repository scan found no Cursor rules and no Copilot instruction files
(no `.cursor/`, no `.cursorrules`, no `.github/copilot-instructions.md`).
No additional external agent rule sets apply beyond this file and global/system instructions.
