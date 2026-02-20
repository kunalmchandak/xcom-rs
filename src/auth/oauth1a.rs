use anyhow::{Context, Result};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;

/// OAuth1.0a client for 3-legged authentication flow
#[derive(Debug, Clone)]
pub struct OAuth1aClient {
    consumer_key: String,
    consumer_secret: String,
    base_url: String,
}

/// OAuth1.0a request token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTokenResponse {
    pub oauth_token: String,
    pub oauth_token_secret: String,
    pub oauth_callback_confirmed: bool,
}

/// OAuth1.0a access token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub oauth_token: String,
    pub oauth_token_secret: String,
}

impl OAuth1aClient {
    /// Create a new OAuth1.0a client
    pub fn new(consumer_key: String, consumer_secret: String) -> Self {
        Self {
            consumer_key,
            consumer_secret,
            base_url: "https://api.x.com".to_string(),
        }
    }

    /// Create a new OAuth1.0a client with custom base URL
    pub fn with_base_url(consumer_key: String, consumer_secret: String, base_url: String) -> Self {
        Self {
            consumer_key,
            consumer_secret,
            base_url,
        }
    }

    /// Request a request token (step 1 of 3-legged flow)
    pub fn request_token(&self, callback_url: &str) -> Result<RequestTokenResponse> {
        let url = format!("{}/oauth/request_token", self.base_url);

        // Build OAuth1.0a authorization header
        let auth_header = self.build_request_token_header(callback_url)?;

        let response = ureq::post(&url)
            .set("Authorization", &auth_header)
            .call()
            .context("Failed to request OAuth1.0a request token")?;

        let body = response
            .into_string()
            .context("Failed to read request token response")?;

        Self::parse_request_token_response(&body)
    }

    /// Get the authorization URL for the user (step 2 of 3-legged flow)
    pub fn authorization_url(&self, oauth_token: &str) -> String {
        format!(
            "{}/oauth/authorize?oauth_token={}",
            self.base_url, oauth_token
        )
    }

    /// Exchange request token for access token (step 3 of 3-legged flow)
    pub fn access_token(
        &self,
        oauth_token: &str,
        oauth_token_secret: &str,
        oauth_verifier: &str,
    ) -> Result<AccessTokenResponse> {
        let url = format!("{}/oauth/access_token", self.base_url);

        // Build OAuth1.0a authorization header
        let auth_header =
            self.build_access_token_header(oauth_token, oauth_token_secret, oauth_verifier)?;

        let response = ureq::post(&url)
            .set("Authorization", &auth_header)
            .call()
            .context("Failed to request OAuth1.0a access token")?;

        let body = response
            .into_string()
            .context("Failed to read access token response")?;

        Self::parse_access_token_response(&body)
    }

    /// Build OAuth1.0a authorization header for request token
    fn build_request_token_header(&self, callback_url: &str) -> Result<String> {
        let url = format!("{}/oauth/request_token", self.base_url);

        let mut params = std::collections::HashMap::new();
        params.insert("oauth_callback", callback_url.to_string());

        let authorization = self.build_oauth_header(&url, "POST", &params, None)?;

        Ok(authorization)
    }

    /// Build OAuth1.0a authorization header for access token
    fn build_access_token_header(
        &self,
        oauth_token: &str,
        oauth_token_secret: &str,
        oauth_verifier: &str,
    ) -> Result<String> {
        let url = format!("{}/oauth/access_token", self.base_url);

        let mut params = std::collections::HashMap::new();
        params.insert("oauth_verifier", oauth_verifier.to_string());

        let authorization = self.build_oauth_header(
            &url,
            "POST",
            &params,
            Some((oauth_token, oauth_token_secret)),
        )?;

        Ok(authorization)
    }

    /// Build OAuth1.0a authorization header
    fn build_oauth_header(
        &self,
        url: &str,
        method: &str,
        extra_params: &std::collections::HashMap<&str, String>,
        token: Option<(&str, &str)>,
    ) -> Result<String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("System time error")?
            .as_secs()
            .to_string();

        let nonce = uuid::Uuid::new_v4().to_string().replace("-", "");

