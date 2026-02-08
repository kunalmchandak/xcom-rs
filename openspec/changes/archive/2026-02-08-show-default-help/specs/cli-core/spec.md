# cli-core Specification Delta

## ADDED Requirements
### Requirement: Show Help on Default Launch
`xcom-rs` MUST display CLI help to standard output and exit successfully when launched without a subcommand.

#### Scenario: Launch without subcommand
- **Given** a user executes `xcom-rs` without arguments
- **When** the CLI launches
- **Then** CLI help is displayed to standard output
- **And** the CLI exits with status code `0`
- **And** the `commands` JSON Envelope is not output
