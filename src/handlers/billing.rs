use crate::{
    billing::{BillingEstimate, BudgetTracker, CostEstimate, CostEstimator},
    cli::BillingCommands,
    context::ExecutionContext,
    output::{print_envelope, OutputFormat},
    protocol::Envelope,
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_billing(
    command: BillingCommands,
    ctx: &ExecutionContext,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing billing command");
    let estimator = CostEstimator::new();
    let budget_tracker =
        BudgetTracker::with_default_storage(ctx.budget_daily_credits).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to create persistent budget tracker, using in-memory tracker");
            BudgetTracker::new(ctx.budget_daily_credits)
        });

    match command {
        BillingCommands::Estimate { operation, text } => handle_estimate(
            estimator,
            budget_tracker,
            operation,
            text,
            ctx,
            create_meta,
            output_format,
        ),
        BillingCommands::Report => handle_report(create_meta, output_format),
    }
}

fn handle_estimate(
    estimator: CostEstimator,
    mut budget_tracker: BudgetTracker,
    operation: String,
    text: Option<String>,
    ctx: &ExecutionContext,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(operation = %operation, "Executing billing estimate command");
    let mut params = HashMap::new();
    if let Some(text_val) = text {
        params.insert("text".to_string(), text_val);
    }

    let cost: CostEstimate = if ctx.dry_run {
        CostEstimate::zero()
    } else {
        estimator.estimate(&operation, &params)
    };

    if let Some(error) = ctx.check_max_cost(&cost) {
        let envelope = if let Some(meta) = create_meta() {
            Envelope::<()>::error_with_meta("error", error, meta)
        } else {
            Envelope::<()>::error("error", error)
        };
        let _ = print_envelope(&envelope, output_format);
        std::process::exit(crate::protocol::ExitCode::OperationFailed.into());
    }

    if let Some(error) = ctx.check_daily_budget(&cost, &budget_tracker) {
        let envelope = if let Some(meta) = create_meta() {
            Envelope::<()>::error_with_meta("error", error, meta)
        } else {
            Envelope::<()>::error("error", error)
        };
        let _ = print_envelope(&envelope, output_format);
        std::process::exit(crate::protocol::ExitCode::OperationFailed.into());
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

fn handle_report(
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
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
