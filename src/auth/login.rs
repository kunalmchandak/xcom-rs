/// OAuth2 login flow implementation
use anyhow::{Context, Result};
use std::io::Write;

use super::storage::AuthStore;
use super::{generate_state, AuthorizeParams, OAuth2Client, OAuth2Credentials, Pkce};

/// Parse authorization code and state from redirect URL
pub fn parse_redirect_url(url: &str) -> Result<(String, String)> {
    let parsed_url = url::Url::parse(url).context("Invalid redirect URL")?;

    let mut code = None;
    let mut state = None;

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            _ => {}
        }
    }

    let code = code.context("Missing 'code' parameter in redirect URL")?;
    let state = state.context("Missing 'state' parameter in redirect URL")?;

    Ok((code, state))
}

/// Perform local server login flow (automatic callback handling)
pub fn local_server_login(
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    scope: String,
    auth_store: &AuthStore,
) -> Result<OAuth2Credentials> {
    use std::sync::{Arc, Mutex};

    let pkce = Pkce::generate()?;
    let state = generate_state();

    let params = AuthorizeParams {
        client_id: client_id.clone(),
        redirect_uri: redirect_uri.clone(),
        scope: scope.clone(),
        state: state.clone(),
        code_challenge: pkce.code_challenge.clone(),
        code_challenge_method: "S256".to_string(),
    };

    let authorize_url = params.build_authorize_url();

    // Parse redirect_uri to get port
    let redirect_url = url::Url::parse(&redirect_uri).context("Failed to parse redirect_uri")?;
    let port = redirect_url.port().unwrap_or(8080);
    let callback_path = redirect_url.path();

    // Start local HTTP server
    let server_addr = format!("127.0.0.1:{}", port);
    let server = tiny_http::Server::http(&server_addr)
        .map_err(|e| anyhow::anyhow!("Failed to start local HTTP server: {}", e))?;

    println!("Starting local callback server on {}", server_addr);
    println!("Please visit the following URL to authorize:");
    println!("{}", authorize_url);
    println!();
    println!("Waiting for authorization callback...");

    // Open browser if possible
    #[cfg(not(test))]
    {
        if let Err(e) = open_browser(&authorize_url) {
            tracing::debug!("Failed to open browser: {}", e);
        }
    }

    // Store authorization response
    let auth_response: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let auth_response_clone = Arc::clone(&auth_response);

    // Handle incoming requests with timeout
    let timeout = std::time::Duration::from_secs(120);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        // Set a short timeout for accept to allow checking the overall timeout
        if let Ok(Some(request)) = server.recv_timeout(std::time::Duration::from_secs(1)) {
            let url_path: &str = request.url();

            if url_path.starts_with(callback_path) {
                // Parse query parameters
                if let Some(_query_start) = url_path.find('?') {
                    let full_url = format!("http://localhost{}", url_path);

                    if let Ok((code, returned_state)) = parse_redirect_url(&full_url) {
                        // Send success response to browser
                        let response_html = r#"
                            <!DOCTYPE html>
                            <html>
                            <head><title>Authorization Successful</title></head>
                            <body>
                                <h1>Authorization Successful!</h1>
                                <p>You can close this window and return to the CLI.</p>
                            </body>
                            </html>
                        "#;

                        let response = tiny_http::Response::from_string(response_html).with_header(
                            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
                                .unwrap(),
                        );
                        let _ = request.respond(response);

                        // Store the authorization response
                        let mut auth_resp = auth_response_clone.lock().unwrap();
                        *auth_resp = Some((code, returned_state));
                        break;
                    }
                }

                // Send error response if parsing failed
                let error_html = r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Authorization Error</title></head>
                    <body>
                        <h1>Authorization Error</h1>
                        <p>Failed to parse authorization callback.</p>
                    </body>
                    </html>
                "#;
                let response = tiny_http::Response::from_string(error_html)
                    .with_status_code(400)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
                            .unwrap(),
                    );
                let _ = request.respond(response);
            } else {
                // Send 404 for other paths
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }

        // Check if we received the authorization response
        let auth_resp = auth_response.lock().unwrap();
        if auth_resp.is_some() {
            break;
        }
    }

    // Extract authorization response
    let auth_resp = auth_response.lock().unwrap();
    let (code, returned_state) = auth_resp
        .as_ref()
        .context("Authorization timeout: no callback received within 2 minutes")?;

    // Verify state
    if returned_state != &state {
        anyhow::bail!("State mismatch: possible CSRF attack");
    }

    // Exchange code for token
    let client = OAuth2Client::new(client_id, client_secret);
    let token_response = client.exchange_code(code, &redirect_uri, &pkce.code_verifier)?;

    // Calculate expires_at
    let expires_at = if let Some(expires_in) = token_response.expires_in {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        Some(now + expires_in as i64)
    } else {
        None
    };

    // Parse scopes
    let scopes = token_response
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect());

    let credentials = OAuth2Credentials {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at,
        scopes,
        auth_mode: "oauth2".to_string(),
    };

    // Save credentials
    auth_store.save_oauth2_credentials(&credentials)?;

    Ok(credentials)
}

