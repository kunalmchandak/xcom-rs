# Change Proposal: Stabilize CI Failures

## Why

GitHub Actions CI is failing across multiple jobs due to mixed failure factors. Specifically:
- Cross-platform test execution instability (particularly on Windows)
- Security audit job instability due to transient external factors

These failures block development workflow unnecessarily when the core build and tests are actually passing.

## What Changes

### In Scope
- Make integration test binary resolution cross-platform compatible
- Revise security audit job failure conditions to tolerate transient external failures

### Out of Scope
- Replacing the audit tool itself
- Changes to production build/release processes

## Expected Outcomes
- Integration tests run reliably across all platforms including Windows
- Audit job transient failures do not block the entire CI pipeline when builds and tests pass
