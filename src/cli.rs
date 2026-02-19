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

    /// Bookmark operations
    Bookmarks {
        #[command(subcommand)]
        command: BookmarksCommands,
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
    Doctor {
        /// Perform an API connectivity probe (requires network access)
        #[arg(long)]
        probe: bool,
    },

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

    /// Timeline operations (home, mentions, user)
    Timeline {
        #[command(subcommand)]
        command: TimelineCommands,
    },

    /// Media operations
    Media {
        #[command(subcommand)]
        command: MediaCommands,
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

    /// Like a tweet
    Like {
        /// Tweet ID to like
        tweet_id: String,
    },

    /// Unlike a tweet
    Unlike {
        /// Tweet ID to unlike
        tweet_id: String,
    },

    /// Retweet a tweet
    Retweet {
        /// Tweet ID to retweet
        tweet_id: String,
    },

    /// Undo a retweet
    Unretweet {
        /// Tweet ID to unretweet
        tweet_id: String,
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
pub enum BookmarksCommands {
    /// Add a tweet to bookmarks
    Add {
        /// Tweet ID to bookmark
        tweet_id: String,
    },

    /// Remove a tweet from bookmarks
    Remove {
        /// Tweet ID to remove from bookmarks
        tweet_id: String,
    },

    /// List bookmarked tweets
    List {
        /// Maximum number of bookmarks to return
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

#[derive(Subcommand, Debug)]
pub enum MediaCommands {
    /// Upload a media file and return the media_id
    Upload {
        /// Path to the media file to upload
        path: String,
    },
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

    // ---------------------------------------------------------------------------
    // Helper: parse CLI args and return the parsed Cli struct.
    // Accepts an iterator of string-like values.
    // ---------------------------------------------------------------------------
    fn parse<I, S>(args: I) -> Cli
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString> + Clone,
    {
        Cli::parse_from(args)
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: global flags
    // ---------------------------------------------------------------------------

    #[test]
    fn test_global_flags() {
        struct Case {
            args: Vec<&'static str>,
            output: &'static str,
            non_interactive: bool,
            trace_id: Option<&'static str>,
            log_format: &'static str,
            max_cost_credits: Option<u32>,
            budget_daily_credits: Option<u32>,
            dry_run: bool,
        }

        let cases = vec![
            Case {
                args: vec!["xcom-rs", "commands"],
                output: "text",
                non_interactive: false,
                trace_id: None,
                log_format: "text",
                max_cost_credits: None,
                budget_daily_credits: None,
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--output", "json", "commands"],
                output: "json",
                non_interactive: false,
                trace_id: None,
                log_format: "text",
                max_cost_credits: None,
                budget_daily_credits: None,
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--trace-id", "test-123", "commands"],
                output: "text",
                non_interactive: false,
                trace_id: Some("test-123"),
                log_format: "text",
                max_cost_credits: None,
                budget_daily_credits: None,
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--non-interactive", "commands"],
                output: "text",
                non_interactive: true,
                trace_id: None,
                log_format: "text",
                max_cost_credits: None,
                budget_daily_credits: None,
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--log-format", "json", "commands"],
                output: "text",
                non_interactive: false,
                trace_id: None,
                log_format: "json",
                max_cost_credits: None,
                budget_daily_credits: None,
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--max-cost-credits", "100", "commands"],
                output: "text",
                non_interactive: false,
                trace_id: None,
                log_format: "text",
                max_cost_credits: Some(100),
                budget_daily_credits: None,
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--budget-daily-credits", "500", "commands"],
                output: "text",
                non_interactive: false,
                trace_id: None,
                log_format: "text",
                max_cost_credits: None,
                budget_daily_credits: Some(500),
                dry_run: false,
            },
            Case {
                args: vec!["xcom-rs", "--dry-run", "commands"],
                output: "text",
                non_interactive: false,
                trace_id: None,
                log_format: "text",
                max_cost_credits: None,
                budget_daily_credits: None,
                dry_run: true,
            },
        ];

        for case in &cases {
            let cli = parse(case.args.iter().copied());
            assert_eq!(cli.output, case.output, "args={:?}", case.args);
            assert_eq!(
                cli.non_interactive, case.non_interactive,
                "args={:?}",
                case.args
            );
            assert_eq!(
                cli.trace_id,
                case.trace_id.map(str::to_owned),
                "args={:?}",
                case.args
            );
            assert_eq!(cli.log_format, case.log_format, "args={:?}", case.args);
            assert_eq!(
                cli.max_cost_credits, case.max_cost_credits,
                "args={:?}",
                case.args
            );
            assert_eq!(
                cli.budget_daily_credits, case.budget_daily_credits,
                "args={:?}",
                case.args
            );
            assert_eq!(cli.dry_run, case.dry_run, "args={:?}", case.args);
        }
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: top-level commands (no subcommand)
    // ---------------------------------------------------------------------------

    #[test]
    fn test_top_level_commands() {
        // (args, expected_matches_fn)
        type Matcher = fn(Option<Commands>) -> bool;
        let cases: Vec<(Vec<&str>, Matcher)> = vec![
            (vec!["xcom-rs"], |cmd| cmd.is_none()),
            (vec!["xcom-rs", "commands"], |cmd| {
                matches!(cmd, Some(Commands::Commands))
            }),
            (vec!["xcom-rs", "doctor"], |cmd| {
                matches!(cmd, Some(Commands::Doctor { probe: false }))
            }),
            (vec!["xcom-rs", "doctor", "--probe"], |cmd| {
                matches!(cmd, Some(Commands::Doctor { probe: true }))
            }),
            (vec!["xcom-rs", "demo-interactive"], |cmd| {
                matches!(cmd, Some(Commands::DemoInteractive))
            }),
        ];

        for (args, matcher) in cases {
            let cli = parse(args.iter().copied());
            assert!(matcher(cli.command), "args={args:?}");
        }
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: schema / help commands
    // ---------------------------------------------------------------------------

    #[test]
    fn test_schema_and_help_commands() {
        // schema --command <name>
        let cli = parse(["xcom-rs", "schema", "--command", "commands"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Schema { command }) if command == "commands"
        ));

        // help <name>
        let cli = parse(["xcom-rs", "help", "commands"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Help { command }) if command == "commands"
        ));
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: tweets subcommands
    // ---------------------------------------------------------------------------

    #[test]
    fn test_tweets_subcommands() {
        // Each row: (args, assertion closure)
        type TweetsAssert = fn(TweetsCommands);

        let cases: Vec<(Vec<&str>, TweetsAssert)> = vec![
            // tweets create
            (vec!["xcom-rs", "tweets", "create", "Hello world"], |cmd| {
                let TweetsCommands::Create {
                    text,
                    client_request_id,
                    if_exists,
                } = cmd
                else {
                    panic!("Expected Create");
                };
                assert_eq!(text, "Hello world");
                assert!(client_request_id.is_none());
                assert_eq!(if_exists, "return");
            }),
            // tweets like
            (vec!["xcom-rs", "tweets", "like", "tweet123"], |cmd| {
                let TweetsCommands::Like { tweet_id } = cmd else {
                    panic!("Expected Like");
                };
                assert_eq!(tweet_id, "tweet123");
            }),
            // tweets unlike
            (vec!["xcom-rs", "tweets", "unlike", "tweet123"], |cmd| {
                let TweetsCommands::Unlike { tweet_id } = cmd else {
                    panic!("Expected Unlike");
                };
                assert_eq!(tweet_id, "tweet123");
            }),
            // tweets retweet
            (vec!["xcom-rs", "tweets", "retweet", "tweet123"], |cmd| {
                let TweetsCommands::Retweet { tweet_id } = cmd else {
                    panic!("Expected Retweet");
                };
                assert_eq!(tweet_id, "tweet123");
            }),
            // tweets unretweet
            (vec!["xcom-rs", "tweets", "unretweet", "tweet123"], |cmd| {
                let TweetsCommands::Unretweet { tweet_id } = cmd else {
                    panic!("Expected Unretweet");
                };
                assert_eq!(tweet_id, "tweet123");
            }),
            // tweets show
            (vec!["xcom-rs", "tweets", "show", "tweet_999"], |cmd| {
                let TweetsCommands::Show { tweet_id } = cmd else {
                    panic!("Expected Show");
                };
                assert_eq!(tweet_id, "tweet_999");
            }),
            // tweets conversation
            (
                vec!["xcom-rs", "tweets", "conversation", "tweet_root"],
                |cmd| {
                    let TweetsCommands::Conversation { tweet_id } = cmd else {
                        panic!("Expected Conversation");
                    };
                    assert_eq!(tweet_id, "tweet_root");
                },
            ),
            // tweets reply (no client_request_id)
            (
                vec!["xcom-rs", "tweets", "reply", "tweet_123", "Hello!"],
                |cmd| {
                    let TweetsCommands::Reply {
                        tweet_id,
                        text,
                        client_request_id,
                        if_exists,
                    } = cmd
                    else {
                        panic!("Expected Reply");
                    };
                    assert_eq!(tweet_id, "tweet_123");
                    assert_eq!(text, "Hello!");
                    assert!(client_request_id.is_none());
                    assert_eq!(if_exists, "return");
                },
            ),
            // tweets reply (with client_request_id)
            (
                vec![
                    "xcom-rs",
                    "tweets",
                    "reply",
                    "tweet_123",
                    "Hello!",
                    "--client-request-id",
                    "my-reply-001",
                ],
                |cmd| {
                    let TweetsCommands::Reply {
                        client_request_id, ..
                    } = cmd
                    else {
                        panic!("Expected Reply");
                    };
                    assert_eq!(client_request_id, Some("my-reply-001".to_string()));
                },
            ),
            // tweets thread (no prefix)
            (
                vec!["xcom-rs", "tweets", "thread", "First tweet", "Second tweet"],
                |cmd| {
                    let TweetsCommands::Thread {
                        texts,
                        client_request_id_prefix,
                        if_exists,
                    } = cmd
                    else {
                        panic!("Expected Thread");
                    };
                    assert_eq!(texts, vec!["First tweet", "Second tweet"]);
                    assert!(client_request_id_prefix.is_none());
                    assert_eq!(if_exists, "return");
                },
            ),
            // tweets thread (with prefix)
            (
                vec![
                    "xcom-rs",
                    "tweets",
                    "thread",
                    "A",
                    "B",
                    "--client-request-id-prefix",
                    "thread-001",
                ],
                |cmd| {
                    let TweetsCommands::Thread {
                        texts,
                        client_request_id_prefix,
                        ..
                    } = cmd
                    else {
                        panic!("Expected Thread");
                    };
                    assert_eq!(texts, vec!["A", "B"]);
                    assert_eq!(client_request_id_prefix, Some("thread-001".to_string()));
                },
            ),
        ];

        for (args, assert_fn) in cases {
            let cli = parse(args.iter().copied());
            let Some(Commands::Tweets { command }) = cli.command else {
                panic!("Expected Tweets command for args={args:?}");
            };
            assert_fn(command);
        }
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: bookmarks subcommands
    // ---------------------------------------------------------------------------

    #[test]
    fn test_bookmarks_subcommands() {
        type BookmarksAssert = fn(BookmarksCommands);

        let cases: Vec<(Vec<&str>, BookmarksAssert)> = vec![
            // bookmarks add
            (vec!["xcom-rs", "bookmarks", "add", "tweet123"], |cmd| {
                let BookmarksCommands::Add { tweet_id } = cmd else {
                    panic!("Expected Add");
                };
                assert_eq!(tweet_id, "tweet123");
            }),
            // bookmarks remove
            (vec!["xcom-rs", "bookmarks", "remove", "tweet123"], |cmd| {
                let BookmarksCommands::Remove { tweet_id } = cmd else {
                    panic!("Expected Remove");
                };
                assert_eq!(tweet_id, "tweet123");
            }),
            // bookmarks list (with limit only)
            (
                vec!["xcom-rs", "bookmarks", "list", "--limit", "10"],
                |cmd| {
                    let BookmarksCommands::List { limit, cursor } = cmd else {
                        panic!("Expected List");
                    };
                    assert_eq!(limit, Some(10));
                    assert!(cursor.is_none());
                },
            ),
            // bookmarks list (with limit and cursor)
            (
                vec![
                    "xcom-rs",
                    "bookmarks",
                    "list",
                    "--limit",
                    "5",
                    "--cursor",
                    "next_page_token",
                ],
                |cmd| {
                    let BookmarksCommands::List { limit, cursor } = cmd else {
                        panic!("Expected List");
                    };
                    assert_eq!(limit, Some(5));
                    assert_eq!(cursor, Some("next_page_token".to_string()));
                },
            ),
        ];

        for (args, assert_fn) in cases {
            let cli = parse(args.iter().copied());
            let Some(Commands::Bookmarks { command }) = cli.command else {
                panic!("Expected Bookmarks command for args={args:?}");
            };
            assert_fn(command);
        }
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: timeline subcommands
    // ---------------------------------------------------------------------------

    #[test]
    fn test_timeline_subcommands() {
        type TimelineAssert = fn(TimelineCommands);

        let cases: Vec<(Vec<&str>, TimelineAssert)> = vec![
            // timeline home (defaults)
            (vec!["xcom-rs", "timeline", "home"], |cmd| {
                let TimelineCommands::Home { limit, cursor } = cmd else {
                    panic!("Expected Home");
                };
                assert_eq!(limit, 10);
                assert!(cursor.is_none());
            }),
            // timeline home (custom limit)
            (
                vec!["xcom-rs", "timeline", "home", "--limit", "20"],
                |cmd| {
                    let TimelineCommands::Home { limit, cursor } = cmd else {
                        panic!("Expected Home");
                    };
                    assert_eq!(limit, 20);
                    assert!(cursor.is_none());
                },
            ),
            // timeline home (with cursor)
            (
                vec!["xcom-rs", "timeline", "home", "--cursor", "next_token_123"],
                |cmd| {
                    let TimelineCommands::Home { cursor, .. } = cmd else {
                        panic!("Expected Home");
                    };
                    assert_eq!(cursor, Some("next_token_123".to_string()));
                },
            ),
            // timeline mentions (defaults)
            (vec!["xcom-rs", "timeline", "mentions"], |cmd| {
                let TimelineCommands::Mentions { limit, cursor } = cmd else {
                    panic!("Expected Mentions");
                };
                assert_eq!(limit, 10);
                assert!(cursor.is_none());
            }),
            // timeline user (defaults)
            (vec!["xcom-rs", "timeline", "user", "johndoe"], |cmd| {
                let TimelineCommands::User {
                    handle,
                    limit,
                    cursor,
                } = cmd
                else {
                    panic!("Expected User");
                };
                assert_eq!(handle, "johndoe");
                assert_eq!(limit, 10);
                assert!(cursor.is_none());
            }),
            // timeline user (with limit and cursor)
            (
                vec![
                    "xcom-rs",
                    "timeline",
                    "user",
                    "johndoe",
                    "--limit",
                    "5",
                    "--cursor",
                    "cursor_abc",
                ],
                |cmd| {
                    let TimelineCommands::User {
                        handle,
                        limit,
                        cursor,
                    } = cmd
                    else {
                        panic!("Expected User");
                    };
                    assert_eq!(handle, "johndoe");
                    assert_eq!(limit, 5);
                    assert_eq!(cursor, Some("cursor_abc".to_string()));
                },
            ),
        ];

        for (args, assert_fn) in cases {
            let cli = parse(args.iter().copied());
            let Some(Commands::Timeline { command }) = cli.command else {
                panic!("Expected Timeline command for args={args:?}");
            };
            assert_fn(command);
        }
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: search subcommands
    // ---------------------------------------------------------------------------

    #[test]
    fn test_search_subcommands() {
        type SearchAssert = fn(SearchCommands);

        let cases: Vec<(Vec<&str>, SearchAssert)> = vec![
            // search recent (no options)
            (vec!["xcom-rs", "search", "recent", "hello world"], |cmd| {
                let SearchCommands::Recent {
                    query,
                    limit,
                    cursor,
                } = cmd
                else {
                    panic!("Expected Recent");
                };
                assert_eq!(query, "hello world");
                assert!(limit.is_none());
                assert!(cursor.is_none());
            }),
            // search recent (with limit)
            (
                vec!["xcom-rs", "search", "recent", "rust", "--limit", "20"],
                |cmd| {
                    let SearchCommands::Recent { query, limit, .. } = cmd else {
                        panic!("Expected Recent");
                    };
                    assert_eq!(query, "rust");
                    assert_eq!(limit, Some(20));
                },
            ),
            // search recent (with cursor)
            (
                vec![
                    "xcom-rs",
                    "search",
                    "recent",
                    "rust",
                    "--cursor",
                    "cursor_10",
                ],
                |cmd| {
                    let SearchCommands::Recent { cursor, .. } = cmd else {
                        panic!("Expected Recent");
                    };
                    assert_eq!(cursor, Some("cursor_10".to_string()));
                },
            ),
            // search users (no options)
            (vec!["xcom-rs", "search", "users", "alice"], |cmd| {
                let SearchCommands::Users {
                    query,
                    limit,
                    cursor,
                } = cmd
                else {
                    panic!("Expected Users");
                };
                assert_eq!(query, "alice");
                assert!(limit.is_none());
                assert!(cursor.is_none());
            }),
            // search users (with limit and cursor)
            (
                vec![
                    "xcom-rs", "search", "users", "bob", "--limit", "5", "--cursor", "cursor_5",
                ],
                |cmd| {
                    let SearchCommands::Users {
                        query,
                        limit,
                        cursor,
                    } = cmd
                    else {
                        panic!("Expected Users");
                    };
                    assert_eq!(query, "bob");
                    assert_eq!(limit, Some(5));
                    assert_eq!(cursor, Some("cursor_5".to_string()));
                },
            ),
        ];

        for (args, assert_fn) in cases {
            let cli = parse(args.iter().copied());
            let Some(Commands::Search { command }) = cli.command else {
                panic!("Expected Search command for args={args:?}");
            };
            assert_fn(command);
        }
    }

    // ---------------------------------------------------------------------------
    // Table-driven tests: media subcommands
    // ---------------------------------------------------------------------------

    #[test]
    fn test_media_subcommands() {
        type MediaAssert = fn(MediaCommands);

        let cases: Vec<(Vec<&str>, MediaAssert)> = vec![
            // media upload (basic)
            (
                vec!["xcom-rs", "media", "upload", "/tmp/image.jpg"],
                |cmd| {
                    let MediaCommands::Upload { path } = cmd;
                    assert_eq!(path, "/tmp/image.jpg");
                },
            ),
            // media upload (with --output json global flag)
            (
                vec![
                    "xcom-rs",
                    "--output",
                    "json",
                    "media",
                    "upload",
                    "/tmp/photo.png",
                ],
                |cmd| {
                    let MediaCommands::Upload { path } = cmd;
                    assert_eq!(path, "/tmp/photo.png");
                },
            ),
        ];

        for (args, assert_fn) in cases {
            let cli = parse(args.iter().copied());
            let Some(Commands::Media { command }) = cli.command else {
                panic!("Expected Media command for args={args:?}");
            };
            assert_fn(command);
        }
    }

    // ---------------------------------------------------------------------------
    // Additional regression: media upload with output flag sets global output
    // ---------------------------------------------------------------------------

    #[test]
    fn test_media_upload_global_output_flag() {
        let cli = parse([
            "xcom-rs",
            "--output",
            "json",
            "media",
            "upload",
            "/tmp/photo.png",
        ]);
        assert_eq!(cli.output, "json");
    }
}
