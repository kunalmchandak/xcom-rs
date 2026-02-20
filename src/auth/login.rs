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
