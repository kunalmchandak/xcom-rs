//! X API client trait and HTTP implementation
//!
//! Provides a unified HTTP client for X API requests with base URL,
//! authentication, and common header handling.

use crate::auth::AuthStore;
use crate::protocol::ErrorDetails;
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::env;

/// Configuration for X API client
#[derive(Debug, Clone)]
pub struct XApiConfig {
    /// Base URL for X API (e.g., "https://api.twitter.com")
    pub base_url: String,
    /// Bearer token for authentication (legacy, use auth_store instead)
    pub bearer_token: Option<String>,
    /// User-Agent header value
    pub user_agent: String,
    /// Auth store for token resolution (OAuth2 + env var)
    pub auth_store: Option<AuthStore>,
}

impl XApiConfig {
    /// Create config from environment variables and AuthStore
    ///
    /// Reads:
    /// - XCOM_RS_API_BASE (default: "https://api.twitter.com")
    /// - Resolves bearer token via AuthStore (env var or OAuth2 credentials)
    pub fn from_env() -> Result<Self> {
        let base_url =
            env::var("XCOM_RS_API_BASE").unwrap_or_else(|_| "https://api.twitter.com".to_string());
        let user_agent = format!(
            "xcom-rs/{} ({})",
            env!("CARGO_PKG_VERSION"),
            env::consts::OS
        );

        let auth_store = AuthStore::with_default_storage()?;

        Ok(Self {
            base_url,
            bearer_token: None,
            user_agent,
            auth_store: Some(auth_store),
        })
    }

    /// Create config with explicit bearer token (for testing and legacy use)
    pub fn new(base_url: String, bearer_token: String) -> Self {
        let user_agent = format!(
            "xcom-rs/{} ({})",
            env!("CARGO_PKG_VERSION"),
            env::consts::OS
        );
        Self {
            base_url,
            bearer_token: Some(bearer_token),
            user_agent,
            auth_store: None,
        }
    }

    /// Resolve bearer token from auth_store or direct token
    fn resolve_bearer_token(&self) -> Result<String> {
        // Try direct token first (for testing/legacy)
        if let Some(ref token) = self.bearer_token {
            return Ok(token.clone());
        }

        // Try auth_store
        if let Some(ref auth_store) = self.auth_store {
            if let Some(token) = auth_store.resolve_token()? {
                return Ok(token);
            }
        }

        anyhow::bail!(
            "No bearer token available. Set XCOM_RS_BEARER_TOKEN or run 'xcom-rs auth login'"
        )
    }
}

/// X API client trait
///
/// Abstracts HTTP communication with X API, allowing for testing with mock implementations.
pub trait XApiClient {
    /// Send a GET request to X API
    ///
    /// Returns the deserialized JSON response or an ErrorDetails on failure.
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ErrorDetails>;

    /// Send a POST request with JSON body to X API
    ///
    /// Returns the deserialized JSON response or an ErrorDetails on failure.
    fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ErrorDetails>;
}

/// HTTP implementation of XApiClient using ureq
pub struct HttpXApiClient {
    config: XApiConfig,
}

impl HttpXApiClient {
    /// Create a new HTTP client with the given configuration
    pub fn new(config: XApiConfig) -> Self {
        Self { config }
    }

    /// Build full URL from path
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }

    /// Create base request with common headers
    fn create_request(&self, method: &str, url: &str) -> Result<ureq::Request, ErrorDetails> {
        let bearer_token = self.config.resolve_bearer_token().map_err(|e| {
            ErrorDetails::new(
                crate::protocol::ErrorCode::AuthRequired,
                format!("Authentication required: {}", e),
            )
        })?;

        Ok(ureq::request(method, url)
            .set("Authorization", &format!("Bearer {}", bearer_token))
            .set("User-Agent", &self.config.user_agent)
            .set("Accept", "application/json"))
    }

    /// Handle response and deserialize JSON or classify error
    fn handle_response<T: DeserializeOwned>(
        &self,
        result: Result<ureq::Response, ureq::Error>,
    ) -> Result<T, ErrorDetails> {
        match result {
            Ok(response) => response.into_json::<T>().map_err(|e| {
                ErrorDetails::new(
                    crate::protocol::ErrorCode::InternalError,
                    format!("Failed to parse JSON response: {}", e),
                )
            }),
            Err(ureq::Error::Status(_, response)) => {
                Err(crate::x_api::classify_response_error(response))
            }
            Err(ureq::Error::Transport(e)) => Err(ErrorDetails::new(
                crate::protocol::ErrorCode::NetworkError,
                format!("Network error: {}", e),
            )),
        }
    }
}

