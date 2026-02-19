//! Shell completion script generation handler.

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::{Cli, ShellChoice};

/// Handle the `completion` command by writing a shell completion script to stdout.
pub fn handle_completion(shell: ShellChoice) -> Result<()> {
    let mut cmd = Cli::command();
    let shell = match shell {
        ShellChoice::Bash => Shell::Bash,
        ShellChoice::Zsh => Shell::Zsh,
        ShellChoice::Fish => Shell::Fish,
    };
    generate(shell, &mut cmd, "xcom-rs", &mut io::stdout());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completion_output(shell: ShellChoice) -> String {
        let mut cmd = Cli::command();
        let clap_shell = match shell {
            ShellChoice::Bash => Shell::Bash,
            ShellChoice::Zsh => Shell::Zsh,
            ShellChoice::Fish => Shell::Fish,
        };
        let mut buf = Vec::new();
        generate(clap_shell, &mut cmd, "xcom-rs", &mut buf);
        String::from_utf8(buf).expect("completion output is valid UTF-8")
    }

    #[test]
    fn test_completion_bash_contains_xcom_rs() {
        let output = completion_output(ShellChoice::Bash);
        assert!(
            output.contains("xcom-rs") || output.contains("xcom_rs"),
            "bash completion should reference the binary name"
        );
    }

    #[test]
    fn test_completion_zsh_contains_xcom_rs() {
        let output = completion_output(ShellChoice::Zsh);
        assert!(
            output.contains("xcom-rs") || output.contains("xcom_rs"),
            "zsh completion should reference the binary name"
        );
    }

    #[test]
    fn test_completion_fish_contains_xcom_rs() {
        let output = completion_output(ShellChoice::Fish);
        assert!(
            output.contains("xcom-rs") || output.contains("xcom_rs"),
            "fish completion should reference the binary name"
        );
    }

    #[test]
    fn test_completion_bash_non_empty() {
        let output = completion_output(ShellChoice::Bash);
        assert!(!output.is_empty(), "bash completion should not be empty");
    }

    #[test]
    fn test_completion_zsh_non_empty() {
        let output = completion_output(ShellChoice::Zsh);
        assert!(!output.is_empty(), "zsh completion should not be empty");
    }

    #[test]
    fn test_completion_fish_non_empty() {
        let output = completion_output(ShellChoice::Fish);
        assert!(!output.is_empty(), "fish completion should not be empty");
    }
}
