use std::str::{from_utf8, FromStr};

use axum::{body::{Body, Bytes}, http::{HeaderMap, HeaderName, HeaderValue, Request}, middleware::Next, response::Response};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
struct RelayRequest
{
    body: String,
    headers: Vec<(String, String)>,
    endpoint: String
}

pub async fn filter<B>
    (
        headers: HeaderMap,
        request: Request<B>,
        next: Next<B>
    ) -> Result<Response, StatusCode>
    where B: axum::body::HttpBody<Data = Bytes>
{
    if !headers.contains_key("relay")
    {
        return Ok(next.run(request).await);
    }

    let body = request.into_body();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    };
    let body = match from_utf8(&bytes)
    {
        Ok(b) => b,
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    };

    let req: RelayRequest = match serde_json::from_str(body)
    {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    };

    let mut relay_headers = HeaderMap::new();
    for (key, value) in req.headers
    {
        relay_headers.insert(HeaderName::from_str(&key).unwrap(),HeaderValue::from_str(&value).unwrap());
    }

    let _: serde::de::IgnoredAny = match serde_json::from_str(&req.body)
    {
        Ok(j) => j,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let client = reqwest::Client::new();

    let response = match client.post(req.endpoint)
        .json(&serde_json::json!(req.body))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    };

    let response_builder = Response::builder().status(response.status().as_u16());
    // Here the mapping of headers is required due to reqwest and axum differ on the http crate versions
    let mut headers = HeaderMap::with_capacity(response.headers().len());
    headers.extend(response.headers().into_iter().map(|(name, value)| {
        let name = HeaderName::from_bytes(name.as_ref()).unwrap();
        let value = HeaderValue::from_bytes(value.as_ref()).unwrap();
        (name, value)
    }));
    response_builder.body(Body::wrap_stream(response.bytes_stream())).
}