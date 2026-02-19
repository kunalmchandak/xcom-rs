//! Command registry: single source of truth for all command metadata.
//!
//! Every list/schema/help generator reads from the `CommandsList` defined here,
//! so adding or removing a command only requires one edit.

use serde::{Deserialize, Serialize};

/// Risk level associated with a command.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

/// Metadata for a single command argument.
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

/// Metadata for a single command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub arguments: Vec<ArgumentInfo>,
    pub risk: RiskLevel,
    #[serde(rename = "hasCost")]
    pub has_cost: bool,
}

/// The complete list of all available commands.
///
/// This is the **single registration point** for command metadata. All
/// introspection outputs (`commands`, `schema`, `help`) derive their data
/// from this list.
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
                CommandInfo {
                    name: "demo-interactive".to_string(),
                    description:
                        "Demo command that requires interaction (for testing non-interactive mode)"
                            .to_string(),
                    arguments: vec![],
                    risk: RiskLevel::Low,
                    has_cost: false,
                },
                CommandInfo {
                    name: "install-skills".to_string(),
                    description: "Install skills from embedded repository".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "skill".to_string(),
                            description:
                                "Specific skill name to install (installs all if not specified)"
                                    .to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "agent".to_string(),
                            description: "Target agent (claude or opencode)".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "global".to_string(),
                            description: "Install to global location instead of project"
                                .to_string(),
                            required: false,
                            arg_type: "boolean".to_string(),
                            default: Some("false".to_string()),
                        },
                        ArgumentInfo {
                            name: "yes".to_string(),
                            description: "Skip confirmation prompts".to_string(),
                            required: false,
                            arg_type: "boolean".to_string(),
                            default: Some("false".to_string()),
                        },
                    ],
                    risk: RiskLevel::Low,
                    has_cost: false,
                },
                CommandInfo {
                    name: "search recent".to_string(),
                    description: "Search recent tweets matching a query".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "query".to_string(),
                            description: "Search query string".to_string(),
                            required: true,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "limit".to_string(),
                            description: "Maximum number of results to return".to_string(),
                            required: false,
                            arg_type: "integer".to_string(),
                            default: Some("10".to_string()),
                        },
                        ArgumentInfo {
                            name: "cursor".to_string(),
                            description: "Pagination cursor".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                    ],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "search users".to_string(),
                    description: "Search users matching a query".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "query".to_string(),
                            description: "Search query string".to_string(),
                            required: true,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "limit".to_string(),
                            description: "Maximum number of results to return".to_string(),
                            required: false,
                            arg_type: "integer".to_string(),
                            default: Some("10".to_string()),
                        },
                        ArgumentInfo {
                            name: "cursor".to_string(),
                            description: "Pagination cursor".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                    ],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets reply".to_string(),
                    description: "Reply to a tweet".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "tweet_id".to_string(),
                            description: "ID of the tweet to reply to".to_string(),
                            required: true,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "text".to_string(),
                            description: "Reply text content".to_string(),
                            required: true,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "client_request_id".to_string(),
                            description:
                                "Client request ID for idempotency (auto-generated if not provided)"
                                    .to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "if_exists".to_string(),
                            description: "Policy when operation with same client_request_id exists"
                                .to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: Some("return".to_string()),
                        },
                    ],
                    risk: RiskLevel::Medium,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets thread".to_string(),
                    description: "Post a thread of tweets (sequential replies)".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "texts".to_string(),
                            description:
                                "Tweet texts (at least one; first is standalone, rest are replies)"
                                    .to_string(),
                            required: true,
                            arg_type: "array<string>".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "client_request_id_prefix".to_string(),
                            description: "Prefix for generating per-tweet client_request_ids"
                                .to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "if_exists".to_string(),
                            description: "Policy when operation with same client_request_id exists"
                                .to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: Some("return".to_string()),
                        },
                    ],
                    risk: RiskLevel::Medium,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets show".to_string(),
                    description: "Show a single tweet by ID".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to fetch".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets conversation".to_string(),
                    description: "Retrieve a conversation tree starting from a tweet".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID (root of the conversation)".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "timeline.home".to_string(),
                    description: "Get home timeline (reverse chronological feed)".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "limit".to_string(),
                            description: "Maximum number of tweets to return".to_string(),
                            required: false,
                            arg_type: "integer".to_string(),
                            default: Some("10".to_string()),
                        },
                        ArgumentInfo {
                            name: "cursor".to_string(),
                            description: "Pagination cursor token".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                    ],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "timeline.mentions".to_string(),
                    description: "Get mentions timeline".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "limit".to_string(),
                            description: "Maximum number of tweets to return".to_string(),
                            required: false,
                            arg_type: "integer".to_string(),
                            default: Some("10".to_string()),
                        },
                        ArgumentInfo {
                            name: "cursor".to_string(),
                            description: "Pagination cursor token".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                    ],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "timeline.user".to_string(),
                    description: "Get tweets from a specific user".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "handle".to_string(),
                            description: "User handle (without @)".to_string(),
                            required: true,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                        ArgumentInfo {
                            name: "limit".to_string(),
                            description: "Maximum number of tweets to return".to_string(),
                            required: false,
                            arg_type: "integer".to_string(),
                            default: Some("10".to_string()),
                        },
                        ArgumentInfo {
                            name: "cursor".to_string(),
                            description: "Pagination cursor token".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                    ],
                    risk: RiskLevel::Safe,
                    has_cost: true,
                },
                CommandInfo {
                    name: "media.upload".to_string(),
                    description: "Upload a media file and return the media_id".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "path".to_string(),
                        description: "Path to the media file to upload".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Medium,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets like".to_string(),
                    description: "Like a tweet".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to like".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Low,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets unlike".to_string(),
                    description: "Unlike a tweet".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to unlike".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Low,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets retweet".to_string(),
                    description: "Retweet a tweet".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to retweet".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Medium,
                    has_cost: true,
                },
                CommandInfo {
                    name: "tweets unretweet".to_string(),
                    description: "Undo a retweet".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to unretweet".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Low,
                    has_cost: true,
                },
                CommandInfo {
                    name: "bookmarks add".to_string(),
                    description: "Add a tweet to bookmarks".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to bookmark".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Low,
                    has_cost: true,
                },
                CommandInfo {
                    name: "bookmarks remove".to_string(),
                    description: "Remove a tweet from bookmarks".to_string(),
                    arguments: vec![ArgumentInfo {
                        name: "tweet_id".to_string(),
                        description: "Tweet ID to remove from bookmarks".to_string(),
                        required: true,
                        arg_type: "string".to_string(),
                        default: None,
                    }],
                    risk: RiskLevel::Low,
                    has_cost: true,
                },
                CommandInfo {
                    name: "bookmarks list".to_string(),
                    description: "List bookmarked tweets".to_string(),
                    arguments: vec![
                        ArgumentInfo {
                            name: "limit".to_string(),
                            description: "Maximum number of bookmarks to return".to_string(),
                            required: false,
                            arg_type: "integer".to_string(),
                            default: Some("10".to_string()),
                        },
                        ArgumentInfo {
                            name: "cursor".to_string(),
                            description: "Pagination cursor".to_string(),
                            required: false,
                            arg_type: "string".to_string(),
                            default: None,
                        },
                    ],
                    risk: RiskLevel::Safe,
                    has_cost: true,
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
