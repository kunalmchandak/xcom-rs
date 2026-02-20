//! Integration tests for HTTP API calls using mockito

mod common;

use mockito::Server;
use xcom_rs::media::commands::MediaClient;
use xcom_rs::tweets::http_client::XApiClient;

#[test]
fn test_create_tweet_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("POST", "/2/tweets")
        .match_header("authorization", "Bearer test_token")
        .match_header("content-type", "application/json")
        .with_status(201)
        .with_body(r#"{"data":{"id":"1234567890","text":"Hello world"}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.create_tweet("Hello world", None);

    mock.assert();
    assert!(result.is_ok());
    let tweet = result.unwrap();
    assert_eq!(tweet.id, "1234567890");
    assert_eq!(tweet.text, Some("Hello world".to_string()));

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_retweet_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let expected_body = r#"{"tweet_id":"999"}"#;
    let mock = server
        .mock("POST", "/2/users/123456/retweets")
        .match_header("authorization", "Bearer test_token")
        .match_body(expected_body)
        .with_status(200)
        .with_body(r#"{"data":{"retweeted":true}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.retweet("123456", "999");

    mock.assert();
    assert!(result.is_ok());
    assert!(result.unwrap());

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_unretweet_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/2/users/123456/retweets/999")
        .match_header("authorization", "Bearer test_token")
        .with_status(200)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.unretweet("123456", "999");

    mock.assert();
    assert!(result.is_ok());
    assert!(result.unwrap());

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_bookmark_add_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let expected_body = r#"{"tweet_id":"999"}"#;
    let mock = server
        .mock("POST", "/2/users/123456/bookmarks")
        .match_header("authorization", "Bearer test_token")
        .match_body(expected_body)
        .with_status(200)
        .with_body(r#"{"data":{"bookmarked":true}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.add_bookmark("123456", "999");

    mock.assert();
    assert!(result.is_ok());
    assert!(result.unwrap());

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_bookmark_remove_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/2/users/123456/bookmarks/999")
        .match_header("authorization", "Bearer test_token")
        .with_status(200)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.remove_bookmark("123456", "999");

    mock.assert();
    assert!(result.is_ok());
    assert!(result.unwrap());

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_media_upload_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("POST", "/2/media/upload")
        .match_header("authorization", "Bearer test_token")
        .with_status(200)
        .with_body(r#"{"data":{"media_id":"media_123456"}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    // Create XMediaClient with test base URL
    let client = xcom_rs::media::XMediaClient::with_base_url(server.url());
    let result = client.upload_bytes(b"test image data", "image/jpeg");

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "media_123456");

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_get_user_id_with_application_only_token_returns_auth_required() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("GET", "/2/users/me")
        .match_header("authorization", "Bearer app_only_token")
        .with_status(403)
        .with_body(r#"{"title":"Forbidden","detail":"When authenticating requests to the Twitter API v2 endpoints, you must use keys and tokens from a Twitter developer App that is attached to a Project. You can create a project via the developer portal. OAuth 2.0 Application-Only is not allowed for this endpoint.","type":"about:blank","status":403}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "app_only_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.get_user_id();

    mock.assert();
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("auth_required"),
        "Expected auth_required error, got: {}",
        error_msg
    );

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_media_upload_auth_error() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("POST", "/2/media/upload")
        .with_status(401)
        .with_body(r#"{"errors":[{"message":"Unauthorized"}]}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = xcom_rs::media::XMediaClient::with_base_url(server.url());
    let result = client.upload_bytes(b"test image data", "image/jpeg");

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let classified = err.downcast_ref::<xcom_rs::tweets::ClassifiedError>();
    assert!(classified.is_some());
    assert_eq!(classified.unwrap().status_code, Some(401));

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_media_upload_service_error() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("POST", "/2/media/upload")
        .with_status(503)
        .with_body(r#"{"errors":[{"message":"Service unavailable"}]}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = xcom_rs::media::XMediaClient::with_base_url(server.url());
    let result = client.upload_bytes(b"test image data", "image/jpeg");

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let classified = err.downcast_ref::<xcom_rs::tweets::ClassifiedError>();
    assert!(classified.is_some());
    assert_eq!(classified.unwrap().status_code, Some(503));

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_create_tweet_rate_limit() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("POST", "/2/tweets")
        .with_status(429)
        .with_body(r#"{"errors":[{"message":"Rate limit exceeded"}]}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.create_tweet("Hello", None);

    mock.assert();
    assert!(result.is_err());
    let err = result.unwrap_err();
    let classified = err.downcast_ref::<xcom_rs::tweets::ClassifiedError>();
    assert!(classified.is_some());
    assert_eq!(classified.unwrap().status_code, Some(429));

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_get_user_id_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("GET", "/2/users/me")
        .match_header("authorization", "Bearer test_token")
        .with_status(200)
        .with_body(r#"{"data":{"id":"123456","username":"testuser"}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.get_user_id();

    mock.assert();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "123456");

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_like_tweet_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let expected_body = r#"{"tweet_id":"999"}"#;
    let mock = server
        .mock("POST", "/2/users/123456/likes")
        .match_header("authorization", "Bearer test_token")
        .match_body(expected_body)
        .with_status(200)
        .with_body(r#"{"data":{"liked":true}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.like_tweet("123456", "999");

    mock.assert();
    assert!(result.is_ok());
    assert!(result.unwrap());

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_unlike_tweet_success() {
    let _guard = common::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/2/users/123456/likes/999")
        .match_header("authorization", "Bearer test_token")
        .with_status(200)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.unlike_tweet("123456", "999");

    mock.assert();
    assert!(result.is_ok());
    assert!(result.unwrap());

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}