/// Open browser to authorization URL (platform-specific)
#[cfg(not(test))]
fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .context("Failed to open browser")?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("Failed to open browser")?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .context("Failed to open browser")?;
    }
    Ok(())
}

/// Perform manual login flow (user copies redirect URL)
pub fn manual_login(
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    scope: String,
    auth_store: &AuthStore,
) -> Result<OAuth2Credentials> {
    let pkce = Pkce::generate()?;
    let state = generate_state();

    let params = AuthorizeParams {
        client_id: client_id.clone(),
        redirect_uri: redirect_uri.clone(),
        scope: scope.clone(),
        state: state.clone(),
        code_challenge: pkce.code_challenge.clone(),
        code_challenge_method: "S256".to_string(),
    };

    let authorize_url = params.build_authorize_url();

    // Display authorization URL
    println!("Please visit the following URL to authorize:");
    println!("{}", authorize_url);
    println!();
    print!("After authorization, paste the redirect URL here: ");
    std::io::stdout().flush()?;

    // Read redirect URL from user
    let mut redirect_response = String::new();
    std::io::stdin().read_line(&mut redirect_response)?;
    let redirect_response = redirect_response.trim();

    // Parse redirect URL
    let (code, returned_state) = parse_redirect_url(redirect_response)?;

    // Verify state
    if returned_state != state {
        anyhow::bail!("State mismatch: possible CSRF attack");
    }

    // Exchange code for token
    let client = OAuth2Client::new(client_id, client_secret);
    let token_response = client.exchange_code(&code, &redirect_uri, &pkce.code_verifier)?;

    // Calculate expires_at
    let expires_at = if let Some(expires_in) = token_response.expires_in {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        Some(now + expires_in as i64)
    } else {
        None
    };

    // Parse scopes
    let scopes = token_response
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect());

    let credentials = OAuth2Credentials {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at,
        scopes,
        auth_mode: "oauth2".to_string(),
    };

    // Save credentials
    auth_store.save_oauth2_credentials(&credentials)?;

    Ok(credentials)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_redirect_url() {
        let url = "http://localhost:8080/callback?code=test_code&state=test_state";
        let (code, state) = parse_redirect_url(url).unwrap();
        assert_eq!(code, "test_code");
        assert_eq!(state, "test_state");
    }

    #[test]
    fn test_parse_redirect_url_with_query_params() {
        let url = "http://localhost:8080/callback?code=test_code&state=test_state&extra=param";
        let (code, state) = parse_redirect_url(url).unwrap();
        assert_eq!(code, "test_code");
        assert_eq!(state, "test_state");
    }

    #[test]
    fn test_parse_redirect_url_missing_code() {
        let url = "http://localhost:8080/callback?state=test_state";
        let result = parse_redirect_url(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'code'"));
    }

    #[test]
    fn test_parse_redirect_url_missing_state() {
        let url = "http://localhost:8080/callback?code=test_code";
        let result = parse_redirect_url(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'state'"));
    }
}
