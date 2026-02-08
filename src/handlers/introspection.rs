use crate::{
    introspection::{CommandHelp, CommandSchema, CommandsList},
    output::{print_envelope, OutputFormat},
    protocol::Envelope,
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_commands(
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing commands command");
    let commands = CommandsList::new();
    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("commands", commands, meta)
    } else {
        Envelope::success("commands", commands)
    };
    print_envelope(&envelope, output_format)
}

pub fn handle_schema(
    command: &str,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(command = %command, "Executing schema command");
    let schema = CommandSchema::for_command(command);
    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("schema", schema, meta)
    } else {
        Envelope::success("schema", schema)
    };
    print_envelope(&envelope, output_format)
}

pub fn handle_help(
    command: &str,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(command = %command, "Executing help command");
    let help = CommandHelp::for_command(command);
    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("help", help, meta)
    } else {
        Envelope::success("help", help)
    };
    print_envelope(&envelope, output_format)
}
