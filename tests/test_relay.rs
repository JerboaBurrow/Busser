mod common;

#[cfg(test)]
mod relay
{
    use axum::{body::{Body, Bytes}, http::{HeaderMap, HeaderValue, Request}, middleware::Next};
    use busser::server::relay::request::{filter_relay, get_request, is_relay};
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_relay()
    {
        let headers = HeaderMap::new();
        assert!(!is_relay(headers));

        let request = Request::new(Body::from(""));
        assert!(get_request(request).await.is_none());
    }
}