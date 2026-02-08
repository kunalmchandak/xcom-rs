# cli-error-response Specification

## Purpose
TBD - created by archiving change centralize-cli-errors. Update Purpose after archive.
## Requirements
### Requirement: Unified Failure Response Generation
`xcom-rs` MUST construct failure response `Envelope` and `ErrorDetails` through a common generation path.

#### Scenario: Common Helper Usage
- **Given** any CLI command has failed
- **When** the CLI generates an error response
- **Then** the failure response is constructed via a common helper
- **And** the existing `error.code`, `error.message`, and `error.isRetryable` format is preserved

