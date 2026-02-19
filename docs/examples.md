# xcom-rs CLI Examples

This document demonstrates the agent-friendly CLI capabilities of xcom-rs.

## Basic Commands

### List Available Commands

```bash
# JSON format (for agents)
xcom-rs commands --output json

# YAML format
xcom-rs commands --output yaml

# Human-readable text format
xcom-rs commands --output text
```

Example JSON output:
```json
{
  "ok": true,
  "type": "commands",
  "schemaVersion": 1,
  "data": {
    "commands": [
      {
        "name": "commands",
        "description": "List all available commands with metadata",
        "arguments": [],
        "risk": "safe",
        "hasCost": false
      },
      {
        "name": "schema",
        "description": "Get JSON schema for command input/output",
        "arguments": [
          {
            "name": "command",
            "description": "Command name to get schema for",
            "required": true,
            "type": "string"
          }
        ],
        "risk": "safe",
        "hasCost": false
      },
      {
        "name": "help",
        "description": "Get detailed help for a command including exit codes",
        "arguments": [
          {
            "name": "command",
            "description": "Command name to get help for",
            "required": true,
            "type": "string"
          }
        ],
        "risk": "safe",
        "hasCost": false
      }
    ]
  }
}
```

### Get Command Schema

```bash
# Get schema for the 'commands' command
xcom-rs schema --command commands --output json
```

Example output:
```json
{
  "ok": true,
  "type": "schema",
  "schemaVersion": 1,
  "data": {
    "command": "commands",
    "inputSchema": {
      "type": "object",
      "properties": {},
      "additionalProperties": false
    },
    "outputSchema": {
      "type": "object",
      "required": ["commands"],
      "properties": {
        "commands": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["name", "description", "arguments", "risk", "hasCost"],
            "properties": {
              "name": { "type": "string" },
              "description": { "type": "string" },
              "arguments": { "type": "array" },
              "risk": { "type": "string", "enum": ["safe", "low", "medium", "high"] },
              "hasCost": { "type": "boolean" }
            }
          }
        }
      }
    }
  }
}
```

### Get Detailed Help

```bash
# Get help including exit codes and error vocabulary
xcom-rs help schema --output json
```

Example output:
```json
{
  "ok": true,
  "type": "help",
  "schemaVersion": 1,
  "data": {
    "command": "schema",
    "description": "Get JSON schema for command input/output",
    "usage": "xcom-rs schema --command <name> [--output json|yaml|text]",
    "exitCodes": [
      { "code": 0, "description": "Success" },
      { "code": 2, "description": "Invalid argument or missing required argument" },
      { "code": 3, "description": "Authentication or authorization failed" },
      { "code": 4, "description": "Operation failed (network, rate limit, service unavailable, etc.)" }
    ],
    "errors": [
      { "code": "INVALID_ARGUMENT", "description": "Invalid argument provided", "isRetryable": false },
      { "code": "UNKNOWN_COMMAND", "description": "Command not recognized", "isRetryable": false },
      { "code": "RATE_LIMIT_EXCEEDED", "description": "Rate limit exceeded, retry after delay", "isRetryable": true },
      // ... more error codes
    ]
  }
}
```

## Error Handling

### Unknown Command

```bash
xcom-rs unknown-command
# Exit code: 2
```

Output:
```json
{
  "ok": false,
  "type": "error",
  "schemaVersion": 1,
  "error": {
    "code": "UNKNOWN_COMMAND",
    "message": "error: unrecognized subcommand 'unknown-command'...",
    "isRetryable": false
  }
}
```

## Logging and Tracing

### JSON Logging with Trace ID

```bash
# Enable JSON logging and set trace ID for correlation
xcom-rs commands --log-format json --trace-id req-12345 --output json
```

The stderr output will include structured logs:
```json
{"timestamp":"2026-02-07T12:00:00.000Z","level":"INFO","message":"Request started","traceId":"req-12345"}
{"timestamp":"2026-02-07T12:00:00.001Z","level":"INFO","message":"Executing commands command","traceId":"req-12345"}
{"timestamp":"2026-02-07T12:00:00.002Z","level":"INFO","message":"Command completed successfully","traceId":"req-12345"}
```

