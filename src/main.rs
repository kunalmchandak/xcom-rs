use clap::Parser;
use std::str::FromStr;
use xcom_rs::{
    auth::AuthStore,
    billing::{BillingEstimate, BudgetTracker, CostEstimator},
    cli::{AuthCommands, BillingCommands, Cli, Commands},
    context::ExecutionContext,
    introspection::{CommandHelp, CommandSchema, CommandsList},
    logging::{init_logging, LogFormat},
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
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
    let ctx = ExecutionContext::new(
        cli.non_interactive,
        cli.trace_id.clone(),
        cli.max_cost_credits,
        cli.budget_daily_credits,
        cli.dry_run,
    );

    // Log non-interactive mode if enabled
    if ctx.non_interactive {
        tracing::info!("Running in non-interactive mode");
    }

    // Create auth store (in-memory for now; in real impl would persist to disk)
    let mut auth_store = AuthStore::new();

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
                            let error = ErrorDetails::new(ErrorCode::AuthRequired, e.to_string());
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
            let mut budget_tracker = BudgetTracker::new(ctx.budget_daily_credits);

            match command {
                BillingCommands::Estimate { operation, text } => {
                    tracing::info!(operation = %operation, "Executing billing estimate command");
                    let mut params = std::collections::HashMap::new();
                    if let Some(text_val) = text {
                        params.insert("text".to_string(), text_val);
                    }

                    let cost = if ctx.dry_run {
                        // In dry-run mode, return zero cost
                        xcom_rs::billing::CostEstimate::zero()
                    } else {
                        estimator.estimate(&operation, &params)
                    };

                    // Check cost limits
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

                    // Record usage if not dry-run
                    if !ctx.dry_run {
                        budget_tracker.record_usage(cost.credits);
                    }

                    let estimate = BillingEstimate {
                        operation: operation.clone(),
                        cost,
                    };

                    let mut meta_map = create_meta().unwrap_or_default();
                    if ctx.dry_run {
                        meta_map.insert("dryRun".to_string(), serde_json::json!(true));
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
                    // For now, return a stub report
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
