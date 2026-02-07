use clap::Parser;
use std::str::FromStr;
use xcom_rs::{
    cli::{Cli, Commands, TweetsCommands},
    context::ExecutionContext,
    introspection::{CommandHelp, CommandSchema, CommandsList},
    logging::{init_logging, LogFormat},
    output::{print_envelope, print_ndjson, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
    tweets::{
        ClassifiedError, CreateArgs, IdempotencyConflictError, IdempotencyLedger, IfExistsPolicy,
        ListArgs, TweetCommand, TweetFields,
    },
};

fn main() {
    // Try to parse CLI, catch errors and convert to JSON
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Special handling for --help and --version (success cases)
            match e.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    // Print help/version text to stdout and exit successfully
                    print!("{}", e);
                    std::process::exit(ExitCode::Success.into());
                }
                _ => {
                    // Determine error type and exit code
                    let (error_code, exit_code) = match e.kind() {
                        clap::error::ErrorKind::InvalidSubcommand
                        | clap::error::ErrorKind::UnknownArgument => {
                            (ErrorCode::UnknownCommand, ExitCode::InvalidArgument)
                        }
                        _ => (ErrorCode::InvalidArgument, ExitCode::InvalidArgument),
                    };

                    let error = ErrorDetails::new(error_code, e.to_string());
                    // Note: trace_id is not available at this point since CLI parsing failed
                    let envelope = Envelope::<()>::error("error", error);
                    // Use JSON format for errors by default
                    let _ = print_envelope(&envelope, OutputFormat::Json);
                    std::process::exit(exit_code.into());
                }
            }
        }
    };

    // Initialize logging
    let log_format = LogFormat::from_str(&cli.log_format).unwrap();
    init_logging(log_format, cli.trace_id.clone());

    // Parse output format
    let output_format = match OutputFormat::from_str(&cli.output) {
        Ok(fmt) => fmt,
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InvalidArgument, e.to_string());
            let meta = cli.trace_id.as_ref().map(|trace_id| {
                let mut m = std::collections::HashMap::new();
                m.insert("traceId".to_string(), serde_json::json!(trace_id));
                m
            });
            let envelope = if let Some(meta) = meta {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, OutputFormat::Json);
            std::process::exit(ExitCode::InvalidArgument.into());
        }
    };

    // Helper function to create meta with trace_id if provided
    let create_meta = || -> Option<std::collections::HashMap<String, serde_json::Value>> {
        cli.trace_id.as_ref().map(|trace_id| {
            let mut m = std::collections::HashMap::new();
            m.insert("traceId".to_string(), serde_json::json!(trace_id));
            m
        })
    };

    // Create execution context for commands
    let ctx = ExecutionContext::new(cli.non_interactive, cli.trace_id.clone());

    // Log non-interactive mode if enabled
    if ctx.non_interactive {
        tracing::info!("Running in non-interactive mode");
    }

    // Execute command
    // Note: ExecutionContext is available for commands that need to check interaction requirements
    // Commands can use ctx.check_interaction_required() to handle non-interactive mode properly
    let result = match cli.command {
        Commands::Commands => {
            tracing::info!("Executing commands command");
            let commands = CommandsList::new();
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("commands", commands, meta)
            } else {
                Envelope::success("commands", commands)
            };
            print_envelope(&envelope, output_format)
        }
        Commands::Schema { command } => {
            tracing::info!(command = %command, "Executing schema command");
            let schema = CommandSchema::for_command(&command);
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("schema", schema, meta)
            } else {
                Envelope::success("schema", schema)
            };
            print_envelope(&envelope, output_format)
        }
        Commands::Help { command } => {
            tracing::info!(command = %command, "Executing help command");
            let help = CommandHelp::for_command(&command);
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("help", help, meta)
            } else {
                Envelope::success("help", help)
            };
            print_envelope(&envelope, output_format)
        }
        Commands::DemoInteractive => {
            tracing::info!("Executing demo-interactive command");

            // Check if interaction is required and we're in non-interactive mode
            if let Some(error) = ctx.check_interaction_required(
                "This command requires user confirmation",
                vec![
                    "Run with interactive mode enabled (remove --non-interactive flag)".to_string(),
                    "Or use --yes flag to auto-confirm (not implemented in this demo)".to_string(),
                ],
            ) {
                // Return structured error with next steps
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::<()>::error_with_meta("error", error, meta)
                } else {
                    Envelope::<()>::error("error", error)
                };
                let _ = print_envelope(&envelope, output_format);
                std::process::exit(ExitCode::OperationFailed.into());
            }

            // In interactive mode, would show prompt here
            // For demo purposes, just return success
            #[derive(serde::Serialize)]
            struct DemoResult {
                message: String,
                confirmed: bool,
            }

            let result = DemoResult {
                message: "User confirmed action".to_string(),
                confirmed: true,
            };

            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("demo-interactive", result, meta)
            } else {
                Envelope::success("demo-interactive", result)
            };
            print_envelope(&envelope, output_format)
        }
        Commands::Tweets { command } => {
            // Initialize idempotency ledger
            let ledger =
                IdempotencyLedger::new(None).expect("Failed to initialize idempotency ledger");
            let tweet_cmd = TweetCommand::new(ledger);

            match command {
                TweetsCommands::Create {
                    text,
                    client_request_id,
                    if_exists,
                } => {
                    tracing::info!(text = %text, "Creating tweet");

                    let if_exists_policy =
                        IfExistsPolicy::from_str(&if_exists).unwrap_or_else(|e| {
                            eprintln!("Invalid if-exists policy: {}", e);
                            std::process::exit(ExitCode::InvalidArgument.into());
                        });

                    let args = CreateArgs {
                        text,
                        client_request_id,
                        if_exists: if_exists_policy,
                    };

                    match tweet_cmd.create(args) {
                        Ok(result) => {
                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::success_with_meta("tweets.create", result, meta)
                            } else {
                                Envelope::success("tweets.create", result)
                            };
                            print_envelope(&envelope, output_format)
                        }
                        Err(e) => {
                            // Determine error code and retry info based on error type
                            let error = if e.downcast_ref::<IdempotencyConflictError>().is_some() {
                                ErrorDetails::new(ErrorCode::IdempotencyConflict, e.to_string())
                            } else if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
                                // Use ClassifiedError to get proper error code and retry info
                                if let Some(retry_after_ms) = classified.retry_after_ms {
                                    ErrorDetails::with_retry_after(
                                        classified.to_error_code(),
                                        e.to_string(),
                                        retry_after_ms,
                                    )
                                } else {
                                    ErrorDetails::new(classified.to_error_code(), e.to_string())
                                }
                            } else {
                                ErrorDetails::new(ErrorCode::InternalError, e.to_string())
                            };

                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::<()>::error_with_meta("error", error, meta)
                            } else {
                                Envelope::<()>::error("error", error)
                            };
                            let _ = print_envelope(&envelope, output_format);
                            std::process::exit(ExitCode::OperationFailed.into());
                        }
                    }
                }
                TweetsCommands::List {
                    fields,
                    limit,
                    cursor,
                } => {
                    tracing::info!("Listing tweets");

                    // Parse fields or use defaults
                    let field_list = if let Some(fields_str) = fields {
                        fields_str
                            .split(',')
                            .filter_map(|s| TweetFields::parse(s.trim()))
                            .collect()
                    } else {
                        TweetFields::default_fields()
                    };

                    let args = ListArgs {
                        fields: field_list,
                        limit,
                        cursor,
                    };

                    match tweet_cmd.list(args) {
                        Ok(result) => {
                            // For NDJSON format, print tweets line-by-line
                            if output_format == OutputFormat::Ndjson {
                                print_ndjson(&result.tweets)
                            } else {
                                let envelope = if let Some(meta) = create_meta() {
                                    Envelope::success_with_meta("tweets.list", result, meta)
                                } else {
                                    Envelope::success("tweets.list", result)
                                };
                                print_envelope(&envelope, output_format)
                            }
                        }
                        Err(e) => {
                            // Classify error if possible
                            let error =
                                if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
                                    if let Some(retry_after_ms) = classified.retry_after_ms {
                                        ErrorDetails::with_retry_after(
                                            classified.to_error_code(),
                                            e.to_string(),
                                            retry_after_ms,
                                        )
                                    } else {
                                        ErrorDetails::new(classified.to_error_code(), e.to_string())
                                    }
                                } else {
                                    ErrorDetails::new(ErrorCode::InternalError, e.to_string())
                                };

                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::<()>::error_with_meta("error", error, meta)
                            } else {
                                Envelope::<()>::error("error", error)
                            };
                            let _ = print_envelope(&envelope, output_format);
                            std::process::exit(ExitCode::OperationFailed.into());
                        }
                    }
                }
            }
        }
    };

    // Handle errors
    match result {
        Ok(_) => {
            tracing::info!("Command completed successfully");
            std::process::exit(ExitCode::Success.into());
        }
        Err(e) => {
            tracing::error!(error = %e, "Command failed");
            let error = ErrorDetails::new(ErrorCode::InternalError, e.to_string());
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::OperationFailed.into());
        }
    }
}
