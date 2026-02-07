use serde::{Deserialize, Serialize};

/// Command metadata for introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub arguments: Vec<ArgumentInfo>,
    pub risk: RiskLevel,
    #[serde(rename = "hasCost")]
    pub has_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentInfo {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(rename = "type")]
    pub arg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

/// List of all available commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsList {
    pub commands: Vec<CommandInfo>,
}

impl CommandsList {
    pub fn new() -> Self {
        Self {
            commands: vec![
                CommandInfo {
                    name: "commands".to_string(),
                    description: "List all available commands with metadata".to_string(),
                    arguments: vec![],
                    risk: RiskLevel::Safe,
                    has_cost: false,
                },
                CommandInfo {
                    name: "schema".to_string(),
                    description: "Get JSON schema for command input/output".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "command".to_string(),
                        description: "Command name to get schema for".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Safe,
                    has_cost: false,
                },
                CommandInfo {
                    name: "help".to_string(),
                    description: "Get detailed help for a command including exit codes".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "command".to_string(),
                        description: "Command name to get help for".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Safe,
                    has_cost: false,
                },
            ],
        }
    }
}

impl Default for CommandsList {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON Schema for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSchema {
    pub command: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(rename = "outputSchema")]
    pub output_schema: serde_json::Value,
}

impl CommandSchema {
    pub fn for_command(command: &str) -> Self {
        match command {
            "commands" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
                output_schema: serde_json::json!({
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
                                    "arguments": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "required": ["name", "description", "required", "type"],
                                            "properties": {
                                                "name": { "type": "string" },
                                                "description": { "type": "string" },
                                                "required": { "type": "boolean" },
                                                "type": { "type": "string" },
                                                "default": { "type": "string" }
                                            }
                                        }
                                    },
                                    "risk": {
                                        "type": "string",
                                        "enum": ["safe", "low", "medium", "high"]
                                    },
                                    "hasCost": { "type": "boolean" }
                                }
                            }
                        }
                    }
                }),
            },
            "schema" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["command"],
                    "properties": {
                        "command": { "type": "string" }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "required": ["command", "inputSchema", "outputSchema"],
                    "properties": {
                        "command": { "type": "string" },
                        "inputSchema": { "type": "object" },
                        "outputSchema": { "type": "object" }
                    }
                }),
            },
            "help" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["command"],
                    "properties": {
                        "command": { "type": "string" }
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "required": ["command", "description", "usage", "exitCodes"],
                    "properties": {
                        "command": { "type": "string" },
                        "description": { "type": "string" },
                        "usage": { "type": "string" },
                        "exitCodes": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["code", "description"],
                                "properties": {
                                    "code": { "type": "integer" },
                                    "description": { "type": "string" }
                                }
                            }
                        },
                        "errors": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["code", "description", "isRetryable"],
                                "properties": {
                                    "code": { "type": "string" },
                                    "description": { "type": "string" },
                                    "isRetryable": { "type": "boolean" }
                                }
                            }
                        }
                    }
                }),
            },
            _ => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        }
    }
}

/// Exit code information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodeInfo {
    pub code: i32,
    pub description: String,
}

/// Error code information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeInfo {
    pub code: String,
    pub description: String,
    #[serde(rename = "isRetryable")]
    pub is_retryable: bool,
}

/// Detailed help for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHelp {
    pub command: String,
    pub description: String,
    pub usage: String,
    #[serde(rename = "exitCodes")]
    pub exit_codes: Vec<ExitCodeInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorCodeInfo>>,
}

impl CommandHelp {
    pub fn for_command(command: &str) -> Self {
        let exit_codes = vec![
            ExitCodeInfo {
                code: 0,
                description: "Success".to_string(),
            },
            ExitCodeInfo {
                code: 2,
                description: "Invalid argument or missing required argument".to_string(),
            },
            ExitCodeInfo {
                code: 3,
                description: "Authentication or authorization failed".to_string(),
            },
            ExitCodeInfo {
                code: 4,
                description: "Operation failed (network, rate limit, service unavailable, etc.)"
                    .to_string(),
            },
        ];

        let errors = Some(vec![
            ErrorCodeInfo {
                code: "INVALID_ARGUMENT".to_string(),
                description: "Invalid argument provided".to_string(),
                is_retryable: false,
            },
            ErrorCodeInfo {
                code: "MISSING_ARGUMENT".to_string(),
                description: "Required argument missing".to_string(),
                is_retryable: false,
            },
            ErrorCodeInfo {
                code: "UNKNOWN_COMMAND".to_string(),
                description: "Command not recognized".to_string(),
                is_retryable: false,
            },
            ErrorCodeInfo {
                code: "AUTHENTICATION_FAILED".to_string(),
                description: "Authentication credentials invalid or expired".to_string(),
                is_retryable: false,
            },
            ErrorCodeInfo {
                code: "AUTHORIZATION_FAILED".to_string(),
                description: "Insufficient permissions".to_string(),
                is_retryable: false,
            },
            ErrorCodeInfo {
                code: "RATE_LIMIT_EXCEEDED".to_string(),
                description: "Rate limit exceeded, retry after delay".to_string(),
                is_retryable: true,
            },
            ErrorCodeInfo {
                code: "NETWORK_ERROR".to_string(),
                description: "Network connection failed".to_string(),
                is_retryable: true,
            },
            ErrorCodeInfo {
                code: "SERVICE_UNAVAILABLE".to_string(),
                description: "Service temporarily unavailable".to_string(),
                is_retryable: true,
            },
            ErrorCodeInfo {
                code: "INTERNAL_ERROR".to_string(),
                description: "Internal error occurred".to_string(),
                is_retryable: false,
            },
        ]);

        match command {
            "commands" => Self {
                command: command.to_string(),
                description: "List all available commands with metadata".to_string(),
                usage: "xcom-rs commands [--output json|yaml|text]".to_string(),
                exit_codes,
                errors,
            },
            "schema" => Self {
                command: command.to_string(),
                description: "Get JSON schema for command input/output".to_string(),
                usage: "xcom-rs schema --command <name> [--output json|yaml|text]".to_string(),
                exit_codes,
                errors,
            },
            "help" => Self {
                command: command.to_string(),
                description: "Get detailed help for a command including exit codes".to_string(),
                usage: "xcom-rs help <command> [--output json|yaml|text]".to_string(),
                exit_codes,
                errors,
            },
            _ => Self {
                command: command.to_string(),
                description: format!("Help for {}", command),
                usage: format!("xcom-rs {} [options]", command),
                exit_codes,
                errors,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands_list() {
        let list = CommandsList::new();
        assert!(!list.commands.is_empty());
        assert!(list.commands.iter().any(|c| c.name == "commands"));
        assert!(list.commands.iter().any(|c| c.name == "schema"));
        assert!(list.commands.iter().any(|c| c.name == "help"));
    }

    #[test]
    fn test_command_schema() {
        let schema = CommandSchema::for_command("commands");
        assert_eq!(schema.command, "commands");
        assert!(schema.input_schema.is_object());
        assert!(schema.output_schema.is_object());
    }

    #[test]
    fn test_command_help() {
        let help = CommandHelp::for_command("commands");
        assert_eq!(help.command, "commands");
        assert!(!help.exit_codes.is_empty());
        assert!(help.errors.is_some());
    }
}
