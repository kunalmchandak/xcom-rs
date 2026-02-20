use anyhow::{Context, Result};
use std::path::Path;

use super::models::UploadResult;
use crate::tweets::commands::types::ClassifiedError;

/// Trait for uploading media to the X API.
/// This abstraction allows mocking in tests.
pub trait MediaClient {
    /// Upload raw bytes and return the media ID.
    fn upload_bytes(&self, data: &[u8], mime_type: &str) -> Result<String>;
}

/// Production stub client (env-var driven for tests).
///
/// In a real implementation this would call `POST /2/media/upload`.
/// For now it returns a deterministic fake ID so the CLI is fully wired
/// without requiring live credentials.
pub struct StubMediaClient;

impl MediaClient for StubMediaClient {
    fn upload_bytes(&self, _data: &[u8], _mime_type: &str) -> Result<String> {
        // Check for simulated errors via environment variable (testing hook)
        if let Ok(err) = std::env::var("XCOM_MEDIA_SIMULATE_ERROR") {
            match err.as_str() {
                "auth" => {
                    anyhow::bail!("AuthRequired: media.write scope is required for media upload")
                }
                "server_error" => {
                    anyhow::bail!("ServiceUnavailable: X API returned 503")
                }
                _ => {}
            }
        }

        // Return a deterministic fake media ID
        let media_id = format!("media_{}", uuid::Uuid::new_v4().as_simple());
        Ok(media_id)
    }
}

/// Real X API media client
pub struct XMediaClient {
    base_url: String,
    auth_store: Option<crate::auth::AuthStore>,
}

impl XMediaClient {
    /// Create a new X API media client
    pub fn new() -> Self {
        Self {
            base_url: "https://upload.x.com".to_string(),
            auth_store: crate::auth::AuthStore::with_default_storage().ok(),
        }
    }

    /// Create a client with a custom base URL (for testing)
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            base_url,
            auth_store: crate::auth::AuthStore::with_default_storage().ok(),
        }
    }

    /// Create a client with custom auth_store (for testing)
    pub fn with_auth_store(base_url: String, auth_store: Option<crate::auth::AuthStore>) -> Self {
        Self {
            base_url,
            auth_store,
        }
    }

    /// Get bearer token from environment
    fn get_bearer_token(&self) -> Result<String> {
        std::env::var("XCOM_RS_BEARER_TOKEN")
            .context("XCOM_RS_BEARER_TOKEN not set")?
            .strip_prefix("Bearer ")
            .map(String::from)
            .or_else(|| std::env::var("XCOM_RS_BEARER_TOKEN").ok())
            .context("Failed to parse bearer token")
    }

    /// Resolve OAuth1.0a credentials from auth_store
    fn resolve_oauth1a_credentials(&self) -> Result<Option<crate::auth::OAuth1aCredentials>> {
        if let Some(ref auth_store) = self.auth_store {
            return auth_store.resolve_oauth1a_credentials();
        }
        Ok(None)
    }
}

impl Default for XMediaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaClient for XMediaClient {
    fn upload_bytes(&self, data: &[u8], mime_type: &str) -> Result<String> {
        let url = format!("{}/2/media/upload", self.base_url);

        // Create multipart form data manually
        let boundary = format!("----boundary{}", uuid::Uuid::new_v4().as_simple());
        let mut body = Vec::new();

        // Add media data part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"media\"; filename=\"file\"\r\n",
        );
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        let content_type = format!("multipart/form-data; boundary={}", boundary);

        let mut request = ureq::post(&url).set("Content-Type", &content_type);

        // Try OAuth1.0a first (priority 1)
        if let Ok(Some(oauth1a_creds)) = self.resolve_oauth1a_credentials() {
            let client = crate::auth::OAuth1aClient::new(
                oauth1a_creds.consumer_key.clone(),
                oauth1a_creds.consumer_secret.clone(),
            );

            // Generate OAuth1.0a authorization header
            // Note: multipart/form-data body is NOT included in signature
            let auth_header = client
                .generate_auth_header(
                    &url,
                    "POST",
                    &oauth1a_creds.access_token,
                    &oauth1a_creds.access_token_secret,
                    None,
                )
                .context("Failed to generate OAuth1.0a signature")?;

            request = request.set("Authorization", &auth_header);
        } else {
            // Fall back to Bearer token (priority 2)
            let token = self.get_bearer_token()?;
            request = request.set("Authorization", &format!("Bearer {}", token));
        }

        let response = request.send_bytes(&body);

