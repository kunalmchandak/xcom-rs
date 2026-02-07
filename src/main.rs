use clap::Parser;
use std::str::FromStr;
use xcom_rs::{
    cli::{Cli, Commands},
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
            // Determine error type and exit code
            let (error_code, exit_code) = match e.kind() {
                clap::error::ErrorKind::InvalidSubcommand
                | clap::error::ErrorKind::UnknownArgument => {
                    (ErrorCode::UnknownCommand, ExitCode::InvalidArgument)
                }
                _ => (ErrorCode::InvalidArgument, ExitCode::InvalidArgument),
            };

            let error = ErrorDetails::new(error_code, e.to_string());
            let envelope = Envelope::<()>::error("error", error);
            // Use JSON format for errors by default
            let _ = print_envelope(&envelope, OutputFormat::Json);
            std::process::exit(exit_code.into());
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
            let envelope = Envelope::<()>::error("error", error);
            let _ = print_envelope(&envelope, OutputFormat::Json);
            std::process::exit(ExitCode::InvalidArgument.into());
        }
    };

    // Execute command
    let result = match cli.command {
        Commands::Commands => {
            tracing::info!("Executing commands command");
            let commands = CommandsList::new();
            let envelope = Envelope::success("commands", commands);
            print_envelope(&envelope, output_format)
        }
        Commands::Schema { command } => {
            tracing::info!(command = %command, "Executing schema command");
            let schema = CommandSchema::for_command(&command);
            let envelope = Envelope::success("schema", schema);
            print_envelope(&envelope, output_format)
        }
        Commands::Help { command } => {
            tracing::info!(command = %command, "Executing help command");
            let help = CommandHelp::for_command(&command);
            let envelope = Envelope::success("help", help);
            print_envelope(&envelope, output_format)
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
            let envelope = Envelope::<()>::error("error", error);
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::OperationFailed.into());
        }
    }
}
