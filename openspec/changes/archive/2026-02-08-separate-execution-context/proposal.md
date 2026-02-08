# separate-execution-context Proposal

## Why
`ExecutionContext` currently holds validation logic (`check_interaction_required`, `check_max_cost`, `check_daily_budget`), violating the single responsibility principle. This makes the code harder to test and maintain. Separating execution context from business rules will improve testability and maintainability.

## What Changes
- Transform `ExecutionContext` into a pure data holder
- Extract validation logic into a separate type (`ExecutionPolicy` / `Validator`)
- Update existing tests to reflect the new structure
- Preserve existing validation behavior

### In Scope
- Refactor `ExecutionContext` to remove validation methods
- Create new `ExecutionPolicy` type for validation logic
- Update call sites to use the new structure

### Out of Scope
- Changing validation logic specifications
- Adding new validation rules

## Risks
- Changes may affect multiple call sites across the codebase