        match response {
            Ok(resp) => {
                let body = resp
                    .into_string()
                    .context("Failed to read media upload response body")?;

                // Parse the response to get media_id
                let response_json: serde_json::Value =
                    serde_json::from_str(&body).context("Failed to parse media upload response")?;

                let media_id = response_json["data"]["media_id"]
                    .as_str()
                    .or_else(|| response_json["media_id_string"].as_str())
                    .context("Failed to extract media_id from response")?
                    .to_string();

                Ok(media_id)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }
}

/// Arguments for a media upload operation
#[derive(Debug, Clone)]
pub struct UploadArgs {
    /// Filesystem path of the file to upload
    pub path: String,
}

/// Media command handler
pub struct MediaCommand<C: MediaClient> {
    client: C,
}

impl<C: MediaClient> MediaCommand<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// Upload a media file.
    ///
    /// Validates the path before reading the file, then delegates to the
    /// configured [`MediaClient`].
    pub fn upload(&self, args: UploadArgs) -> Result<UploadResult> {
        let path = Path::new(&args.path);

        // Task 2.1 – file existence and readability check
        if !path.exists() {
            anyhow::bail!("InvalidInput: file does not exist: {}", args.path);
        }
        if !path.is_file() {
            anyhow::bail!("InvalidInput: path is not a regular file: {}", args.path);
        }

        let data =
            std::fs::read(path).with_context(|| format!("Failed to read file: {}", args.path))?;

        // Detect MIME type from extension (basic heuristic)
        let mime_type = mime_from_path(path);

        // Task 2.2 – delegate to API client
        let media_id = self
            .client
            .upload_bytes(&data, mime_type)
            .context("Media upload failed")?;

        Ok(UploadResult::new(media_id))
    }
}

/// Infer a MIME type from a file path extension.
fn mime_from_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Mock client that always returns a fixed media_id
    struct MockMediaClient {
        media_id: String,
    }

    impl MockMediaClient {
        fn new(media_id: impl Into<String>) -> Self {
            Self {
                media_id: media_id.into(),
            }
        }
    }

    impl MediaClient for MockMediaClient {
        fn upload_bytes(&self, _data: &[u8], _mime_type: &str) -> Result<String> {
            Ok(self.media_id.clone())
        }
    }

    /// Mock client that always returns an error
    struct FailingMediaClient {
        message: String,
    }

    impl FailingMediaClient {
        fn new(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    impl MediaClient for FailingMediaClient {
        fn upload_bytes(&self, _data: &[u8], _mime_type: &str) -> Result<String> {
            anyhow::bail!("{}", self.message)
        }
    }

    // Task 5.1 – upload success test
    #[test]
    fn test_upload_success_returns_media_id() {
        let client = MockMediaClient::new("fixture_media_id_1234");
        let cmd = MediaCommand::new(client);

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"fake image data").unwrap();

        let args = UploadArgs {
            path: tmp.path().to_str().unwrap().to_string(),
        };

        let result = cmd.upload(args).unwrap();
        assert_eq!(result.media_id, "fixture_media_id_1234");
    }

    // Task 5.1 – upload failure test: file not found
    #[test]
    fn test_upload_nonexistent_file_returns_error() {
        let client = MockMediaClient::new("should_not_be_called");
        let cmd = MediaCommand::new(client);

        let args = UploadArgs {
            path: "/nonexistent/path/image.jpg".to_string(),
        };

        let err = cmd.upload(args).unwrap_err();
        assert!(
            err.to_string().contains("InvalidInput"),
            "Expected InvalidInput error, got: {}",
            err
        );
    }

    // Task 5.1 – upload failure test: client error
    #[test]
    fn test_upload_client_error_propagates() {
        let client = FailingMediaClient::new("AuthRequired: missing scope");
        let cmd = MediaCommand::new(client);

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();

        let args = UploadArgs {
            path: tmp.path().to_str().unwrap().to_string(),
        };

        let err = cmd.upload(args).unwrap_err();
        // The error is wrapped with anyhow context; check the full chain
        let chain = format!("{:#}", err);
        assert!(
            chain.contains("AuthRequired"),
            "Expected AuthRequired in error chain, got: {}",
            chain
        );
    }

    #[test]
    fn test_mime_from_extension() {
        assert_eq!(mime_from_path(Path::new("image.jpg")), "image/jpeg");
        assert_eq!(mime_from_path(Path::new("image.jpeg")), "image/jpeg");
        assert_eq!(mime_from_path(Path::new("image.png")), "image/png");
        assert_eq!(mime_from_path(Path::new("image.gif")), "image/gif");
        assert_eq!(mime_from_path(Path::new("video.mp4")), "video/mp4");
        assert_eq!(
            mime_from_path(Path::new("unknown.bin")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_xmedia_client_oauth1a_upload() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original OAuth1.0a environment variables
        let original_consumer_key = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_KEY").ok();
        let original_consumer_secret = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_SECRET").ok();
        let original_access_token = std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN").ok();
        let original_access_token_secret =
            std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET").ok();

        // Set OAuth1.0a credentials via environment
        std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_KEY", "test_consumer_key");
        std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET", "test_consumer_secret");
        std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN", "test_access_token");
        std::env::set_var(
            "XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET",
            "test_access_token_secret",
        );

        let mut mock_server = mockito::Server::new();
        let _m = mock_server
            .mock("POST", "/2/media/upload")
            .match_header(
                "authorization",
                mockito::Matcher::Regex(r"^OAuth .*oauth_signature=.*".to_string()),
            )
            .with_status(200)
            .with_body(r#"{"data":{"media_id":"test_media_123"}}"#)
            .create();

        let auth_store = crate::auth::AuthStore::new();
        let client = XMediaClient::with_auth_store(mock_server.url(), Some(auth_store));

        let result = client.upload_bytes(b"test image data", "image/jpeg");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_media_123");

        // Restore original environment
        match original_consumer_key {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_KEY", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_CONSUMER_KEY"),
        }
        match original_consumer_secret {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET"),
        }
        match original_access_token {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN"),
        }
        match original_access_token_secret {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET"),
        }
    }
}
