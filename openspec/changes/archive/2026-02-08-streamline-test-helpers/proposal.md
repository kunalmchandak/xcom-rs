# streamline-test-helpers Proposal

## Why
Test code currently has excessive use of `unwrap` and dependencies on shared resources like `ENV_LOCK`, which reduces test readability and maintainability. Providing common test helpers will eliminate code duplication and improve test quality.

## What Changes
- Add test helper functions module
- Isolate environment-variable-dependent tests
- Reduce `unwrap` usage in test code

### In Scope
- Test helper function additions
- Environment variable dependency isolation

### Out of Scope
- Production code changes
- Test specification changes

### Deliverables
- Enhanced test helpers module
- Improved test readability

### Risks and Considerations
- Test execution order dependencies may remain
