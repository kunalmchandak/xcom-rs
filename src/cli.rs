use clap::{Parser, Subcommand};

/// X.com CLI tool for agent-friendly interactions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_subcommand = true)]
pub struct Cli {
    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub output: String,

    /// Run in non-interactive mode (no prompts)
    #[arg(long, global = true)]
    pub non_interactive: bool,

    /// Trace ID for request correlation
    #[arg(long, global = true)]
    pub trace_id: Option<String>,

    /// Log format (json or text)
    #[arg(long, global = true, default_value = "text")]
    pub log_format: String,

    /// Maximum cost in credits for a single operation (fail if exceeded)
    #[arg(long, global = true)]
    pub max_cost_credits: Option<u32>,

    /// Daily budget in credits (fail if daily total would exceed)
    #[arg(long, global = true)]
    pub budget_daily_credits: Option<u32>,

    /// Dry run mode - estimate costs without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all available commands with metadata
    Commands,

    /// Get JSON schema for command input/output
    Schema {
        /// Command name to get schema for
        #[arg(long)]
        command: String,
    },

    /// Get detailed help for a command
    Help {
        /// Command name to get help for
        command: String,
    },

    /// Demo command that requires interaction (for testing non-interactive mode)
    DemoInteractive,

    /// Tweet operations
    Tweets {
        #[command(subcommand)]
        command: TweetsCommands,
    },

    /// Authentication commands
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    /// Billing commands
    Billing {
        #[command(subcommand)]
        command: BillingCommands,
    },

    /// Diagnostic information about configuration and runtime state
    Doctor,

    /// Install skills from embedded repository
    InstallSkills {
        /// Specific skill name to install (installs all if not specified)
        #[arg(long)]
        skill: Option<String>,

        /// Target agent (claude or opencode)
        #[arg(long)]
        agent: Option<String>,

        /// Install to global location instead of project
        #[arg(long)]
        global: bool,

        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
    },

    /// Timeline operations (home, mentions, user)
    Timeline {
        #[command(subcommand)]
        command: TimelineCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum TimelineCommands {
    /// Get home timeline (reverse chronological feed)
    Home {
        /// Maximum number of tweets to return
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Pagination cursor token
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get mentions timeline
    Mentions {
        /// Maximum number of tweets to return
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Pagination cursor token
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get tweets from a specific user
    User {
        /// User handle (without @)
        handle: String,

        /// Maximum number of tweets to return
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Pagination cursor token
        #[arg(long)]
        cursor: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TweetsCommands {
    /// Create a new tweet
    Create {
        /// Tweet text content
        text: String,

        /// Client request ID for idempotency (auto-generated if not provided)
        #[arg(long)]
        client_request_id: Option<String>,

        /// Policy when operation with same client_request_id exists
        #[arg(long, default_value = "return")]
        if_exists: String,
    },

    /// List tweets
    List {
        /// Fields to include in response (comma-separated: id,text,author_id,created_at)
        #[arg(long)]
        fields: Option<String>,

        /// Maximum number of tweets to return
        #[arg(long)]
        limit: Option<usize>,

        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Get current authentication status
    Status,

    /// Export authentication data
    Export,

    /// Import authentication data
    Import {
        /// Authentication data to import
        data: String,

        /// Dry run mode - show what would be changed without saving
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum BillingCommands {
    /// Estimate cost for an operation
    Estimate {
        /// Operation to estimate (e.g., "tweets.create")
        operation: String,

        /// Optional parameters (key=value format)
        #[arg(long)]
        text: Option<String>,
    },

    /// Get billing report
    Report,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(["xcom-rs", "commands"]);
        assert!(matches!(cli.command, Some(Commands::Commands)));
    }

    #[test]
    fn test_cli_with_output_format() {
        let cli = Cli::parse_from(["xcom-rs", "--output", "json", "commands"]);
        assert_eq!(cli.output, "json");
    }

    #[test]
    fn test_cli_with_trace_id() {
        let cli = Cli::parse_from(["xcom-rs", "--trace-id", "test-123", "commands"]);
        assert_eq!(cli.trace_id, Some("test-123".to_string()));
    }

    #[test]
    fn test_schema_command() {
        let cli = Cli::parse_from(["xcom-rs", "schema", "--command", "commands"]);
        if let Some(Commands::Schema { command, .. }) = cli.command {
            assert_eq!(command, "commands");
        } else {
            panic!("Expected Schema command");
        }
    }

    #[test]
    fn test_help_command() {
        let cli = Cli::parse_from(["xcom-rs", "help", "commands"]);
        if let Some(Commands::Help { command }) = cli.command {
            assert_eq!(command, "commands");
        } else {
            panic!("Expected Help command");
        }
    }

    #[test]
    fn test_cli_without_subcommand() {
        let cli = Cli::parse_from(["xcom-rs"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_timeline_home_command() {
        let cli = Cli::parse_from(["xcom-rs", "timeline", "home"]);
        if let Some(Commands::Timeline {
            command: TimelineCommands::Home { limit, cursor },
        }) = cli.command
        {
            assert_eq!(limit, 10);
            assert!(cursor.is_none());
        } else {
            panic!("Expected Timeline::Home command");
        }
    }

    #[test]
    fn test_timeline_home_with_limit() {
        let cli = Cli::parse_from(["xcom-rs", "timeline", "home", "--limit", "20"]);
        if let Some(Commands::Timeline {
            command: TimelineCommands::Home { limit, cursor },
        }) = cli.command
        {
            assert_eq!(limit, 20);
            assert!(cursor.is_none());
        } else {
            panic!("Expected Timeline::Home command with limit");
        }
    }

    #[test]
    fn test_timeline_home_with_cursor() {
        let cli = Cli::parse_from(["xcom-rs", "timeline", "home", "--cursor", "next_token_123"]);
        if let Some(Commands::Timeline {
            command: TimelineCommands::Home { limit: _, cursor },
        }) = cli.command
        {
            assert_eq!(cursor, Some("next_token_123".to_string()));
        } else {
            panic!("Expected Timeline::Home command with cursor");
        }
    }

    #[test]
    fn test_timeline_mentions_command() {
        let cli = Cli::parse_from(["xcom-rs", "timeline", "mentions"]);
        if let Some(Commands::Timeline {
            command: TimelineCommands::Mentions { limit, cursor },
        }) = cli.command
        {
            assert_eq!(limit, 10);
            assert!(cursor.is_none());
        } else {
            panic!("Expected Timeline::Mentions command");
        }
    }

    #[test]
    fn test_timeline_user_command() {
        let cli = Cli::parse_from(["xcom-rs", "timeline", "user", "johndoe"]);
        if let Some(Commands::Timeline {
            command:
                TimelineCommands::User {
                    handle,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(handle, "johndoe");
            assert_eq!(limit, 10);
            assert!(cursor.is_none());
        } else {
            panic!("Expected Timeline::User command");
        }
    }

    #[test]
    fn test_timeline_user_with_options() {
        let cli = Cli::parse_from([
            "xcom-rs",
            "timeline",
            "user",
            "johndoe",
            "--limit",
            "5",
            "--cursor",
            "cursor_abc",
        ]);
        if let Some(Commands::Timeline {
            command:
                TimelineCommands::User {
                    handle,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(handle, "johndoe");
            assert_eq!(limit, 5);
            assert_eq!(cursor, Some("cursor_abc".to_string()));
        } else {
            panic!("Expected Timeline::User command with options");
        }
    }
}
