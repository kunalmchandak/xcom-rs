use crate::protocol::Envelope;
use anyhow::Result;
use serde::Serialize;
use std::str::FromStr;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Yaml,
    Text,
    Ndjson,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" | "json-schema" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "text" => Ok(OutputFormat::Text),
            "ndjson" => Ok(OutputFormat::Ndjson),
            _ => Err(anyhow::anyhow!("Invalid output format: {}", s)),
        }
    }
}

/// Format and print an envelope to stdout
pub fn print_envelope<T: Serialize>(envelope: &Envelope<T>, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(envelope)?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(envelope)?;
            println!("{}", yaml);
        }
        OutputFormat::Text => {
            print_envelope_text(envelope)?;
        }
        OutputFormat::Ndjson => {
            // For NDJSON, print compact JSON without pretty formatting
            let json = serde_json::to_string(envelope)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print array items as NDJSON (newline-delimited JSON)
pub fn print_ndjson<T: Serialize>(items: &[T]) -> Result<()> {
    for item in items {
        let json = serde_json::to_string(item)?;
        println!("{}", json);
    }
    Ok(())
}

/// Print envelope in human-readable text format
fn print_envelope_text<T: Serialize>(envelope: &Envelope<T>) -> Result<()> {
    if envelope.ok {
        if let Some(ref data) = envelope.data {
            let json = serde_json::to_value(data)?;
            print_value_text(&json, 0);
        } else {
            println!("Success");
        }
    } else if let Some(ref error) = envelope.error {
        eprintln!("Error: {:?} - {}", error.code, error.message);
        if let Some(ref details) = error.details {
            eprintln!("Details:");
            let json = serde_json::to_value(details)?;
            print_value_text(&json, 1);
        }
    }
    Ok(())
}

/// Helper to print JSON value in text format
fn print_value_text(value: &serde_json::Value, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        println!("{}{}:", prefix, key);
                        print_value_text(val, indent + 1);
                    }
                    _ => {
                        println!("{}{}: {}", prefix, key, format_value(val));
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        println!("{}[{}]:", prefix, i);
                        print_value_text(val, indent + 1);
                    }
                    _ => {
                        println!("{}[{}] {}", prefix, i, format_value(val));
                    }
                }
            }
        }
        _ => {
            println!("{}{}", prefix, format_value(value));
        }
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ErrorCode, ErrorDetails};
    use std::str::FromStr;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("yaml").unwrap(), OutputFormat::Yaml);
        assert_eq!(OutputFormat::from_str("text").unwrap(), OutputFormat::Text);
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_print_json_envelope() {
        let envelope = Envelope::success("test", "data");
        let result = print_envelope(&envelope, OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_error_envelope() {
        let error = ErrorDetails::new(ErrorCode::InvalidArgument, "test error");
        let envelope = Envelope::<()>::error("test", error);
        let result = print_envelope(&envelope, OutputFormat::Json);
        assert!(result.is_ok());
    }
}
