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

    /// Search operations
    Search {
        #[command(subcommand)]
        command: SearchCommands,
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

    /// Reply to a tweet
    Reply {
        /// ID of the tweet to reply to
        tweet_id: String,

        /// Reply text content
        text: String,

        /// Client request ID for idempotency (auto-generated if not provided)
        #[arg(long)]
        client_request_id: Option<String>,

        /// Policy when operation with same client_request_id exists
        #[arg(long, default_value = "return")]
        if_exists: String,
    },

    /// Post a thread of tweets (sequential replies)
    Thread {
        /// Tweet texts (at least one required; first is standalone, rest are replies)
        texts: Vec<String>,

        /// Prefix for generating per-tweet client_request_ids
        #[arg(long)]
        client_request_id_prefix: Option<String>,

        /// Policy when operation with same client_request_id exists
        #[arg(long, default_value = "return")]
        if_exists: String,
    },

    /// Show a single tweet by ID
    Show {
        /// Tweet ID to fetch
        tweet_id: String,
    },

    /// Retrieve a conversation tree starting from a tweet
    Conversation {
        /// Tweet ID (root of the conversation)
        tweet_id: String,
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

#[derive(Subcommand, Debug)]
pub enum SearchCommands {
    /// Search recent tweets matching a query
    Recent {
        /// Search query string
        query: String,

        /// Maximum number of results to return
        #[arg(long)]
        limit: Option<usize>,

        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Search users matching a query
    Users {
        /// Search query string
        query: String,

        /// Maximum number of results to return
        #[arg(long)]
        limit: Option<usize>,

        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },
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
    fn test_search_recent_command() {
        let cli = Cli::parse_from(["xcom-rs", "search", "recent", "hello world"]);
        if let Some(Commands::Search {
            command:
                SearchCommands::Recent {
                    query,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(query, "hello world");
            assert!(limit.is_none());
            assert!(cursor.is_none());
        } else {
            panic!("Expected Search Recent command");
        }
    }

    #[test]
    fn test_search_recent_with_limit() {
        let cli = Cli::parse_from(["xcom-rs", "search", "recent", "rust", "--limit", "20"]);
        if let Some(Commands::Search {
            command:
                SearchCommands::Recent {
                    query,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(query, "rust");
            assert_eq!(limit, Some(20));
            assert!(cursor.is_none());
        } else {
            panic!("Expected Search Recent command with limit");
        }
    }

    #[test]
    fn test_search_recent_with_cursor() {
        let cli = Cli::parse_from([
            "xcom-rs",
            "search",
            "recent",
            "rust",
            "--cursor",
            "cursor_10",
        ]);
        if let Some(Commands::Search {
            command:
                SearchCommands::Recent {
                    query,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(query, "rust");
            assert!(limit.is_none());
            assert_eq!(cursor, Some("cursor_10".to_string()));
        } else {
            panic!("Expected Search Recent command with cursor");
        }
    }

    #[test]
    fn test_search_users_command() {
        let cli = Cli::parse_from(["xcom-rs", "search", "users", "alice"]);
        if let Some(Commands::Search {
            command:
                SearchCommands::Users {
                    query,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(query, "alice");
            assert!(limit.is_none());
            assert!(cursor.is_none());
        } else {
            panic!("Expected Search Users command");
        }
    }

    #[test]
    fn test_search_users_with_limit_and_cursor() {
        let cli = Cli::parse_from([
            "xcom-rs", "search", "users", "bob", "--limit", "5", "--cursor", "cursor_5",
        ]);
        if let Some(Commands::Search {
            command:
                SearchCommands::Users {
                    query,
                    limit,
                    cursor,
                },
        }) = cli.command
        {
            assert_eq!(query, "bob");
            assert_eq!(limit, Some(5));
            assert_eq!(cursor, Some("cursor_5".to_string()));
        } else {
            panic!("Expected Search Users command with limit and cursor");
        }
    }

    #[test]
    fn test_tweets_reply_command() {
        let cli = Cli::parse_from(["xcom-rs", "tweets", "reply", "tweet_123", "Hello!"]);
        if let Some(Commands::Tweets {
            command:
                TweetsCommands::Reply {
                    tweet_id,
                    text,
                    client_request_id,
                    if_exists,
                },
        }) = cli.command
        {
            assert_eq!(tweet_id, "tweet_123");
            assert_eq!(text, "Hello!");
            assert!(client_request_id.is_none());
            assert_eq!(if_exists, "return");
        } else {
            panic!("Expected Tweets Reply command");
        }
    }

    #[test]
    fn test_tweets_reply_with_client_request_id() {
        let cli = Cli::parse_from([
            "xcom-rs",
            "tweets",
            "reply",
            "tweet_123",
            "Hello!",
            "--client-request-id",
            "my-reply-001",
        ]);
        if let Some(Commands::Tweets {
            command:
                TweetsCommands::Reply {
                    tweet_id,
                    text,
                    client_request_id,
                    if_exists,
                },
        }) = cli.command
        {
            assert_eq!(tweet_id, "tweet_123");
            assert_eq!(text, "Hello!");
            assert_eq!(client_request_id, Some("my-reply-001".to_string()));
            assert_eq!(if_exists, "return");
        } else {
            panic!("Expected Tweets Reply command with client_request_id");
        }
    }

    #[test]
    fn test_tweets_thread_command() {
        let cli = Cli::parse_from(["xcom-rs", "tweets", "thread", "First tweet", "Second tweet"]);
        if let Some(Commands::Tweets {
            command:
                TweetsCommands::Thread {
                    texts,
                    client_request_id_prefix,
                    if_exists,
                },
        }) = cli.command
        {
            assert_eq!(texts, vec!["First tweet", "Second tweet"]);
            assert!(client_request_id_prefix.is_none());
            assert_eq!(if_exists, "return");
        } else {
            panic!("Expected Tweets Thread command");
        }
    }

    #[test]
    fn test_tweets_thread_with_prefix() {
        let cli = Cli::parse_from([
            "xcom-rs",
            "tweets",
            "thread",
            "A",
            "B",
            "--client-request-id-prefix",
            "thread-001",
        ]);
        if let Some(Commands::Tweets {
            command:
                TweetsCommands::Thread {
                    texts,
                    client_request_id_prefix,
                    if_exists: _,
                },
        }) = cli.command
        {
            assert_eq!(texts, vec!["A", "B"]);
            assert_eq!(client_request_id_prefix, Some("thread-001".to_string()));
        } else {
            panic!("Expected Tweets Thread command with prefix");
        }
    }

    #[test]
    fn test_tweets_show_command() {
        let cli = Cli::parse_from(["xcom-rs", "tweets", "show", "tweet_999"]);
        if let Some(Commands::Tweets {
            command: TweetsCommands::Show { tweet_id },
        }) = cli.command
        {
            assert_eq!(tweet_id, "tweet_999");
        } else {
            panic!("Expected Tweets Show command");
        }
    }

    #[test]
    fn test_tweets_conversation_command() {
        let cli = Cli::parse_from(["xcom-rs", "tweets", "conversation", "tweet_root"]);
        if let Some(Commands::Tweets {
            command: TweetsCommands::Conversation { tweet_id },
        }) = cli.command
        {
            assert_eq!(tweet_id, "tweet_root");
        } else {
            panic!("Expected Tweets Conversation command");
        }
    }
}
