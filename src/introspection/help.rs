//! Detailed help generation for all commands.
//!
//! [`CommandHelp::for_command`] returns human-readable and machine-readable
//! help for a given command, including usage, exit codes, error vocabulary,
//! and examples.
//!
//! The `description` field is derived from [`super::registry::CommandsList`] so
//! that it stays in sync with the single source of truth for command metadata.

use serde::{Deserialize, Serialize};

use super::registry::CommandsList;

/// A single exit-code descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodeInfo {
    pub code: i32,
    pub description: String,
}

/// A single error-code descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeInfo {
    pub code: String,
    pub description: String,
    #[serde(rename = "isRetryable")]
    pub is_retryable: bool,
}

/// A single usage example.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleInfo {
    pub description: String,
    pub command: String,
}

/// Full help record for a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHelp {
    pub command: String,
    pub description: String,
    pub usage: String,
    #[serde(rename = "exitCodes")]
    pub exit_codes: Vec<ExitCodeInfo>,
    #[serde(rename = "errorVocabulary")]
    pub error_vocabulary: Vec<ErrorCodeInfo>,
    pub examples: Vec<ExampleInfo>,
}

impl CommandHelp {
    /// Standard exit codes shared by every command.
    fn standard_exit_codes() -> Vec<ExitCodeInfo> {
        vec![
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
        ]
    }

    /// Standard error vocabulary shared by every command.
    fn standard_error_vocabulary() -> Vec<ErrorCodeInfo> {
        vec![
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
            ErrorCodeInfo {
                code: "INTERACTION_REQUIRED".to_string(),
                description: "User interaction required but --non-interactive mode is enabled"
                    .to_string(),
                is_retryable: false,
            },
        ]
    }

    /// Look up the description for `command` from the registry, falling back to
    /// a generic string if the command is not found.
    fn description_from_registry(command: &str) -> String {
        CommandsList::new()
            .commands
            .into_iter()
            .find(|c| c.name == command)
            .map(|c| c.description)
            .unwrap_or_else(|| format!("Help for {command}"))
    }

