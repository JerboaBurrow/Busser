mod common;

#[cfg(test)]
mod github
{
    use std::{sync::Arc, time::SystemTime};

    use axum::{body::{Body, Bytes}, http::{HeaderMap, HeaderValue, Request}};
    use busser::{integrations::github::{handle_push, is_push, is_watched_repo}, util::dump_bytes};
    use openssl::sha::sha256;
    use reqwest::StatusCode;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_is_push()
    {
        let headers = HeaderMap::new();
        assert_eq!(is_push(&headers).await, StatusCode::CONTINUE);

        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_str("not_github").unwrap());
        assert_eq!(is_push(&headers).await, StatusCode::CONTINUE);

        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_str("GitHub-Hookshot").unwrap());
        assert_eq!(is_push(&headers).await, StatusCode::BAD_REQUEST);

        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_str("GitHub-Hookshot").unwrap());
        headers.insert("x-github-event", HeaderValue::from_str("not_a_push").unwrap());
        assert_eq!(is_push(&headers).await, StatusCode::CONTINUE);

        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_str("GitHub-Hookshot").unwrap());
        headers.insert("x-github-event", HeaderValue::from_str("push").unwrap());
        assert_eq!(is_push(&headers).await, StatusCode::OK);
    }

    #[test]
    fn test_is_watched_repo()
    {
        let body_html = Bytes::from(r#"{"repository": {"html_url": "something"}}"#);
        let body_ssh = Bytes::from(r#"{"repository": {"ssh_url": "something_ssh"}}"#);

        assert!(is_watched_repo(&body_html, "something"));
        assert!(!is_watched_repo(&body_html, "something_else"));
        assert!(!is_watched_repo(&body_ssh, "something"));

        assert!(is_watched_repo(&body_ssh, "something_ssh"));
        assert!(!is_watched_repo(&body_ssh, "something_else"));
        assert!(!is_watched_repo(&body_html, "something_ssh"));

        assert!(!is_watched_repo(&Bytes::from(""), ""));
    }

    #[tokio::test]
    async fn test_handle_push()
    {
        let lock = Arc::new(Mutex::new(SystemTime::now()));
        let headers = HeaderMap::new();
        let request = Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .header("X-Custom-Foo", "Bar")
        .body(Body::empty())
        .unwrap();

        assert_eq!(handle_push(lock, headers, request, "not_a_repo".to_string(), "".to_string()).await, StatusCode::OK);

        let lock = Arc::new(Mutex::new(SystemTime::now()));
        let headers = HeaderMap::new();
        let request = Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .header("X-Custom-Foo", "Bar")
        .body(Body::from(r#"{"repository": {"html_url": "something"}}"#))
        .unwrap();

        assert_eq!(handle_push(lock, headers, request, "something".to_string(), "".to_string()).await, StatusCode::UNAUTHORIZED);

        let lock = Arc::new(Mutex::new(SystemTime::now()));
        let mut headers = HeaderMap::new();
        headers.insert("x-hub-signature-256", HeaderValue::from_str(&dump_bytes(&sha256("NOT VALID".as_bytes()))).unwrap());
        let request = Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .header("X-Custom-Foo", "Bar")
        .body(Body::from(r#"{"repository": {"html_url": "something"}}"#))
        .unwrap();

        assert_eq!(handle_push(lock, headers, request, "something".to_string(), "not_a_token".to_string()).await, StatusCode::UNAUTHORIZED);
    }
}