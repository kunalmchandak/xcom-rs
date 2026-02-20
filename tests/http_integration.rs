//! Integration tests for HTTP API calls using mockito

use mockito::Server;
use xcom_rs::tweets::http_client::XApiClient;

#[test]
fn test_create_tweet_success() {
    let _guard = xcom_rs::test_utils::env_lock::ENV_LOCK
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
fn test_create_tweet_with_reply() {
    let _guard = xcom_rs::test_utils::env_lock::ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let mut server = Server::new();
    let expected_body = r#"{"text":"Reply text","reply":{"in_reply_to_tweet_id":"999"}}"#;
    let mock = server
        .mock("POST", "/2/tweets")
        .match_header("authorization", "Bearer test_token")
        .match_body(expected_body)
        .with_status(201)
        .with_body(r#"{"data":{"id":"1234567891","text":"Reply text"}}"#)
        .create();

    std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

    let client = XApiClient::with_base_url(server.url());
    let result = client.create_tweet("Reply text", Some("999"));

    mock.assert();
    assert!(result.is_ok());
    let tweet = result.unwrap();
    assert_eq!(tweet.id, "1234567891");

    std::env::remove_var("XCOM_RS_BEARER_TOKEN");
}

#[test]
fn test_create_tweet_rate_limit() {
    let _guard = xcom_rs::test_utils::env_lock::ENV_LOCK
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
    let _guard = xcom_rs::test_utils::env_lock::ENV_LOCK
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
    let _guard = xcom_rs::test_utils::env_lock::ENV_LOCK
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
    let _guard = xcom_rs::test_utils::env_lock::ENV_LOCK
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