    /// Build the [`CommandHelp`] for the given command name.
    pub fn for_command(command: &str) -> Self {
        let exit_codes = Self::standard_exit_codes();
        let error_vocabulary = Self::standard_error_vocabulary();
        let description = Self::description_from_registry(command);

        match command {
            "commands" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs commands [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "List commands in JSON format".to_string(),
                        command: "xcom-rs commands --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "List commands in text format".to_string(),
                        command: "xcom-rs commands --output text".to_string(),
                    },
                ],
            },
            "schema" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs schema --command <name> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Get schema for commands command".to_string(),
                        command: "xcom-rs schema --command commands --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "Get schema for help command".to_string(),
                        command: "xcom-rs schema --command help --output json".to_string(),
                    },
                ],
            },
            "help" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs help <command> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Get help for commands command".to_string(),
                        command: "xcom-rs help commands --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "Get help for schema command".to_string(),
                        command: "xcom-rs help schema --output json".to_string(),
                    },
                ],
            },
            "demo-interactive" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs demo-interactive [--non-interactive] [--output json|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Run in interactive mode".to_string(),
                        command: "xcom-rs demo-interactive".to_string(),
                    },
                    ExampleInfo {
                        description:
                            "Run in non-interactive mode (will fail with INTERACTION_REQUIRED)"
                                .to_string(),
                        command: "xcom-rs demo-interactive --non-interactive --output json"
                            .to_string(),
                    },
                ],
            },
            "install-skills" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs install-skills [--skill <name>] [--agent <agent>] [--global] \
                        [--yes] [--output json|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Install all skills to project scope with JSON output"
                            .to_string(),
                        command: "xcom-rs install-skills --yes --non-interactive --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Install specific skill to global scope for Claude"
                            .to_string(),
                        command: "xcom-rs install-skills --skill example-skill --agent claude \
                                  --global --yes --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Install all skills to OpenCode project scope".to_string(),
                        command: "xcom-rs install-skills --agent opencode --yes --output json"
                            .to_string(),
                    },
                ],
            },
            "search recent" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs search recent \"<query>\" [--limit N] [--cursor <token>] \
                        [--output json|ndjson|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Search for recent tweets about Rust".to_string(),
                        command: "xcom-rs search recent \"rust programming\" --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Search with pagination limit".to_string(),
                        command: "xcom-rs search recent \"AI\" --limit 20 --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Paginate through search results".to_string(),
                        command: "xcom-rs search recent \"AI\" --cursor cursor_20 --output ndjson"
                            .to_string(),
                    },
                ],
            },
            "search users" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs search users \"<query>\" [--limit N] [--cursor <token>] \
                        [--output json|ndjson|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Search for users named Alice".to_string(),
                        command: "xcom-rs search users \"alice\" --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "Search users with limit".to_string(),
                        command: "xcom-rs search users \"developer\" --limit 5 --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Get users as NDJSON stream".to_string(),
                        command: "xcom-rs search users \"rust\" --output ndjson".to_string(),
                    },
                ],
            },
            "tweets reply" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets reply <tweet_id> \"<text>\" [--client-request-id <id>] \
                        [--if-exists return|error] [--output json|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Reply to a tweet".to_string(),
                        command: "xcom-rs tweets reply 1234567890 \"Great post!\" --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Reply with idempotency key".to_string(),
                        command: "xcom-rs tweets reply 1234567890 \"Reply\" \
                                  --client-request-id my-reply-001 --output json"
                            .to_string(),
                    },
                ],
            },
            "tweets thread" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets thread \"<t1>\" \"<t2>\" ... \
                        [--client-request-id-prefix <prefix>] [--if-exists return|error] \
                        [--output json|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Post a two-tweet thread".to_string(),
                        command:
                            "xcom-rs tweets thread \"First tweet\" \"Second tweet\" --output json"
                                .to_string(),
                    },
                    ExampleInfo {
                        description: "Post thread with idempotency prefix".to_string(),
                        command: "xcom-rs tweets thread \"A\" \"B\" \"C\" \
                                  --client-request-id-prefix thread-001 --output json"
                            .to_string(),
                    },
                ],
            },
            "tweets show" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets show <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Show a tweet by ID".to_string(),
                    command: "xcom-rs tweets show 1234567890 --output json".to_string(),
                }],
            },
            "tweets conversation" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets conversation <tweet_id> [--output json|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Fetch conversation tree for a tweet".to_string(),
                    command: "xcom-rs tweets conversation 1234567890 --output json".to_string(),
                }],
            },
            "timeline.home" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs timeline home [--limit <n>] [--cursor <token>] \
                        [--output json|ndjson|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Get home timeline in JSON format".to_string(),
                        command: "xcom-rs timeline home --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "Get 20 tweets with NDJSON output".to_string(),
                        command: "xcom-rs timeline home --limit 20 --output ndjson".to_string(),
                    },
                    ExampleInfo {
                        description: "Get next page using cursor".to_string(),
                        command: "xcom-rs timeline home --cursor next_token_10 --output json"
                            .to_string(),
                    },
                ],
            },
            "timeline.mentions" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs timeline mentions [--limit <n>] [--cursor <token>] \
                        [--output json|ndjson|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Get mentions in JSON format".to_string(),
                        command: "xcom-rs timeline mentions --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "Get 5 mentions with NDJSON output".to_string(),
                        command: "xcom-rs timeline mentions --limit 5 --output ndjson".to_string(),
                    },
                ],
            },
            "timeline.user" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs timeline user <handle> [--limit <n>] [--cursor <token>] \
                        [--output json|ndjson|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Get tweets from user in JSON format".to_string(),
                        command: "xcom-rs timeline user johndoe --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "Get 5 tweets from user with NDJSON output".to_string(),
                        command: "xcom-rs timeline user johndoe --limit 5 --output ndjson"
                            .to_string(),
                    },
                ],
            },
            "media.upload" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs media upload <path> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "Upload an image and get media_id in JSON format".to_string(),
                        command: "xcom-rs media upload /path/to/image.jpg --output json"
                            .to_string(),
                    },
                    ExampleInfo {
                        description: "Upload a video file".to_string(),
                        command: "xcom-rs media upload /path/to/video.mp4 --output json"
                            .to_string(),
                    },
                ],
            },
            "tweets like" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets like <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Like a specific tweet".to_string(),
                    command: "xcom-rs tweets like 1234567890 --output json".to_string(),
                }],
            },
            "tweets unlike" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets unlike <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Unlike a specific tweet".to_string(),
                    command: "xcom-rs tweets unlike 1234567890 --output json".to_string(),
                }],
            },
            "tweets retweet" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets retweet <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Retweet a specific tweet".to_string(),
                    command: "xcom-rs tweets retweet 1234567890 --output json".to_string(),
                }],
            },
            "tweets unretweet" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs tweets unretweet <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Unretweet a specific tweet".to_string(),
                    command: "xcom-rs tweets unretweet 1234567890 --output json".to_string(),
                }],
            },
            "bookmarks add" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs bookmarks add <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Bookmark a specific tweet".to_string(),
                    command: "xcom-rs bookmarks add 1234567890 --output json".to_string(),
                }],
            },
            "bookmarks remove" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs bookmarks remove <tweet_id> [--output json|yaml|text]".to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![ExampleInfo {
                    description: "Remove a specific tweet from bookmarks".to_string(),
                    command: "xcom-rs bookmarks remove 1234567890 --output json".to_string(),
                }],
            },
            "bookmarks list" => Self {
                command: command.to_string(),
                description,
                usage: "xcom-rs bookmarks list [--limit N] [--cursor <token>] \
                        [--output json|ndjson|yaml|text]"
                    .to_string(),
                exit_codes,
                error_vocabulary,
                examples: vec![
                    ExampleInfo {
                        description: "List bookmarks with JSON output".to_string(),
                        command: "xcom-rs bookmarks list --output json".to_string(),
                    },
                    ExampleInfo {
                        description: "List bookmarks with limit and cursor".to_string(),
                        command:
                            "xcom-rs bookmarks list --limit 20 --cursor <next_token> --output json"
                                .to_string(),
                    },
                    ExampleInfo {
                        description: "List bookmarks as NDJSON stream".to_string(),
                        command: "xcom-rs bookmarks list --output ndjson".to_string(),
                    },
                ],
            },
            _ => Self {
                command: command.to_string(),
                description,
                usage: format!("xcom-rs {} [options]", command),
                exit_codes,
                error_vocabulary,
                examples: vec![],
            },
        }
    }
}
