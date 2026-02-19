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

## Tweet Operations

### Create a Tweet

```bash
xcom-rs tweets create "Hello from xcom-rs!" --output json
```

### Reply to a Tweet

```bash
xcom-rs tweets reply <tweet_id> "Great point!" --output json
```

### Post a Thread

Post multiple tweets as a sequential thread (first tweet is standalone; the rest are replies):

```bash
xcom-rs tweets thread "First tweet in the thread" "Second tweet" "Third tweet" --output json
```

With idempotency prefix:

```bash
xcom-rs tweets thread "Part 1" "Part 2" --client-request-id-prefix "my-thread-001" --output json
```

### Like / Unlike a Tweet

```bash
xcom-rs tweets like <tweet_id> --output json
xcom-rs tweets unlike <tweet_id> --output json
```

### Retweet / Unretweet

```bash
xcom-rs tweets retweet <tweet_id> --output json
xcom-rs tweets unretweet <tweet_id> --output json
```

### Show a Single Tweet

```bash
xcom-rs tweets show <tweet_id> --output json
```

### Retrieve a Conversation Tree

```bash
xcom-rs tweets conversation <tweet_id> --output json
```

## Search

### Search Recent Tweets

```bash
xcom-rs search recent "rust programming" --limit 20 --output json
```

### Search Users

```bash
xcom-rs search users "alice" --limit 10 --output json
```

## Timeline

### Home Timeline

```bash
# Get the 10 most recent tweets from the home feed
xcom-rs timeline home --output json

# With custom limit and pagination cursor
xcom-rs timeline home --limit 20 --cursor "<next_cursor_token>" --output json
```

### Mentions Timeline

```bash
xcom-rs timeline mentions --limit 10 --output json
```

### User Timeline

```bash
# Get tweets from a specific user handle (without @)
xcom-rs timeline user johndoe --limit 10 --output json
```

## Media

### Upload a Media File

```bash
# Upload an image and get back a media_id for use in tweets
xcom-rs media upload /path/to/image.jpg --output json
```

Example output:

```json
{
  "ok": true,
  "type": "media.upload",
  "schemaVersion": 1,
  "data": {
    "mediaId": "1234567890",
    "mimeType": "image/jpeg",
    "size": 102400
  }
}
```

## Bookmarks

### Add a Tweet to Bookmarks

```bash
xcom-rs bookmarks add <tweet_id> --output json
```

### Remove a Tweet from Bookmarks

```bash
xcom-rs bookmarks remove <tweet_id> --output json
```

### List Bookmarked Tweets

```bash
xcom-rs bookmarks list --limit 20 --output json

# With pagination cursor
xcom-rs bookmarks list --limit 10 --cursor "<next_cursor_token>" --output json
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

## Diagnostics

### Basic diagnostic check (no network calls)

```bash
xcom-rs doctor --output json
```

Example output:
```json
{
  "ok": true,
  "type": "doctor",
  "schemaVersion": 1,
  "data": {
    "authStatus": {
      "authenticated": true,
      "authMode": "oauth2",
      "scopes": ["tweet.read", "tweet.write", "users.read", "bookmark.read",
                 "bookmark.write", "like.read", "like.write", "offline.access"]
    },
    "executionMode": {
      "nonInteractive": false,
      "dryRun": false
    },
    "scopeCheck": {
      "ok": true,
      "grantedScopes": ["tweet.read", "tweet.write", "users.read", "bookmark.read",
                        "bookmark.write", "like.read", "like.write", "offline.access"],
      "missingScopes": []
    }
  }
}
```

### Diagnostic check with API connectivity probe

Pass `--probe` to perform a TCP connectivity check to `api.twitter.com:443`.
The probe is **skipped** (no network access) when the flag is omitted.

```bash
xcom-rs doctor --probe --output json
```

Example output when probe succeeds:
```json
{
  "ok": true,
  "type": "doctor",
  "schemaVersion": 1,
  "data": {
    "authStatus": { "authenticated": true, "authMode": "oauth2" },
    "executionMode": { "nonInteractive": false, "dryRun": false },
    "scopeCheck": { "ok": true, "grantedScopes": ["tweet.read", "..."], "missingScopes": [] },
    "apiProbe": {
      "status": "ok",
      "httpStatus": 200,
      "message": "API is reachable"
    }
  }
}
```

Example output when probe fails (no network / firewall):
```json
{
  "ok": true,
  "type": "doctor",
  "schemaVersion": 1,
  "data": {
    "authStatus": { "authenticated": true, "authMode": "oauth2" },
    "executionMode": { "nonInteractive": false, "dryRun": false },
    "scopeCheck": { "ok": true, "grantedScopes": ["tweet.read", "..."], "missingScopes": [] },
    "apiProbe": {
      "status": "failed",
      "message": "TCP connection to api.twitter.com:443 failed: Connection refused (os error 111)"
    },
    "warnings": ["API probe failed: TCP connection to api.twitter.com:443 failed: ..."],
    "nextSteps": [
      "Check network connectivity to api.twitter.com",
      "Verify that your access token is valid and not expired"
    ]
  }
}
```

### Missing OAuth scopes

When the token lacks required scopes, `scopeCheck.ok` is `false` and
`warnings` / `nextSteps` guide remediation:

```json
{
  "ok": true,
  "type": "doctor",
  "schemaVersion": 1,
  "data": {
    "authStatus": { "authenticated": true, "authMode": "oauth2",
                    "scopes": ["tweet.read"] },
    "executionMode": { "nonInteractive": false, "dryRun": false },
    "scopeCheck": {
      "ok": false,
      "grantedScopes": ["tweet.read"],
      "missingScopes": ["tweet.write", "users.read", "bookmark.read",
                        "bookmark.write", "like.read", "like.write", "offline.access"]
    },
    "warnings": ["Missing required OAuth scopes: tweet.write, users.read, ..."],
    "nextSteps": [
      "Re-authenticate with the required scopes: xcom-rs auth ...",
      "Missing scopes: tweet.write, users.read, ..."
    ]
  }
}
```
