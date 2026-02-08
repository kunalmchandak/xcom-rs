use crate::{
    context::ExecutionContext,
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ExitCode},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_demo_interactive(
    ctx: &ExecutionContext,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
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
