# cli-core Specification Delta

## ADDED Requirements
### Requirement: Early Error Output Format Alignment
`xcom-rs` MUST return early errors (such as argument parsing failures or invalid `--log-format` values) using `text` as the default output format when `--output` is not specified. When `--output` is explicitly provided, early errors MUST respect the specified format.

#### Scenario: Early error with default output
- **Given** user runs `xcom-rs --log-format invalid commands` without specifying `--output`
- **When** CLI returns an error
- **Then** error is output in `text` format
- **And** CLI returns exit code `2`

#### Scenario: Early error without subcommand
- **Given** user runs `xcom-rs auth` without specifying `--output`
- **When** CLI returns an error
- **Then** error is output in `text` format
- **And** CLI returns exit code `2`

#### Scenario: Early error with explicit output format
- **Given** user runs `xcom-rs --output json --log-format invalid commands`
- **When** CLI returns an error
- **Then** error is output in JSON `Envelope` format
- **And** CLI returns exit code `2`

#### Scenario: Early error with invalid output format
- **Given** user runs `xcom-rs auth --output txt`
- **When** CLI returns an error
- **Then** invalid `--output` value falls back to default `text` and error is output in `text` format
- **And** CLI returns exit code `2`
