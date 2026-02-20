/// OAuth2 authentication flow implementation
use anyhow::Result;
use sha2::{Digest, Sha256};

/// PKCE (Proof Key for Code Exchange) helper
pub struct Pkce {
    pub code_verifier: String,
    pub code_challenge: String,
}

impl Pkce {
    /// Generate a new PKCE code verifier and challenge (S256 method)
    pub fn generate() -> Result<Self> {
        // Generate a random 128-character code verifier
        use uuid::Uuid;
        let code_verifier = format!(
            "{}{}{}{}",
            Uuid::new_v4().simple(),
            Uuid::new_v4().simple(),
            Uuid::new_v4().simple(),
            Uuid::new_v4().simple()
        );

        // Generate S256 code challenge
        let code_challenge = Self::generate_challenge(&code_verifier)?;

        Ok(Self {
            code_verifier,
            code_challenge,
        })
    }

    /// Generate S256 code challenge from verifier
    fn generate_challenge(verifier: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();

        // Base64url encode (no padding)
        Ok(base64_url_encode(&hash))
    }
}

/// Base64url encoding without padding (RFC 4648 Section 5)
fn base64_url_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    URL_SAFE_NO_PAD.encode(data)
}

/// OAuth2 authorization parameters
pub struct AuthorizeParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

impl AuthorizeParams {
    /// Build authorization URL from parameters
    pub fn build_authorize_url(&self) -> String {
        let base_url = "https://twitter.com/i/oauth2/authorize";

        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method={}",
            base_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&self.scope),
            urlencoding::encode(&self.state),
            urlencoding::encode(&self.code_challenge),
            urlencoding::encode(&self.code_challenge_method),
        )
    }
}

/// Generate a random state parameter for OAuth2 flow
pub fn generate_state() -> String {
    use uuid::Uuid;
    Uuid::new_v4().simple().to_string()
}

/// Token response from OAuth2 token endpoint
#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub scope: Option<String>,
    pub token_type: String,
}

/// OAuth2 client for token operations
pub struct OAuth2Client {
    client_id: String,
    client_secret: Option<String>,
    #[cfg(test)]
    base_url: Option<String>,
}

