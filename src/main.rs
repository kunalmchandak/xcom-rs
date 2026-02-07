use clap::Parser;
use std::str::FromStr;
use xcom_rs::{
    auth::AuthStore,
    billing::{BillingEstimate, BudgetTracker, CostEstimate, CostEstimator},
    cli::{AuthCommands, BillingCommands, Cli, Commands, TweetsCommands},
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
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => match e.kind() {
            clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                print!("{}", e);
                std::process::exit(ExitCode::Success.into());
            }
            _ => {
                let (error_code, exit_code) = match e.kind() {
                    clap::error::ErrorKind::InvalidSubcommand
                    | clap::error::ErrorKind::UnknownArgument => {
                        (ErrorCode::UnknownCommand, ExitCode::InvalidArgument)
                    }
                    _ => (ErrorCode::InvalidArgument, ExitCode::InvalidArgument),
                };

                let error = ErrorDetails::new(error_code, e.to_string());
                let envelope = Envelope::<()>::error("error", error);
                let _ = print_envelope(&envelope, OutputFormat::Json);
                std::process::exit(exit_code.into());
            }
        },
    };

    let log_format = LogFormat::from_str(&cli.log_format).unwrap();
    init_logging(log_format, cli.trace_id.clone());

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

    let create_meta = || -> Option<std::collections::HashMap<String, serde_json::Value>> {
        cli.trace_id.as_ref().map(|trace_id| {
            let mut m = std::collections::HashMap::new();
            m.insert("traceId".to_string(), serde_json::json!(trace_id));
            m
        })
    };

    let ctx = ExecutionContext::new(
        cli.non_interactive,
        cli.trace_id.clone(),
        cli.max_cost_credits,
        cli.budget_daily_credits,
        cli.dry_run,
    );

    if ctx.non_interactive {
        tracing::info!("Running in non-interactive mode");
    }

    let mut auth_store = AuthStore::with_default_storage().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to create persistent auth store, using in-memory store");
        AuthStore::new()
    });

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

            if let Some(error) = ctx.check_interaction_required(
                "This command requires user confirmation",
                vec![
                    "Run with interactive mode enabled (remove --non-interactive flag)".to_string(),
                    "Or use --yes flag to auto-confirm (not implemented in this demo)".to_string(),
                ],
            ) {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::<()>::error_with_meta("error", error, meta)
                } else {
                    Envelope::<()>::error("error", error)
                };
                let _ = print_envelope(&envelope, output_format);
                std::process::exit(ExitCode::AuthenticationError.into());
            }

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
                            let error = if e.downcast_ref::<IdempotencyConflictError>().is_some() {
                                ErrorDetails::new(ErrorCode::IdempotencyConflict, e.to_string())
                            } else if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
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
        Commands::Auth { command } => {
            tracing::info!("Executing auth command");
            match command {
                AuthCommands::Status => {
                    tracing::info!("Executing auth status command");
                    let status = auth_store.status();
                    let envelope = if let Some(meta) = create_meta() {
                        Envelope::success_with_meta("auth.status", status, meta)
                    } else {
                        Envelope::success("auth.status", status)
                    };
                    print_envelope(&envelope, output_format)
                }
                AuthCommands::Export => {
                    tracing::info!("Executing auth export command");
                    match auth_store.export() {
                        Ok(data) => {
                            #[derive(serde::Serialize)]
                            struct ExportResult {
                                data: String,
                            }
                            let result = ExportResult { data };
                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::success_with_meta("auth.export", result, meta)
                            } else {
                                Envelope::success("auth.export", result)
                            };
                            print_envelope(&envelope, output_format)
                        }
                        Err(e) => {
                            let error = ErrorDetails::auth_required(
                                e.to_string(),
                                vec![
                                    "Authenticate first by running 'xcom-rs auth login'"
                                        .to_string(),
                                    "Or import existing credentials with 'xcom-rs auth import'"
                                        .to_string(),
                                ],
                            );
                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::<()>::error_with_meta("error", error, meta)
                            } else {
                                Envelope::<()>::error("error", error)
                            };
                            let _ = print_envelope(&envelope, output_format);
                            std::process::exit(ExitCode::AuthenticationError.into());
                        }
                    }
                }
                AuthCommands::Import { data } => {
                    tracing::info!("Executing auth import command");
                    match auth_store.import(&data) {
                        Ok(_) => {
                            let status = auth_store.status();
                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::success_with_meta("auth.import", status, meta)
                            } else {
                                Envelope::success("auth.import", status)
                            };
                            print_envelope(&envelope, output_format)
                        }
                        Err(e) => {
                            let error =
                                ErrorDetails::new(ErrorCode::InvalidArgument, e.to_string());
                            let envelope = if let Some(meta) = create_meta() {
                                Envelope::<()>::error_with_meta("error", error, meta)
                            } else {
                                Envelope::<()>::error("error", error)
                            };
                            let _ = print_envelope(&envelope, output_format);
                            std::process::exit(ExitCode::InvalidArgument.into());
                        }
                    }
                }
            }
        }
        Commands::Billing { command } => {
            tracing::info!("Executing billing command");
            let estimator = CostEstimator::new();
            let mut budget_tracker =
                BudgetTracker::with_default_storage(ctx.budget_daily_credits).unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Failed to create persistent budget tracker, using in-memory tracker");
                    BudgetTracker::new(ctx.budget_daily_credits)
                });

            match command {
                BillingCommands::Estimate { operation, text } => {
                    tracing::info!(operation = %operation, "Executing billing estimate command");
                    let mut params = std::collections::HashMap::new();
                    if let Some(text_val) = text {
                        params.insert("text".to_string(), text_val);
                    }

                    let cost: CostEstimate = if ctx.dry_run {
                        CostEstimate::zero()
                    } else {
                        estimator.estimate(&operation, &params)
                    };

                    if let Some(error) = ctx.check_max_cost(&cost) {
                        let envelope = if let Some(meta) = create_meta() {
                            Envelope::<()>::error_with_meta("error", error, meta)
                        } else {
                            Envelope::<()>::error("error", error)
                        };
                        let _ = print_envelope(&envelope, output_format);
                        std::process::exit(ExitCode::OperationFailed.into());
                    }

                    if let Some(error) = ctx.check_daily_budget(&cost, &budget_tracker) {
                        let envelope = if let Some(meta) = create_meta() {
                            Envelope::<()>::error_with_meta("error", error, meta)
                        } else {
                            Envelope::<()>::error("error", error)
                        };
                        let _ = print_envelope(&envelope, output_format);
                        std::process::exit(ExitCode::OperationFailed.into());
                    }

                    if !ctx.dry_run {
                        budget_tracker.record_usage(cost.credits);
                    }

                    let estimate = BillingEstimate {
                        operation: operation.clone(),
                        cost: cost.clone(),
                    };

                    let mut meta_map = create_meta().unwrap_or_default();
                    if ctx.dry_run {
                        meta_map.insert("dryRun".to_string(), serde_json::json!(true));
                        meta_map.insert(
                            "cost".to_string(),
                            serde_json::json!({
                                "credits": 0,
                                "usdEstimated": 0.0
                            }),
                        );
                    }

                    let envelope = if !meta_map.is_empty() {
                        Envelope::success_with_meta("billing.estimate", estimate, meta_map)
                    } else {
                        Envelope::success("billing.estimate", estimate)
                    };
                    print_envelope(&envelope, output_format)
                }
                BillingCommands::Report => {
                    tracing::info!("Executing billing report command");
                    #[derive(serde::Serialize)]
                    struct BillingReport {
                        #[serde(rename = "todayUsage")]
                        today_usage: u32,
                    }

                    let report = BillingReport { today_usage: 0 };
                    let envelope = if let Some(meta) = create_meta() {
                        Envelope::success_with_meta("billing.report", report, meta)
                    } else {
                        Envelope::success("billing.report", report)
                    };
                    print_envelope(&envelope, output_format)
                }
            }
        }
    };

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
