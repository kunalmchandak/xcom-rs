use std::str::FromStr;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Json,
    Text,
}

impl FromStr for LogFormat {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(LogFormat::Json),
            _ => Ok(LogFormat::Text),
        }
    }
}

/// Initialize logging with the specified format and trace ID
pub fn init_logging(format: LogFormat, trace_id: Option<String>) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match format {
        LogFormat::Json => {
            let fmt_layer = fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(false)
                .with_writer(std::io::stderr);

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Text => {
            let fmt_layer = fmt::layer().with_writer(std::io::stderr).with_target(false);

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
    }

    // Log the trace ID if provided
    if let Some(tid) = trace_id {
        tracing::info!(traceId = %tid, "Request started");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("JSON").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("text").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::from_str("anything").unwrap(), LogFormat::Text);
    }
}
