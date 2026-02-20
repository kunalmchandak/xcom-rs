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
}

impl XMediaClient {
    /// Create a new X API media client
    pub fn new() -> Self {
        Self {
            base_url: "https://upload.x.com".to_string(),
        }
    }

    /// Create a client with a custom base URL (for testing)
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
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
}

impl Default for XMediaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaClient for XMediaClient {
    fn upload_bytes(&self, data: &[u8], mime_type: &str) -> Result<String> {
        let token = self.get_bearer_token()?;
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

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", &content_type)
            .send_bytes(&body);

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
}
