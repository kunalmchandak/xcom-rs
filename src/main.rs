use clap::{CommandFactory, Parser};
use std::str::FromStr;
use xcom_rs::{
    auth::AuthStore,
    billing::{BillingEstimate, BudgetTracker, CostEstimate, CostEstimator},
    cli::{AuthCommands, BillingCommands, Cli, Commands, TweetsCommands},
    context::{ExecutionContext, ExecutionPolicy},
    doctor,
    errors::ErrorResponder,
    introspection::{CommandHelp, CommandSchema, CommandsList},
    logging::{init_logging, LogFormat},
    output::{print_envelope, print_ndjson, OutputFormat},
    protocol::{Envelope, ErrorCode, ExitCode},
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

                let error = ErrorResponder::error(error_code, e.to_string());
                ErrorResponder::emit(error, OutputFormat::Json, None, exit_code);
            }
        },
    };

    let log_format = LogFormat::from_str(&cli.log_format).unwrap();
    init_logging(log_format, cli.trace_id.clone());

    let output_format = match OutputFormat::from_str(&cli.output) {
        Ok(fmt) => fmt,
        Err(e) => {
            let error = ErrorResponder::error(ErrorCode::InvalidArgument, e.to_string());
            let meta = ErrorResponder::create_meta(cli.trace_id.as_ref());
            ErrorResponder::emit(error, OutputFormat::Json, meta, ExitCode::InvalidArgument);
        }
    };

    let create_meta = || -> Option<std::collections::HashMap<String, serde_json::Value>> {
        ErrorResponder::create_meta(cli.trace_id.as_ref())
    };

    // If no subcommand is provided, show help and exit successfully
    if cli.command.is_none() {
        let _ = Cli::command().print_help();
        std::process::exit(ExitCode::Success.into());
    }

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

    let policy = ExecutionPolicy::new();

    let mut auth_store = AuthStore::with_default_storage().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to create persistent auth store, using in-memory store");
        AuthStore::new()
    });

    let result = match cli
        .command
        .expect("Command should be present after None check")
    {
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

            if let Some(error) = policy.check_interaction_required(
                &ctx,
                "This command requires user confirmation",
                vec![
                    "Run with interactive mode enabled (remove --non-interactive flag)".to_string(),
                    "Or use --yes flag to auto-confirm (not implemented in this demo)".to_string(),
                ],
            ) {
                let meta = create_meta();
                ErrorResponder::emit(error, output_format, meta, ExitCode::AuthenticationError);
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
                            let error =
                                ErrorResponder::error(ErrorCode::InvalidArgument, e.to_string());
                            let meta = create_meta();
                            ErrorResponder::emit(
                                error,
                                output_format,
                                meta,
                                ExitCode::InvalidArgument,
                            );
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
                                ErrorResponder::error(ErrorCode::IdempotencyConflict, e.to_string())
                            } else if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
                                if let Some(retry_after_ms) = classified.retry_after_ms {
                                    ErrorResponder::error_with_retry(
                                        classified.to_error_code(),
                                        e.to_string(),
                                        retry_after_ms,
                                    )
                                } else {
                                    ErrorResponder::error(classified.to_error_code(), e.to_string())
                                }
                            } else {
                                ErrorResponder::error(ErrorCode::InternalError, e.to_string())
                            };

                            let meta = create_meta();
                            ErrorResponder::emit(
                                error,
                                output_format,
                                meta,
                                ExitCode::OperationFailed,
                            );
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
                            let error = if let Some(classified) =
                                e.downcast_ref::<ClassifiedError>()
                            {
                                if let Some(retry_after_ms) = classified.retry_after_ms {
                                    ErrorResponder::error_with_retry(
                                        classified.to_error_code(),
                                        e.to_string(),
                                        retry_after_ms,
                                    )
                                } else {
                                    ErrorResponder::error(classified.to_error_code(), e.to_string())
                                }
                            } else {
                                ErrorResponder::error(ErrorCode::InternalError, e.to_string())
                            };

                            let meta = create_meta();
                            ErrorResponder::emit(
                                error,
                                output_format,
                                meta,
                                ExitCode::OperationFailed,
                            );
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
                            let error = ErrorResponder::auth_required_error(
                                e.to_string(),
                                vec![
                                    "Authenticate first by running 'xcom-rs auth login'"
                                        .to_string(),
                                    "Or import existing credentials with 'xcom-rs auth import'"
                                        .to_string(),
                                ],
                            );
                            let meta = create_meta();
                            ErrorResponder::emit(
                                error,
                                output_format,
                                meta,
                                ExitCode::AuthenticationError,
                            );
                        }
                    }
                }
                AuthCommands::Import { data, dry_run } => {
                    tracing::info!(dry_run = dry_run, "Executing auth import command");
                    match auth_store.import_with_plan(&data, dry_run) {
                        Ok(plan) => {
                            // Check if the plan indicates failure
                            if plan.action == xcom_rs::auth::ImportAction::Fail {
                                let error = ErrorResponder::error(
                                    ErrorCode::InvalidArgument,
                                    plan.reason.unwrap_or_else(|| "Import failed".to_string()),
                                );
                                let meta = create_meta();
                                ErrorResponder::emit(
                                    error,
                                    output_format,
                                    meta,
                                    ExitCode::InvalidArgument,
                                );
                            }

                            // For dry-run, return the plan
                            if dry_run {
                                let envelope = if let Some(meta) = create_meta() {
                                    Envelope::success_with_meta("auth.import", plan, meta)
                                } else {
                                    Envelope::success("auth.import", plan)
                                };
                                print_envelope(&envelope, output_format)
                            } else {
                                // For actual import, return the status
                                let status = auth_store.status();
                                let envelope = if let Some(meta) = create_meta() {
                                    Envelope::success_with_meta("auth.import", status, meta)
                                } else {
                                    Envelope::success("auth.import", status)
                                };
                                print_envelope(&envelope, output_format)
                            }
                        }
                        Err(e) => {
                            let error =
                                ErrorResponder::error(ErrorCode::InvalidArgument, e.to_string());
                            let meta = create_meta();
                            ErrorResponder::emit(
                                error,
                                output_format,
                                meta,
                                ExitCode::InvalidArgument,
                            );
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

                    if let Some(error) = policy.check_max_cost(&ctx, &cost) {
                        let meta = create_meta();
                        ErrorResponder::emit(error, output_format, meta, ExitCode::OperationFailed);
                    }

                    if let Some(error) = policy.check_daily_budget(&ctx, &cost, &budget_tracker) {
                        let meta = create_meta();
                        ErrorResponder::emit(error, output_format, meta, ExitCode::OperationFailed);
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
        Commands::Doctor => {
            tracing::info!("Executing doctor command");
            match doctor::collect_diagnostics(&auth_store, &ctx) {
                Ok(diagnostics) => {
                    let envelope = if let Some(meta) = create_meta() {
                        Envelope::success_with_meta("doctor", diagnostics, meta)
                    } else {
                        Envelope::success("doctor", diagnostics)
                    };
                    print_envelope(&envelope, output_format)
                }
                Err(e) => {
                    // If diagnostics collection fails completely, return error with next steps
                    let mut next_steps = vec![
                        "Check that configuration directories are accessible".to_string(),
                        "Verify file permissions for auth and budget storage locations".to_string(),
                    ];

                    // Try to provide specific paths even if collection failed
                    if let Ok(auth_path) = AuthStore::default_storage_path() {
                        next_steps.push(format!("Auth storage: {}", auth_path.display()));
                    }
                    if let Ok(budget_path) = BudgetTracker::default_storage_path() {
                        next_steps.push(format!("Budget storage: {}", budget_path.display()));
                    }

                    let mut details = std::collections::HashMap::new();
                    details.insert("nextSteps".to_string(), serde_json::json!(next_steps));

                    let error = ErrorResponder::error_with_details(
                        ErrorCode::InternalError,
                        format!("Failed to collect diagnostics: {}", e),
                        details,
                    );
                    let meta = create_meta();
                    ErrorResponder::emit(error, output_format, meta, ExitCode::OperationFailed);
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
            let error = ErrorResponder::error(ErrorCode::InternalError, e.to_string());
            let meta = create_meta();
            ErrorResponder::emit(error, output_format, meta, ExitCode::OperationFailed);
        }
    }
}
