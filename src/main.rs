use clap::{CommandFactory, Parser};
use std::str::FromStr;
use xcom_rs::{
    auth::AuthStore,
    cli::{Cli, Commands},
    context::ExecutionContext,
    handlers,
    logging::{init_logging, LogFormat},
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
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

    let mut auth_store = AuthStore::with_default_storage().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to create persistent auth store, using in-memory store");
        AuthStore::new()
    });

    let result = match cli
        .command
        .expect("Command should be present after None check")
    {
        Commands::Commands => handlers::introspection::handle_commands(&create_meta, output_format),
        Commands::Schema { command } => {
            handlers::introspection::handle_schema(&command, &create_meta, output_format)
        }
        Commands::Help { command } => {
            handlers::introspection::handle_help(&command, &create_meta, output_format)
        }
        Commands::DemoInteractive => {
            handlers::demo::handle_demo_interactive(&ctx, &create_meta, output_format)
        }
        Commands::Tweets { command } => {
            handlers::tweets::handle_tweets(command, &create_meta, output_format)
        }
        Commands::Auth { command } => {
            handlers::auth::handle_auth(command, &mut auth_store, &create_meta, output_format)
        }
        Commands::Billing { command } => {
            handlers::billing::handle_billing(command, &ctx, &create_meta, output_format)
        }
        Commands::Doctor => {
            handlers::doctor::handle_doctor(&auth_store, &ctx, &create_meta, output_format)
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
