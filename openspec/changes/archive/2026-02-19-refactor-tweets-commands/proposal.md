# refactor-tweets-commands Proposal

## Why
- `src/tweets/commands.rs` is oversized, mixing argument definitions, implementations, and tests in one file, making it hard to navigate.
- Changes to add functionality concentrate edits in a single file, increasing merge conflicts and review burden.

## What Changes
- Split the `tweets` command's arguments, implementations, and tests into feature-oriented modules.
- Preserve existing public APIs and behaviour; reorganise internal structure only.

### Scope
- Module decomposition of `tweets` command implementations and tests
- Relocation of existing tests and addition of regression tests

### Out of Scope
- Changes to CLI arguments or output specifications
- Addition of new APIs

### Deliverables
- Feature-scoped modules (create / list / engagement / thread, etc.)
- Regression tests for key flows

### Acceptance Criteria
- Output of all `tweets` subcommands remains unchanged
- `cargo test` and `cargo clippy -- -D warnings` pass

### Risks / Notes
- Module visibility adjustments may be required when reorganising the test layout
