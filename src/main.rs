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

/// Parse --output flag from raw command-line arguments
/// Returns the output format (defaults to Text if not specified or invalid)
fn parse_output_from_args() -> OutputFormat {
    let args: Vec<String> = std::env::args().collect();
    let mut output_value: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        // Stop processing after --
        if arg == "--" {
            break;
        }

        // Handle --output=value
        if let Some(stripped) = arg.strip_prefix("--output=") {
            output_value = Some(stripped);
            i += 1;
            continue;
        }

        // Handle --output value
        if arg == "--output" && i + 1 < args.len() {
            output_value = Some(&args[i + 1]);
            i += 2;
            continue;
        }

        i += 1;
    }

    // Try to parse the output value, default to Text on invalid/missing
    output_value
        .and_then(|v| OutputFormat::from_str(v).ok())
        .unwrap_or(OutputFormat::Text)
}

fn main() {
    // Parse --output early to determine error format before CLI parsing
    let early_output_format = parse_output_from_args();

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
                ErrorResponder::emit(error, early_output_format, None, exit_code);
            }
        },
    };

    // Determine final output format from parsed CLI
    let output_format = match OutputFormat::from_str(&cli.output) {
        Ok(fmt) => fmt,
        Err(e) => {
            let error = ErrorResponder::error(ErrorCode::InvalidArgument, e.to_string());
            let meta = ErrorResponder::create_meta(cli.trace_id.as_ref());
            ErrorResponder::emit(error, early_output_format, meta, ExitCode::InvalidArgument);
        }
    };

    // Validate log format after output format is determined
    let log_format = match LogFormat::from_str(&cli.log_format) {
        Ok(fmt) => fmt,
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InvalidArgument, e.to_string());
            let envelope = Envelope::<()>::error("error", error);
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::InvalidArgument.into());
        }
    };
    init_logging(log_format, cli.trace_id.clone());

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
        Commands::Completion { shell } => handlers::completion::handle_completion(shell),
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
