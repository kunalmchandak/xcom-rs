use crate::{
    auth::{local_server_login, manual_login, AuthStore, OAuth1aClient, OAuth2Client},
    cli::{AuthCommands, AuthMode, LoginMethod},
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorDetails},
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
        AuthCommands::Login {
            mode,
            method,
            scope,
        } => handle_login(
            mode,
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
    mode: AuthMode,
    method: LoginMethod,
    scope: String,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
    non_interactive: bool,
) -> Result<()> {
    tracing::info!("Executing auth login command with mode: {:?}", mode);

    // Check non-interactive mode
    if non_interactive {
        let error = ErrorDetails::auth_required(
            "Cannot perform interactive login in non-interactive mode",
            vec![
                "Run 'xcom-rs auth login' in an interactive terminal".to_string(),
                "Or set XCOM_RS_BEARER_TOKEN environment variable".to_string(),
            ],
        );
        let envelope = if let Some(meta) = create_meta() {
            Envelope::error_with_meta("auth.login", error, meta)
        } else {
            Envelope::error("auth.login", error)
        };
        print_envelope(&envelope, output_format)?;
        std::process::exit(crate::protocol::ExitCode::AuthenticationError.into());
    }

    match mode {
        AuthMode::OAuth2 => {
            handle_oauth2_login(method, scope, auth_store, create_meta, output_format)
        }
        AuthMode::OAuth1a => handle_oauth1a_login(method, auth_store, create_meta, output_format),
    }
}

fn handle_oauth2_login(
    method: LoginMethod,
    scope: String,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    // Get client credentials from environment
    let client_id = std::env::var("XCOM_RS_CLIENT_ID").unwrap_or_else(|_| {
        eprintln!("Error: XCOM_RS_CLIENT_ID environment variable is required");
        std::process::exit(1);
    });
    let client_secret = std::env::var("XCOM_RS_CLIENT_SECRET").ok();
    let redirect_uri = std::env::var("XCOM_RS_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8080/callback".to_string());

    let creds = match method {
        LoginMethod::Manual => {
            manual_login(client_id, client_secret, redirect_uri, scope, auth_store)?
        }
        LoginMethod::LocalServer => {
            local_server_login(client_id, client_secret, redirect_uri, scope, auth_store)?
        }
    };

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

fn handle_oauth1a_login(
    method: LoginMethod,
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    use crate::auth::OAuth1aCredentials;

    // Get OAuth1.0a credentials from environment
    let consumer_key = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_KEY").unwrap_or_else(|_| {
        eprintln!("Error: XCOM_RS_OAUTH1A_CONSUMER_KEY environment variable is required");
        std::process::exit(1);
    });
    let consumer_secret = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_SECRET").unwrap_or_else(|_| {
        eprintln!("Error: XCOM_RS_OAUTH1A_CONSUMER_SECRET environment variable is required");
        std::process::exit(1);
    });

    let client = OAuth1aClient::new(consumer_key.clone(), consumer_secret.clone());

    let redirect_uri = std::env::var("XCOM_RS_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8080/callback".to_string());

    // Step 1: Request token
    let request_token_response = client.request_token(&redirect_uri)?;

    // Step 2: Get authorization URL
    let auth_url = client.authorization_url(&request_token_response.oauth_token);

    println!("\nPlease visit this URL to authorize the application:\n");
    println!("{}\n", auth_url);

    // Step 3: Get verifier
    let oauth_verifier = match method {
        LoginMethod::Manual => {
            println!("After authorizing, enter the PIN or verifier code:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
        LoginMethod::LocalServer => {
            // Start local server and wait for callback
            use tiny_http::{Response, Server};

            let server = Server::http("127.0.0.1:8080")
                .map_err(|e| anyhow::anyhow!("Failed to start local server: {}", e))?;

            println!("Waiting for authorization callback on http://localhost:8080/callback...");

            let request = server.recv()?;
            let url = request.url();

            // Parse verifier from URL query params
            let verifier = url
                .split('?')
                .nth(1)
                .and_then(|query| {
                    query.split('&').find_map(|param| {
                        let mut parts = param.split('=');
                        if parts.next()? == "oauth_verifier" {
                            parts.next().map(|v| v.to_string())
                        } else {
                            None
                        }
                    })
                })
                .ok_or_else(|| anyhow::anyhow!("No oauth_verifier in callback URL"))?;

            let response =
                Response::from_string("Authorization successful! You can close this window.");
            request.respond(response)?;

            verifier
        }
    };

    // Step 4: Exchange for access token
    let access_token_response = client.access_token(
        &request_token_response.oauth_token,
        &request_token_response.oauth_token_secret,
        &oauth_verifier,
    )?;

    // Save credentials
    let creds = OAuth1aCredentials {
        auth_mode: "oauth1a".to_string(),
        consumer_key,
        consumer_secret,
        access_token: access_token_response.oauth_token,
        access_token_secret: access_token_response.oauth_token_secret,
        scopes: None,
    };

    auth_store.save_oauth1a_credentials(&creds)?;

    #[derive(serde::Serialize)]
    struct LoginSuccess {
        message: String,
        auth_mode: String,
        scopes: Option<Vec<String>>,
    }

    let response = LoginSuccess {
        message: "Successfully authenticated with OAuth1.0a".to_string(),
        auth_mode: "oauth1a".to_string(),
        scopes: None,
    };

    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("auth.login", response, meta)
    } else {
        Envelope::success("auth.login", response)
    };
    print_envelope(&envelope, output_format)
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
