use std::str::{from_utf8, FromStr};

use axum::{body::Body, http::{HeaderMap, HeaderName, HeaderValue, Request}, middleware::Next, response::Response};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{config::{Config, RelayConfig, CONFIG_PATH}, util::extract_bytes};

#[derive(Deserialize)]
/// Information to relay a request, name must match a name of
///   a [RelayConfig], see [filter_relay].
struct RelayRequest
{
    body: String,
    headers: Vec<(String, String)>,
    name: String
}

/// Relay a request, if the header "relay" is present
///   and matches some [RelayConfig] in [Config].
/// 
/// The request body should be json deserializable into
///   a [RelayRequest]. Using this proxy one may hide API
///   tokens and urls behind a request sent to Busser. E.g
///   an AWS lambda CRUD API may be called by proxy hiding its
///   url and token.
pub async fn filter_relay
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
        Err(_) => 
        {
            crate::debug(format!("Bad JSON body"), Some("Relay"));
            return Err(StatusCode::BAD_REQUEST)
        }
    };

    let mut relay_headers = HeaderMap::new();
    for (key, value) in req.headers
    {
        relay_headers.insert(HeaderName::from_str(&key).unwrap(),HeaderValue::from_str(&value).unwrap());
    }

    match get_relay_config(req.name)
    {
        None => 
        {
            crate::debug(format!("No matching config"), Some("Relay"));
            return Err(StatusCode::BAD_REQUEST)
        },
        Some(relay) =>
        {
            for (key, value) in relay.headers
            {
                relay_headers.insert(HeaderName::from_str(&key).unwrap(),HeaderValue::from_str(&value).unwrap());
            }

            let client = reqwest::Client::new();
            let response = match client.post(relay.url)
                .headers(relay_headers)
                .body(req.body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(_) => return Err(StatusCode::BAD_REQUEST)
            };

            let response_builder = Response::builder().status(response.status().as_u16());
            let mut headers = HeaderMap::with_capacity(response.headers().len());
            headers.extend(response.headers().into_iter().map(|(name, value)| {
                let name = HeaderName::from_bytes(name.as_ref()).unwrap();
                let value = HeaderValue::from_bytes(value.as_ref()).unwrap();
                (name, value)
            }));
            return match response_builder.body(Body::from_stream(response.bytes_stream())) {
                Ok(resp) => Ok(resp),
                Err(_) => Err(StatusCode::BAD_REQUEST)
            }
        }
    }
}

fn get_relay_config(name: String) -> Option<RelayConfig>
{
    let config = Config::load_or_default(CONFIG_PATH);

    if config.relay.is_some()
    {
        for relay in config.relay.unwrap()
        {
            if relay.name == name
            {
                return Some(relay);
            }
        }
    }

    return None;
}