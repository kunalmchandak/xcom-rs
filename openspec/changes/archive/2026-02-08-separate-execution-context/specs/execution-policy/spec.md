# execution-policy Specification Delta

## ADDED Requirements
### Requirement: Separation of Execution Policy
`xcom-rs` MUST separate runtime validation logic from `ExecutionContext`.

#### Scenario: Independent validation execution
- **Given** a developer wants to test execution policy
- **When** instantiating `ExecutionPolicy` standalone
- **Then** validation logic can be tested without depending on `ExecutionContext`
- **And** existing behavior is preserved
