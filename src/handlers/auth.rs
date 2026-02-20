use crate::{
    auth::{manual_login, AuthStore, OAuth2Client},
    cli::{AuthCommands, LoginMethod},
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_auth(
    command: AuthCommands,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
    non_interactive: bool,
) -> Result<()> {
    tracing::info!("Executing auth command");
    match command {
        AuthCommands::Status => handle_status(auth_store, create_meta, output_format),
        AuthCommands::Login { method, scope } => handle_login(
            method,
            scope,
            auth_store,
            create_meta,
            output_format,
            non_interactive,
        ),
        AuthCommands::Logout { revoke } => {
            handle_logout(revoke, auth_store, create_meta, output_format)
        }
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

fn handle_login(
    method: LoginMethod,
    scope: String,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
    non_interactive: bool,
) -> Result<()> {
    tracing::info!("Executing auth login command");

    // Check non-interactive mode
    if non_interactive {
        let error = ErrorDetails::new(
            ErrorCode::AuthRequired,
            "Cannot perform interactive login in non-interactive mode".to_string(),
        );
        let envelope = if let Some(meta) = create_meta() {
            Envelope::error_with_meta("auth.login", error, meta)
        } else {
            Envelope::error("auth.login", error)
        };
        print_envelope(&envelope, output_format)?;
        std::process::exit(crate::protocol::ExitCode::AuthenticationError.into());
    }

    // Get client credentials from environment
    let client_id = std::env::var("XCOM_RS_CLIENT_ID").unwrap_or_else(|_| {
        eprintln!("Error: XCOM_RS_CLIENT_ID environment variable is required");
        std::process::exit(1);
    });
    let client_secret = std::env::var("XCOM_RS_CLIENT_SECRET").ok();
    let redirect_uri = std::env::var("XCOM_RS_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8080/callback".to_string());

    match method {
        LoginMethod::Manual => {
            let creds = manual_login(client_id, client_secret, redirect_uri, scope, auth_store)?;

            #[derive(serde::Serialize)]
            struct LoginSuccess {
                message: String,
                auth_mode: String,
                scopes: Option<Vec<String>>,
            }

            let response = LoginSuccess {
                message: "Successfully authenticated".to_string(),
                auth_mode: creds.auth_mode,
                scopes: creds.scopes,
            };

            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("auth.login", response, meta)
            } else {
                Envelope::success("auth.login", response)
            };
            print_envelope(&envelope, output_format)
        }
        LoginMethod::LocalServer => {
            // TODO: Implement local-server method
            anyhow::bail!("local-server method not yet implemented. Use --method manual instead.");
        }
    }
}

fn handle_logout(
    revoke: bool,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing auth logout command");

    // Revoke token if requested
    if revoke {
        if let Ok(Some(creds)) = auth_store.load_oauth2_credentials() {
            let client_id = std::env::var("XCOM_RS_CLIENT_ID").unwrap_or_else(|_| {
                eprintln!("Warning: XCOM_RS_CLIENT_ID not set, skipping revocation");
                String::new()
            });

            if !client_id.is_empty() {
                let client_secret = std::env::var("XCOM_RS_CLIENT_SECRET").ok();
                let client = OAuth2Client::new(client_id, client_secret);

                tracing::info!("Revoking access token");
                if let Err(e) = client.revoke_token(&creds.access_token) {
                    eprintln!("Warning: Failed to revoke token: {}", e);
                }
            }
        }
    }

    // Delete stored credentials
    auth_store.delete_oauth2_credentials()?;

    #[derive(serde::Serialize)]
    struct LogoutSuccess {
        message: String,
    }

    let response = LogoutSuccess {
        message: "Successfully logged out".to_string(),
    };

    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("auth.logout", response, meta)
    } else {
        Envelope::success("auth.logout", response)
    };
    print_envelope(&envelope, output_format)
}
