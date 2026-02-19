use clap::{CommandFactory, Parser};
use std::str::FromStr;
use xcom_rs::{
    auth::AuthStore,
    cli::{Cli, Commands},
    context::ExecutionContext,
    errors::ErrorResponder,
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

                let error = ErrorResponder::error(error_code, e.to_string());
                ErrorResponder::emit(error, OutputFormat::Json, None, exit_code);
            }
        },
    };

    let log_format = match LogFormat::from_str(&cli.log_format) {
        Ok(fmt) => fmt,
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InvalidArgument, e.to_string());
            let envelope = Envelope::<()>::error("error", error);
            let _ = print_envelope(&envelope, OutputFormat::Json);
            std::process::exit(ExitCode::InvalidArgument.into());
        }
    };
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

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            let _ = Cli::command().print_help();
            std::process::exit(ExitCode::Success.into());
        }
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

    let result = match command {
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
        Commands::Bookmarks { command } => {
            handlers::bookmarks::handle_bookmarks(command, &create_meta, output_format)
        }
        Commands::Auth { command } => {
            handlers::auth::handle_auth(command, &mut auth_store, &create_meta, output_format)
        }
        Commands::Billing { command } => {
            handlers::billing::handle_billing(command, &ctx, &create_meta, output_format)
        }
        Commands::Doctor { probe } => {
            handlers::doctor::handle_doctor(&auth_store, &ctx, probe, &create_meta, output_format)
        }
        Commands::InstallSkills {
            skill,
            agent,
            global,
            yes,
        } => handlers::skills::handle_install_skills(
            skill.as_deref(),
            agent.as_deref(),
            global,
            yes,
            &ctx,
            &create_meta,
            output_format,
        ),
        Commands::Search { command } => {
            handlers::search::handle_search(command, &create_meta, output_format)
        }
        Commands::Timeline { command } => {
            handlers::timeline::handle_timeline(command, &create_meta, output_format)
        }
        Commands::Media { command } => {
            handlers::media::handle_media(command, &create_meta, output_format)
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