impl OAuth2Client {
    /// Create a new OAuth2 client
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        Self {
            client_id,
            client_secret,
            #[cfg(test)]
            base_url: None,
        }
    }

    #[cfg(test)]
    fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    #[cfg(test)]
    fn get_token_url(&self) -> String {
        if let Some(ref base) = self.base_url {
            format!("{}/2/oauth2/token", base)
        } else {
            "https://api.x.com/2/oauth2/token".to_string()
        }
    }

    #[cfg(not(test))]
    fn get_token_url(&self) -> String {
        "https://api.x.com/2/oauth2/token".to_string()
    }

    #[cfg(test)]
    fn get_revoke_url(&self) -> String {
        if let Some(ref base) = self.base_url {
            format!("{}/2/oauth2/revoke", base)
        } else {
            "https://api.x.com/2/oauth2/revoke".to_string()
        }
    }

    #[cfg(not(test))]
    fn get_revoke_url(&self) -> String {
        "https://api.x.com/2/oauth2/revoke".to_string()
    }

    /// Exchange authorization code for access token
    pub fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Result<TokenResponse> {
        let token_url = self.get_token_url();

        let mut form_params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
            ("client_id", &self.client_id),
        ];

        let mut request =
            ureq::post(&token_url).set("Content-Type", "application/x-www-form-urlencoded");

        // Use Basic auth if client_secret is provided (confidential client)
        if let Some(ref secret) = self.client_secret {
            use base64::engine::general_purpose::STANDARD;
            use base64::Engine;
            let credentials = format!("{}:{}", self.client_id, secret);
            let encoded = STANDARD.encode(credentials.as_bytes());
            request = request.set("Authorization", &format!("Basic {}", encoded));
        } else {
            // Public client - send client_id in form
            form_params.push(("client_id", &self.client_id));
        }

        let form_body = form_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let response = request
            .send_string(&form_body)
            .map_err(|e| anyhow::anyhow!("Token exchange failed: {}", e))?;

        let token_response: TokenResponse = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;

        Ok(token_response)
    }

    /// Refresh access token using refresh token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let token_url = self.get_token_url();

        let form_params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
        ];

        let mut request =
            ureq::post(&token_url).set("Content-Type", "application/x-www-form-urlencoded");

        // Use Basic auth if client_secret is provided
        if let Some(ref secret) = self.client_secret {
            use base64::engine::general_purpose::STANDARD;
            use base64::Engine;
            let credentials = format!("{}:{}", self.client_id, secret);
            let encoded = STANDARD.encode(credentials.as_bytes());
            request = request.set("Authorization", &format!("Basic {}", encoded));
        }

        let form_body = form_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let response = request
            .send_string(&form_body)
            .map_err(|e| anyhow::anyhow!("Token refresh failed: {}", e))?;

        let token_response: TokenResponse = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;

        Ok(token_response)
    }

    /// Revoke access token
    pub fn revoke_token(&self, token: &str) -> Result<()> {
        let revoke_url = self.get_revoke_url();

        let form_params = [("token", token), ("client_id", &self.client_id)];

        let mut request =
            ureq::post(&revoke_url).set("Content-Type", "application/x-www-form-urlencoded");

        // Use Basic auth if client_secret is provided
        if let Some(ref secret) = self.client_secret {
            use base64::engine::general_purpose::STANDARD;
            use base64::Engine;
            let credentials = format!("{}:{}", self.client_id, secret);
            let encoded = STANDARD.encode(credentials.as_bytes());
            request = request.set("Authorization", &format!("Basic {}", encoded));
        }

        let form_body = form_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        request
            .send_string(&form_body)
            .map_err(|e| anyhow::anyhow!("Token revocation failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = Pkce::generate().unwrap();

        // Code verifier should be 128 characters (4 UUIDs of 32 hex chars each)
        assert_eq!(pkce.code_verifier.len(), 128);

        // Code challenge should be base64url encoded (43 or 44 chars for SHA256)
        assert!(pkce.code_challenge.len() >= 43);
        assert!(pkce.code_challenge.len() <= 44);

        // Should not contain padding
        assert!(!pkce.code_challenge.contains('='));

        // Should only contain base64url characters
        for ch in pkce.code_challenge.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '_',
                "Invalid character in code_challenge: {}",
                ch
            );
        }
    }

    #[test]
    fn test_pkce_challenge_deterministic() {
        let verifier = "test_verifier_123";
        let challenge1 = Pkce::generate_challenge(verifier).unwrap();
        let challenge2 = Pkce::generate_challenge(verifier).unwrap();

        // Same verifier should produce same challenge
        assert_eq!(challenge1, challenge2);
    }

    #[test]
    fn test_pkce_challenge_sha256() {
        // Known test vector
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        let challenge = Pkce::generate_challenge(verifier).unwrap();
        assert_eq!(challenge, expected_challenge);
    }

    #[test]
    fn test_base64_url_encode() {
        let data = b"hello world";
        let encoded = base64_url_encode(data);

        // Should not contain padding
        assert!(!encoded.contains('='));

        // Should only contain URL-safe characters
        for ch in encoded.chars() {
            assert!(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
        }
    }

    #[test]
    fn test_authorize_params_build_url() {
        let params = AuthorizeParams {
            client_id: "test_client_id".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "tweet.read tweet.write users.read".to_string(),
            state: "test_state_123".to_string(),
            code_challenge: "test_challenge".to_string(),
            code_challenge_method: "S256".to_string(),
        };

        let url = params.build_authorize_url();

        // Check base URL
        assert!(url.starts_with("https://twitter.com/i/oauth2/authorize?"));

        // Check all required parameters are present
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
        // URL encoding uses %20 for spaces
        assert!(url.contains("scope=tweet.read%20tweet.write%20users.read"));
        assert!(url.contains("state=test_state_123"));
        assert!(url.contains("code_challenge=test_challenge"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn test_generate_state() {
        let state1 = generate_state();
        let state2 = generate_state();

        // Should be 32 characters (UUID without hyphens)
        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);

        // Should be different
        assert_ne!(state1, state2);

        // Should only contain hex characters
        for ch in state1.chars() {
            assert!(ch.is_ascii_hexdigit());
        }
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "test_access_token",
            "refresh_token": "test_refresh_token",
            "expires_in": 7200,
            "scope": "tweet.read tweet.write",
            "token_type": "bearer"
        }"#;

        let response: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test_access_token");
        assert_eq!(
            response.refresh_token,
            Some("test_refresh_token".to_string())
        );
        assert_eq!(response.expires_in, Some(7200));
        assert_eq!(response.scope, Some("tweet.read tweet.write".to_string()));
        assert_eq!(response.token_type, "bearer");
    }

    #[test]
    fn test_token_response_minimal() {
        let json = r#"{
            "access_token": "test_token",
            "token_type": "bearer"
        }"#;

        let response: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test_token");
        assert_eq!(response.refresh_token, None);
        assert_eq!(response.expires_in, None);
        assert_eq!(response.scope, None);
    }

    #[test]
    fn test_oauth2_client_exchange_code_mockito() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/2/oauth2/token")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=authorization_code".to_string()),
                mockito::Matcher::Regex("code=test_code".to_string()),
                mockito::Matcher::Regex("code_verifier=test_verifier".to_string()),
                mockito::Matcher::Regex("client_id=test_client_id".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "new_access_token",
                "refresh_token": "new_refresh_token",
                "expires_in": 7200,
                "token_type": "bearer"
            }"#,
            )
            .create();

        let client =
            OAuth2Client::new("test_client_id".to_string(), None).with_base_url(server.url());

        let result = client
            .exchange_code("test_code", "http://localhost/callback", "test_verifier")
            .unwrap();

        assert_eq!(result.access_token, "new_access_token");
        assert_eq!(result.refresh_token, Some("new_refresh_token".to_string()));
        assert_eq!(result.expires_in, Some(7200));

        mock.assert();
    }

    #[test]
    fn test_oauth2_client_refresh_token_mockito() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/2/oauth2/token")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=refresh_token".to_string()),
                mockito::Matcher::Regex("refresh_token=old_refresh_token".to_string()),
                mockito::Matcher::Regex("client_id=test_client_id".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "new_access_token",
                "refresh_token": "new_refresh_token",
                "expires_in": 7200,
                "token_type": "bearer"
            }"#,
            )
            .create();

        let client =
            OAuth2Client::new("test_client_id".to_string(), None).with_base_url(server.url());

        let result = client.refresh_token("old_refresh_token").unwrap();

        assert_eq!(result.access_token, "new_access_token");
        assert_eq!(result.refresh_token, Some("new_refresh_token".to_string()));

        mock.assert();
    }

    #[test]
    fn test_oauth2_client_revoke_token_mockito() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/2/oauth2/revoke")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("token=test_token".to_string()),
                mockito::Matcher::Regex("client_id=test_client_id".to_string()),
            ]))
            .with_status(200)
            .create();

        let client =
            OAuth2Client::new("test_client_id".to_string(), None).with_base_url(server.url());

        client.revoke_token("test_token").unwrap();

        mock.assert();
    }
}
