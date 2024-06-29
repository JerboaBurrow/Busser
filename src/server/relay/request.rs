use std::str::{from_utf8, FromStr};

use axum::{body::Body, http::{HeaderMap, HeaderName, HeaderValue, Request}, middleware::Next, response::Response};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::util::extract_bytes;

#[derive(Deserialize)]
struct RelayRequest
{
    body: String,
    headers: Vec<(String, String)>,
    endpoint: String
}

pub async fn filter
    (
        headers: HeaderMap,
        request: Request<axum::body::Body>,
        next: Next
    ) -> Result<Response, StatusCode>
{
    if !headers.contains_key("relay")
    {
        return Ok(next.run(request).await);
    }

    let bytes = match extract_bytes(request).await
    {
        Ok(b) => b,
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
    match response_builder.body(Body::from_stream(response.bytes_stream())) {
        Ok(resp) => Ok(resp),
        Err(_) => Err(StatusCode::BAD_REQUEST)
    }
}