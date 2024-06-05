mod common;

#[cfg(test)]
mod github
{
    use axum::http::{HeaderMap, HeaderValue};
    use busser::integrations::github::is_push;
    use reqwest::StatusCode;

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
}