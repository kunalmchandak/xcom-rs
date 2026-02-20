use crate::{
    auth::AuthStore,
    cli::AuthCommands,
    output::{print_envelope, OutputFormat},
    protocol::Envelope,
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_auth(
    command: AuthCommands,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing auth command");
    match command {
        AuthCommands::Status => handle_status(auth_store, create_meta, output_format),
    }
}

fn handle_status(
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing auth status command");
    let status = auth_store.status();
    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("auth.status", status, meta)
    } else {
        Envelope::success("auth.status", status)
    };
    print_envelope(&envelope, output_format)
}
