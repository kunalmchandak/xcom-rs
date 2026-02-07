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
    pub command: Commands,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(["xcom-rs", "commands"]);
        assert!(matches!(cli.command, Commands::Commands));
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
        if let Commands::Schema { command, .. } = cli.command {
            assert_eq!(command, "commands");
        } else {
            panic!("Expected Schema command");
        }
    }

    #[test]
    fn test_help_command() {
        let cli = Cli::parse_from(["xcom-rs", "help", "commands"]);
        if let Commands::Help { command } = cli.command {
            assert_eq!(command, "commands");
        } else {
            panic!("Expected Help command");
        }
    }
}
