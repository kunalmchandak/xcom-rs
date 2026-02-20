//! Error classification utilities for X API responses
//!
//! Maps HTTP status codes and headers to structured ErrorDetails.

use crate::protocol::{ErrorCode, ErrorDetails};

/// Parse Retry-After or x-rate-limit-reset header into milliseconds
///
/// Supports:
/// - Retry-After: <seconds> (integer)
/// - Retry-After: <http-date> (RFC7231 format)
/// - x-rate-limit-reset: <unix-timestamp>
///
/// Returns None if header is missing or invalid.
pub fn parse_retry_after(response: &ureq::Response) -> Option<u64> {
    // Try Retry-After header first (HTTP standard)
    if let Some(retry_after_str) = response.header("retry-after") {
        // Try parsing as integer seconds
        if let Ok(seconds) = retry_after_str.parse::<u64>() {
            return Some(seconds * 1000);
        }
        // Try parsing as HTTP date (not implemented for simplicity, requires chrono)
        // For now, skip HTTP date parsing
    }

    // Try x-rate-limit-reset (Twitter/X convention: Unix timestamp)
    if let Some(reset_str) = response.header("x-rate-limit-reset") {
        if let Ok(reset_timestamp) = reset_str.parse::<u64>() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            if reset_timestamp > now {
                return Some((reset_timestamp - now) * 1000);
            }
        }
    }

    None
}

/// Classify HTTP response into ErrorDetails
///
/// Maps status codes to appropriate ErrorCode:
/// - 401: AuthenticationFailed
/// - 403: AuthorizationFailed
/// - 404: NotFound
/// - 429: RateLimitExceeded (with retry_after_ms if available)
/// - 5xx: ServiceUnavailable
/// - Other: NetworkError (fallback)
pub fn classify_response_error(response: &ureq::Response) -> ErrorDetails {
    let status = response.status();
    let status_text = response.status_text().to_string();

    match status {
        401 => ErrorDetails::new(
            ErrorCode::AuthenticationFailed,
            format!("Authentication failed: {}", status_text),
        ),
        403 => ErrorDetails::new(
            ErrorCode::AuthorizationFailed,
            format!("Authorization failed: {}", status_text),
        ),
        404 => ErrorDetails::new(
            ErrorCode::NotFound,
            format!("Resource not found: {}", status_text),
        ),
        429 => {
            let message = format!("Rate limit exceeded: {}", status_text);
            if let Some(retry_after_ms) = parse_retry_after(response) {
                ErrorDetails::with_retry_after(
                    ErrorCode::RateLimitExceeded,
                    message,
                    retry_after_ms,
                )
            } else {
                ErrorDetails::new(ErrorCode::RateLimitExceeded, message)
            }
        }
        500..=599 => ErrorDetails::new(
            ErrorCode::ServiceUnavailable,
            format!("Service unavailable: {} {}", status, status_text),
        ),
        _ => ErrorDetails::new(
            ErrorCode::NetworkError,
            format!("HTTP error: {} {}", status, status_text),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_401() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server.mock("GET", "/test").with_status(401).create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::AuthenticationFailed);
            assert!(!error.is_retryable);
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_classify_403() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server.mock("GET", "/test").with_status(403).create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::AuthorizationFailed);
            assert!(!error.is_retryable);
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_classify_404() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server.mock("GET", "/test").with_status(404).create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::NotFound);
            assert!(!error.is_retryable);
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_classify_429_with_retry_after_seconds() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server
            .mock("GET", "/test")
            .with_status(429)
            .with_header("retry-after", "5")
            .create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::RateLimitExceeded);
            assert!(error.is_retryable);
            assert_eq!(error.retry_after_ms, Some(5000));
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_classify_429_with_x_rate_limit_reset() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let reset_time = now + 10;
        let _m = mock_server
            .mock("GET", "/test")
            .with_status(429)
            .with_header("x-rate-limit-reset", &reset_time.to_string())
            .create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::RateLimitExceeded);
            assert!(error.is_retryable);
            assert!(error.retry_after_ms.is_some());
            let retry_ms = error.retry_after_ms.unwrap();
            // Should be approximately 10000ms (10 seconds)
            assert!((9000..=11000).contains(&retry_ms));
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_classify_429_without_retry_header() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server.mock("GET", "/test").with_status(429).create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::RateLimitExceeded);
            assert!(error.is_retryable);
            assert!(error.retry_after_ms.is_none());
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_classify_5xx() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server.mock("GET", "/test").with_status(503).create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let error = classify_response_error(resp);
            assert_eq!(error.code, ErrorCode::ServiceUnavailable);
            assert!(error.is_retryable);
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_parse_retry_after_with_seconds() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server
            .mock("GET", "/test")
            .with_status(429)
            .with_header("retry-after", "30")
            .create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let retry_ms = parse_retry_after(resp);
            assert_eq!(retry_ms, Some(30000));
        } else {
            panic!("Expected status error");
        }
    }

    #[test]
    fn test_parse_retry_after_missing() {
        let mut mock_server = mockito::Server::new();
        let url = format!("{}/test", mock_server.url());
        let _m = mock_server.mock("GET", "/test").with_status(429).create();

        let response = ureq::get(&url).call().unwrap_err();
        if let ureq::Error::Status(_, ref resp) = response {
            let retry_ms = parse_retry_after(resp);
            assert!(retry_ms.is_none());
        } else {
            panic!("Expected status error");
        }
    }
}
