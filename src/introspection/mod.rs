//! Introspection module.
//!
//! Provides data-driven command registry and generators for the three
//! introspection commands (`commands`, `schema`, `help`).
//!
//! # Module layout
//! - [`registry`] – single source of truth for all command metadata
//! - [`schema`]   – JSON Schema generation per command
//! - [`help`]     – detailed help generation per command
//!
//! # Public API
//! All types previously in `introspection.rs` are re-exported from here so
//! that existing call sites (`crate::introspection::CommandsList`, etc.)
//! continue to work without modification.

pub mod help;
pub mod registry;
pub mod schema;

// Re-export the full public surface so that callers such as
// `src/handlers/introspection.rs` require no changes.
pub use help::{CommandHelp, ErrorCodeInfo, ExampleInfo, ExitCodeInfo};
pub use registry::{ArgumentInfo, CommandInfo, CommandsList, RiskLevel};
pub use schema::CommandSchema;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands_list() {
        let list = CommandsList::new();
        assert!(!list.commands.is_empty());
        assert!(list.commands.iter().any(|c| c.name == "commands"));
        assert!(list.commands.iter().any(|c| c.name == "schema"));
        assert!(list.commands.iter().any(|c| c.name == "help"));
    }

    #[test]
    fn test_commands_list_includes_media_upload() {
        let list = CommandsList::new();
        let media_cmd = list.commands.iter().find(|c| c.name == "media.upload");
        assert!(
            media_cmd.is_some(),
            "media.upload should be in commands list"
        );
        let media_cmd = media_cmd.unwrap();
        assert_eq!(media_cmd.risk, RiskLevel::Medium);
        assert!(media_cmd.has_cost);
        let path_arg = media_cmd.arguments.iter().find(|a| a.name == "path");
        assert!(path_arg.is_some(), "media.upload should have path argument");
        assert!(
            path_arg.unwrap().required,
            "path argument should be required"
        );
    }

    #[test]
    fn test_media_upload_schema() {
        let schema = CommandSchema::for_command("media.upload");
        assert_eq!(schema.command, "media.upload");
        assert!(schema.input_schema.is_object());
        assert!(schema.output_schema.is_object());
        let required = schema.input_schema["required"].as_array().unwrap();
        assert!(
            required.iter().any(|v| v.as_str() == Some("path")),
            "media.upload input schema should require path"
        );
    }

    #[test]
    fn test_media_upload_help() {
        let help = CommandHelp::for_command("media.upload");
        assert_eq!(help.command, "media.upload");
        assert!(!help.exit_codes.is_empty());
        assert!(!help.error_vocabulary.is_empty());
        assert!(!help.examples.is_empty());
        assert!(help.usage.contains("media upload"));
    }

    #[test]
    fn test_commands_list_includes_timeline() {
        let list = CommandsList::new();
        assert!(
            list.commands.iter().any(|c| c.name == "timeline.home"),
            "timeline.home should be in commands list"
        );
        assert!(
            list.commands.iter().any(|c| c.name == "timeline.mentions"),
            "timeline.mentions should be in commands list"
        );
        assert!(
            list.commands.iter().any(|c| c.name == "timeline.user"),
            "timeline.user should be in commands list"
        );

        // timeline commands are read-only (safe) and have cost
        let home_cmd = list
            .commands
            .iter()
            .find(|c| c.name == "timeline.home")
            .unwrap();
        assert_eq!(home_cmd.risk, RiskLevel::Safe);
        assert!(home_cmd.has_cost);

        // timeline.user requires handle argument
        let user_cmd = list
            .commands
            .iter()
            .find(|c| c.name == "timeline.user")
            .unwrap();
        let handle_arg = user_cmd.arguments.iter().find(|a| a.name == "handle");
        assert!(
            handle_arg.is_some(),
            "timeline.user should have handle argument"
        );
        assert!(
            handle_arg.unwrap().required,
            "handle argument should be required"
        );
    }

    #[test]
    fn test_command_schema() {
        let schema = CommandSchema::for_command("commands");
        assert_eq!(schema.command, "commands");
        assert!(schema.input_schema.is_object());
        assert!(schema.output_schema.is_object());
    }

    #[test]
    fn test_timeline_schema() {
        for command in &["timeline.home", "timeline.mentions", "timeline.user"] {
            let schema = CommandSchema::for_command(command);
            assert_eq!(schema.command, *command);
            assert!(schema.input_schema.is_object());
            assert!(schema.output_schema.is_object());
        }

        // timeline.user requires handle
        let user_schema = CommandSchema::for_command("timeline.user");
        let required = user_schema.input_schema["required"].as_array().unwrap();
        assert!(
            required.iter().any(|v| v.as_str() == Some("handle")),
            "timeline.user input schema should require handle"
        );
    }

    #[test]
    fn test_timeline_help() {
        for command in &["timeline.home", "timeline.mentions", "timeline.user"] {
            let help = CommandHelp::for_command(command);
            assert_eq!(help.command, *command);
            assert!(!help.exit_codes.is_empty());
            assert!(!help.error_vocabulary.is_empty());
            assert!(!help.examples.is_empty());
        }
    }

    #[test]
    fn test_command_help() {
        let help = CommandHelp::for_command("commands");
        assert_eq!(help.command, "commands");
        assert!(!help.exit_codes.is_empty());
        assert!(!help.error_vocabulary.is_empty());
        assert!(!help.examples.is_empty());
    }

    // --- Characterization tests: pin the exact output of commands/schema/help ---

    #[test]
    fn test_characterization_commands_list_names() {
        let list = CommandsList::new();
        let names: Vec<&str> = list.commands.iter().map(|c| c.name.as_str()).collect();
        // Pin the complete set of command names so any accidental addition/removal is detected.
        let expected = vec![
            "commands",
            "schema",
            "help",
            "demo-interactive",
            "install-skills",
            "search recent",
            "search users",
            "tweets reply",
            "tweets thread",
            "tweets show",
            "tweets conversation",
            "timeline.home",
            "timeline.mentions",
            "timeline.user",
            "media.upload",
            "tweets like",
            "tweets unlike",
            "tweets retweet",
            "tweets unretweet",
            "bookmarks add",
            "bookmarks remove",
            "bookmarks list",
        ];
        assert_eq!(names, expected, "command list names must not change");
    }

    #[test]
    fn test_characterization_commands_risk_and_cost() {
        let list = CommandsList::new();
        // Sample a few entries to pin their risk/cost metadata.
        let check = |name: &str, risk: RiskLevel, has_cost: bool| {
            let cmd = list
                .commands
                .iter()
                .find(|c| c.name == name)
                .unwrap_or_else(|| panic!("command {} not found", name));
            assert_eq!(cmd.risk, risk, "risk mismatch for {}", name);
            assert_eq!(cmd.has_cost, has_cost, "has_cost mismatch for {}", name);
        };
        check("commands", RiskLevel::Safe, false);
        check("schema", RiskLevel::Safe, false);
        check("help", RiskLevel::Safe, false);
        check("tweets reply", RiskLevel::Medium, true);
        check("tweets retweet", RiskLevel::Medium, true);
        check("media.upload", RiskLevel::Medium, true);
        check("search recent", RiskLevel::Safe, true);
        check("timeline.home", RiskLevel::Safe, true);
    }

    #[test]
    fn test_characterization_schema_for_commands() {
        let schema = CommandSchema::for_command("commands");
        // input schema must have no required fields
        assert!(
            schema.input_schema.get("required").is_none()
                || schema.input_schema["required"]
                    .as_array()
                    .map(|a| a.is_empty())
                    .unwrap_or(true)
        );
        // output schema must wrap commands array
        let data = &schema.output_schema["properties"]["data"]["properties"]["commands"];
        assert_eq!(data["type"].as_str().unwrap(), "array");
    }

    #[test]
    fn test_characterization_schema_for_search_recent() {
        let schema = CommandSchema::for_command("search recent");
        let required = schema.input_schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("query")));
    }

    #[test]
    fn test_characterization_help_exit_codes_count() {
        // All well-known commands share the same 4 exit codes.
        let commands = [
            "commands",
            "schema",
            "help",
            "search recent",
            "tweets reply",
            "timeline.home",
            "media.upload",
            "bookmarks list",
        ];
        for cmd in &commands {
            let help = CommandHelp::for_command(cmd);
            assert_eq!(
                help.exit_codes.len(),
                4,
                "expected 4 exit codes for {}",
                cmd
            );
            assert_eq!(help.exit_codes[0].code, 0);
            assert_eq!(help.exit_codes[1].code, 2);
            assert_eq!(help.exit_codes[2].code, 3);
            assert_eq!(help.exit_codes[3].code, 4);
        }
    }

    #[test]
    fn test_characterization_help_error_vocabulary_codes() {
        let help = CommandHelp::for_command("schema");
        let codes: Vec<&str> = help
            .error_vocabulary
            .iter()
            .map(|e| e.code.as_str())
            .collect();
        let expected = vec![
            "INVALID_ARGUMENT",
            "MISSING_ARGUMENT",
            "UNKNOWN_COMMAND",
            "AUTHENTICATION_FAILED",
            "AUTHORIZATION_FAILED",
            "RATE_LIMIT_EXCEEDED",
            "NETWORK_ERROR",
            "SERVICE_UNAVAILABLE",
            "INTERNAL_ERROR",
            "INTERACTION_REQUIRED",
        ];
        assert_eq!(codes, expected, "error vocabulary codes must not change");
    }

    #[test]
    fn test_characterization_help_usage_strings() {
        let cases = vec![
            ("commands", "xcom-rs commands"),
            ("schema", "xcom-rs schema"),
            ("help", "xcom-rs help"),
            ("tweets reply", "xcom-rs tweets reply"),
            ("timeline.home", "xcom-rs timeline home"),
            ("media.upload", "xcom-rs media upload"),
            ("bookmarks list", "xcom-rs bookmarks list"),
        ];
        for (cmd, prefix) in cases {
            let help = CommandHelp::for_command(cmd);
            assert!(
                help.usage.starts_with(prefix),
                "usage for {} should start with '{}', got '{}'",
                cmd,
                prefix,
                help.usage
            );
        }
    }
}