        let mut params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        params.insert("oauth_consumer_key".to_string(), self.consumer_key.clone());
        params.insert("oauth_nonce".to_string(), nonce);
        params.insert(
            "oauth_signature_method".to_string(),
            "HMAC-SHA1".to_string(),
        );
        params.insert("oauth_timestamp".to_string(), timestamp);
        params.insert("oauth_version".to_string(), "1.0".to_string());

        if let Some((oauth_token, _)) = token {
            params.insert("oauth_token".to_string(), oauth_token.to_string());
        }

        for (k, v) in extra_params {
            params.insert(k.to_string(), v.clone());
        }

        // Create signature base string
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| k.as_str());

        let param_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let signature_base = format!(
            "{}&{}&{}",
            method,
            urlencoding::encode(url),
            urlencoding::encode(&param_string)
        );

        // Create signing key
        let signing_key = if let Some((_, token_secret)) = token {
            format!(
                "{}&{}",
                urlencoding::encode(&self.consumer_secret),
                urlencoding::encode(token_secret)
            )
        } else {
            format!("{}&", urlencoding::encode(&self.consumer_secret))
        };

        // Calculate signature
        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
            .context("Failed to create HMAC instance")?;
        mac.update(signature_base.as_bytes());
        let result = mac.finalize();
        let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());

        // Build authorization header (only include oauth_* params, not extra params like oauth_verifier)
        let oauth_signature_key = "oauth_signature".to_string();
        let mut auth_params: Vec<_> = params
            .iter()
            .filter(|(k, _)| k.starts_with("oauth_"))
            .collect();
        auth_params.push((&oauth_signature_key, &signature));
        auth_params.sort_by_key(|(k, _)| k.as_str());

        let auth_string = auth_params
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!("OAuth {}", auth_string))
    }

    /// Parse request token response from URL-encoded format
    fn parse_request_token_response(body: &str) -> Result<RequestTokenResponse> {
        let mut oauth_token = None;
        let mut oauth_token_secret = None;
        let mut oauth_callback_confirmed = false;

        for pair in body.split('&') {
            let mut parts = pair.split('=');
            let key = parts.next().context("Invalid response format")?;
            let value = parts.next().context("Invalid response format")?;

            match key {
                "oauth_token" => oauth_token = Some(urlencoding::decode(value)?.to_string()),
                "oauth_token_secret" => {
                    oauth_token_secret = Some(urlencoding::decode(value)?.to_string())
                }
                "oauth_callback_confirmed" => {
                    oauth_callback_confirmed = value == "true";
                }
                _ => {}
            }
        }

        Ok(RequestTokenResponse {
            oauth_token: oauth_token.context("Missing oauth_token in response")?,
            oauth_token_secret: oauth_token_secret
                .context("Missing oauth_token_secret in response")?,
            oauth_callback_confirmed,
        })
    }

    /// Generate OAuth1.0a authorization header for an API request
    /// This is the main method for generating OAuth1.0a signatures for API calls
    pub fn generate_auth_header(
        &self,
        url: &str,
        method: &str,
        oauth_token: &str,
        oauth_token_secret: &str,
        query_params: Option<&std::collections::HashMap<&str, String>>,
    ) -> Result<String> {
        let params = query_params.cloned().unwrap_or_default();
        self.build_oauth_header(
            url,
            method,
            &params,
            Some((oauth_token, oauth_token_secret)),
        )
    }

    /// Parse access token response from URL-encoded format
    fn parse_access_token_response(body: &str) -> Result<AccessTokenResponse> {
        let mut oauth_token = None;
        let mut oauth_token_secret = None;

        for pair in body.split('&') {
            let mut parts = pair.split('=');
            let key = parts.next().context("Invalid response format")?;
            let value = parts.next().context("Invalid response format")?;

            match key {
                "oauth_token" => oauth_token = Some(urlencoding::decode(value)?.to_string()),
                "oauth_token_secret" => {
                    oauth_token_secret = Some(urlencoding::decode(value)?.to_string())
                }
                _ => {}
            }
        }

        Ok(AccessTokenResponse {
            oauth_token: oauth_token.context("Missing oauth_token in response")?,
            oauth_token_secret: oauth_token_secret
                .context("Missing oauth_token_secret in response")?,
        })
    }

    /// Invalidate OAuth1.0a access token
    pub fn invalidate_token(&self, oauth_token: &str, oauth_token_secret: &str) -> Result<()> {
        let url = format!("{}/oauth/invalidate_token", self.base_url);

        // Build OAuth1.0a authorization header
        let auth_header = self.build_oauth_header(
            &url,
            "POST",
            &std::collections::HashMap::new(),
            Some((oauth_token, oauth_token_secret)),
        )?;

        ureq::post(&url)
            .set("Authorization", &auth_header)
            .call()
            .context("Failed to invalidate OAuth1.0a token")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_token_response() {
        let body =
            "oauth_token=test_token&oauth_token_secret=test_secret&oauth_callback_confirmed=true";
        let response = OAuth1aClient::parse_request_token_response(body).unwrap();

        assert_eq!(response.oauth_token, "test_token");
        assert_eq!(response.oauth_token_secret, "test_secret");
        assert!(response.oauth_callback_confirmed);
    }

    #[test]
    fn test_parse_access_token_response() {
        let body = "oauth_token=access_token&oauth_token_secret=access_secret";
        let response = OAuth1aClient::parse_access_token_response(body).unwrap();

        assert_eq!(response.oauth_token, "access_token");
        assert_eq!(response.oauth_token_secret, "access_secret");
    }

    #[test]
    fn test_authorization_url() {
        let client = OAuth1aClient::new("consumer_key".to_string(), "consumer_secret".to_string());
        let url = client.authorization_url("test_token");

        assert_eq!(
            url,
            "https://api.x.com/oauth/authorize?oauth_token=test_token"
        );
    }

    #[test]
    fn test_request_token_with_mockito() {
        use mockito;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/oauth/request_token")
            .with_status(200)
            .with_body("oauth_token=mock_token&oauth_token_secret=mock_secret&oauth_callback_confirmed=true")
            .create();

        let client = OAuth1aClient::with_base_url(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            server.url(),
        );

        let response = client
            .request_token("http://localhost:8080/callback")
            .unwrap();

        assert_eq!(response.oauth_token, "mock_token");
        assert_eq!(response.oauth_token_secret, "mock_secret");
        assert!(response.oauth_callback_confirmed);

        mock.assert();
    }

    #[test]
    fn test_access_token_with_mockito() {
        use mockito;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/oauth/access_token")
            .with_status(200)
            .with_body("oauth_token=access_token&oauth_token_secret=access_secret")
            .create();

        let client = OAuth1aClient::with_base_url(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            server.url(),
        );

        let response = client
            .access_token("request_token", "request_secret", "verifier")
            .unwrap();

        assert_eq!(response.oauth_token, "access_token");
        assert_eq!(response.oauth_token_secret, "access_secret");

        mock.assert();
    }

    #[test]
    fn test_generate_auth_header() {
        let client = OAuth1aClient::new("consumer_key".to_string(), "consumer_secret".to_string());

        let header = client
            .generate_auth_header(
                "https://api.x.com/2/tweets",
                "POST",
                "oauth_token",
                "oauth_token_secret",
                None,
            )
            .unwrap();

        // Verify the header format
        assert!(header.starts_with("OAuth "));
        assert!(header.contains("oauth_consumer_key="));
        assert!(header.contains("oauth_token="));
        assert!(header.contains("oauth_signature_method=\"HMAC-SHA1\""));
        assert!(header.contains("oauth_signature="));
        assert!(header.contains("oauth_timestamp="));
        assert!(header.contains("oauth_nonce="));
        assert!(header.contains("oauth_version=\"1.0\""));
    }

    #[test]
    fn test_invalidate_token_with_mockito() {
        use mockito;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/oauth/invalidate_token")
            .with_status(200)
            .create();

        let client = OAuth1aClient::with_base_url(
            "consumer_key".to_string(),
            "consumer_secret".to_string(),
            server.url(),
        );

        let result = client.invalidate_token("access_token", "access_secret");
        assert!(result.is_ok());

        mock.assert();
    }
}