impl XApiClient for HttpXApiClient {
    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ErrorDetails> {
        let url = self.build_url(path);
        let request = self.create_request("GET", &url)?;
        self.handle_response(request.call())
    }

    fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ErrorDetails> {
        let url = self.build_url(path);
        let request = self
            .create_request("POST", &url)?
            .set("Content-Type", "application/json");
        self.handle_response(request.send_json(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = XApiConfig::new("https://test.api".to_string(), "test_token".to_string());
        assert_eq!(config.base_url, "https://test.api");
        assert_eq!(config.bearer_token, Some("test_token".to_string()));
        assert!(config.user_agent.starts_with("xcom-rs/"));
    }

    #[test]
    fn test_build_url() {
        let config = XApiConfig::new("https://api.twitter.com".to_string(), "token".to_string());
        let client = HttpXApiClient::new(config);
        assert_eq!(
            client.build_url("/2/tweets"),
            "https://api.twitter.com/2/tweets"
        );
    }

    #[test]
    fn test_get_request_with_auth_header() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestResponse {
            message: String,
        }

        let mut mock_server = mockito::Server::new();
        let config = XApiConfig::new(mock_server.url(), "test_bearer_token".to_string());
        let client = HttpXApiClient::new(config);

        let _m = mock_server
            .mock("GET", "/test")
            .match_header("authorization", "Bearer test_bearer_token")
            .match_header("user-agent", mockito::Matcher::Any)
            .match_header("accept", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message":"success"}"#)
            .create();

        let result: Result<TestResponse, ErrorDetails> = client.get("/test");
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.message, "success");
    }

    #[test]
    fn test_get_request_401_error() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestResponse {
            message: String,
        }

        let mut mock_server = mockito::Server::new();
        let config = XApiConfig::new(mock_server.url(), "invalid_token".to_string());
        let client = HttpXApiClient::new(config);

        let _m = mock_server.mock("GET", "/test").with_status(401).create();

        let result: Result<TestResponse, ErrorDetails> = client.get("/test");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::protocol::ErrorCode::AuthenticationFailed);
        assert!(!error.is_retryable);
    }

    #[test]
    fn test_post_request_with_json_body() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestRequest {
            text: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct TestResponse {
            id: String,
        }

        let mut mock_server = mockito::Server::new();
        let config = XApiConfig::new(mock_server.url(), "test_token".to_string());
        let client = HttpXApiClient::new(config);

        let _m = mock_server
            .mock("POST", "/tweets")
            .match_header("authorization", "Bearer test_token")
            .match_header("content-type", "application/json")
            .match_body(r#"{"text":"Hello"}"#)
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"123"}"#)
            .create();

        let request_body = TestRequest {
            text: "Hello".to_string(),
        };
        let result: Result<TestResponse, ErrorDetails> = client.post("/tweets", &request_body);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, "123");
    }

    #[test]
    fn test_post_request_429_with_retry() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestRequest {
            text: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct TestResponse {
            id: String,
        }

        let mut mock_server = mockito::Server::new();
        let config = XApiConfig::new(mock_server.url(), "test_token".to_string());
        let client = HttpXApiClient::new(config);

        let _m = mock_server
            .mock("POST", "/tweets")
            .with_status(429)
            .with_header("retry-after", "60")
            .create();

        let request_body = TestRequest {
            text: "Hello".to_string(),
        };
        let result: Result<TestResponse, ErrorDetails> = client.post("/tweets", &request_body);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::protocol::ErrorCode::RateLimitExceeded);
        assert!(error.is_retryable);
        assert_eq!(error.retry_after_ms, Some(60000));
    }

    #[test]
    fn test_network_error_handling() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestResponse {
            message: String,
        }

        // Use an invalid URL to trigger transport error
        let config = XApiConfig::new(
            "http://invalid.invalid.invalid".to_string(),
            "token".to_string(),
        );
        let client = HttpXApiClient::new(config);

        let result: Result<TestResponse, ErrorDetails> = client.get("/test");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::protocol::ErrorCode::NetworkError);
    }
}