### Text Logging

```bash
# Default text logging (human-readable)
xcom-rs commands --log-format text
```

Stderr output:
```
2026-02-07T12:00:00.000Z  INFO Request started traceId=req-12345
2026-02-07T12:00:00.001Z  INFO Executing commands command
2026-02-07T12:00:00.002Z  INFO Command completed successfully
```

## Global Options

All commands support these global options:

- `--output <FORMAT>`: Output format (json, yaml, text, json-schema)
- `--non-interactive`: Disable interactive prompts (for agent use)
- `--trace-id <ID>`: Set trace ID for request correlation
- `--log-format <FORMAT>`: Log format (json, text)

## Exit Codes

The CLI uses a consistent exit code policy:

| Code | Meaning | Examples |
|------|---------|----------|
| 0 | Success | Command completed successfully |
| 2 | Invalid argument | Unknown command, missing required arg, invalid value |
| 3 | Authentication error | Invalid credentials, expired token |
| 4 | Operation failed | Network error, rate limit, service unavailable |

## Error Code Vocabulary

All errors include a machine-readable code:

| Code | Retryable | Description |
|------|-----------|-------------|
| `INVALID_ARGUMENT` | No | Invalid argument provided |
| `MISSING_ARGUMENT` | No | Required argument missing |
| `UNKNOWN_COMMAND` | No | Command not recognized |
| `AUTHENTICATION_FAILED` | No | Authentication credentials invalid |
| `AUTHORIZATION_FAILED` | No | Insufficient permissions |
| `RATE_LIMIT_EXCEEDED` | Yes | Rate limit exceeded |
| `NETWORK_ERROR` | Yes | Network connection failed |
| `SERVICE_UNAVAILABLE` | Yes | Service temporarily unavailable |
| `INTERNAL_ERROR` | No | Internal error occurred |

## Agent Integration Example

```python
import subprocess
import json
import sys

def run_xcom(args):
    """Run xcom-rs and return parsed JSON response."""
    cmd = ["xcom-rs"] + args + ["--output", "json"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    # Parse JSON output
    response = json.loads(result.stdout)
    
    # Check for errors
    if not response["ok"]:
        error = response["error"]
        if error["isRetryable"]:
            # Implement retry logic
            print(f"Retryable error: {error['code']}")
        else:
            print(f"Fatal error: {error['code']}: {error['message']}")
            sys.exit(result.returncode)
    
    return response["data"]

# List commands
commands = run_xcom(["commands"])
print(f"Available commands: {[c['name'] for c in commands['commands']]}")

# Get schema
schema = run_xcom(["schema", "--command", "commands"])
print(f"Schema for commands: {schema}")
```

## Shell Completions

Generate shell completion scripts to enable tab-completion for `xcom-rs` commands and flags.

### Bash

```bash
# Source completions for the current session
source <(xcom-rs completion --shell bash)

# Persist completions (requires bash-completion package)
xcom-rs completion --shell bash > ~/.local/share/bash-completion/completions/xcom-rs
```

### Zsh

```zsh
# Source completions for the current session
source <(xcom-rs completion --shell zsh)

# Persist to a fpath directory and reload
mkdir -p ~/.zsh/completions
xcom-rs completion --shell zsh > ~/.zsh/completions/_xcom-rs
# Add to ~/.zshrc: fpath=(~/.zsh/completions $fpath); autoload -Uz compinit && compinit
```

### Fish

```fish
# Source completions for the current session
xcom-rs completion --shell fish | source

# Persist completions
xcom-rs completion --shell fish > ~/.config/fish/completions/xcom-rs.fish
```

## Best Practices for Agents

1. **Always use JSON output**: `--output json` for machine-readable responses
2. **Enable JSON logging**: `--log-format json` for structured log parsing
3. **Use trace IDs**: `--trace-id <id>` for request correlation
4. **Check `ok` field**: Always check before accessing `data`
5. **Respect retry flags**: Only retry if `error.isRetryable` is `true`
6. **Parse exit codes**: Use exit codes for basic error categorization
7. **Schema introspection**: Use `schema` command to validate inputs/outputs
8. **Error code vocabulary**: Use `error.code` for programmatic error handling
