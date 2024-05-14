use std::{str::from_utf8, sync::Arc};

use axum::{body::Bytes, extract::State, http::{HeaderMap, Request}, middleware::Next, response::{IntoResponse, Response}};
use chrono::DateTime;
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{config::{read_config, CONFIG_PATH}, server::stats::{digest::{digest_message, process_hits}, hits::HitStats}, web::{discord::request::post::post, is_authentic}};

use super::ApiRequest;

/// Payload for [StatsDigest] Api request
///  - ```from_utc```: takes a utc date to compile statistics from
///  - ```to_utc```: takes a utc date to compile statistics to
///  - ```post_discord```: whether to post to dicsord or not
#[derive(Deserialize)]
pub struct StatsDigestPayload
{
    from_utc: Option<String>,
    to_utc: Option<String>,
    post_discord: bool
}

/// Payload for [StatsDigest] Api request, see [StatsDigestPayload]
///  - Takes a utc date to compile statistics from, and a switch to post a discord message
///  - All saved hit statistics after from_utc will be included
pub struct StatsDigest 
{
    payload: StatsDigestPayload
}

impl StatsDigest
{
    pub fn new() -> StatsDigest
    {
        StatsDigest 
        { 
            payload: StatsDigestPayload 
            {
                from_utc: None,
                to_utc: None,
                post_discord: false
            } 
        }
    }
}

impl ApiRequest for StatsDigest
{
    fn is_authentic(headers: HeaderMap, body: Bytes) -> StatusCode
    {

        let config = match read_config(CONFIG_PATH)
        {
            Some(c) => c,
            None =>
            {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

        is_authentic
        (
            headers, 
            "busser-token", 
            config.api_token, 
            body
        )
    }

    fn deserialise_payload(&mut self, _headers: HeaderMap, body: Bytes) -> StatusCode
    {
        
        self.payload = match from_utf8(&body)
        {
            Ok(s) => 
            {
                match serde_json::from_str(s)
                {
                    Ok(p) => p,
                    Err(e) =>
                    {
                        crate::debug(format!("{} deserialising POST payload",e), Some("Stats Digest".to_string()));
                        return StatusCode::BAD_REQUEST
                    }
                }
            }
            Err(e) => 
            {
                crate::debug(format!("{} deserialising POST payload",e), Some("Stats Digest".to_string()));
                return StatusCode::BAD_REQUEST
            }
        };

        StatusCode::OK
    }

    async fn into_response(&self, stats: Option<HitStats>) -> (Option<String>, StatusCode)
    {
        let config = match read_config(CONFIG_PATH)
        {
            Some(c) => c,
            None =>
            {
                return (None, StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let from: Option<DateTime<chrono::Utc>> = match self.payload.from_utc.clone()
        {
            Some(s) =>
            {
                match DateTime::parse_from_rfc3339(&s)
                {
                    Ok(date) => Some(date.into()),
                    Err(e) => 
                    {
                        crate::debug(format!("Error {} parsing from_utc form StatsDigest POST payload",e,), None);
                        return (None, StatusCode::BAD_REQUEST) 
                    }
                }
            },
            None => None
        };

        let to: Option<DateTime<chrono::Utc>> = match self.payload.to_utc.clone()
        {
            Some(s) =>
            {
                match DateTime::parse_from_rfc3339(&s)
                {
                    Ok(date) => Some(date.into()),
                    Err(e) => 
                    {
                        crate::debug(format!("Error {} parsing to_utc form StatsDigest POST payload",e,), None);
                        return (None, StatusCode::BAD_REQUEST) 
                    }
                }
            },
            None => None
        };

        let digest = process_hits(config.stats.path, from,to,config.stats.top_n_digest,stats);
        let msg = digest_message(digest, from, to);

        if self.payload.post_discord
        {
            match post(&config.notification_endpoint, msg.clone()).await
            {
                Ok(_s) => (),
                Err(e) => {crate::debug(format!("Error posting to discord\n{}", e), None);}
            }
        }

        (Some(msg), StatusCode::OK)
    }

    async fn filter<B>
    (
        State(stats): State<Option<Arc<Mutex<HitStats>>>>,
        headers: HeaderMap,
        request: Request<B>,
        next: Next<B>
    ) -> Result<Response, StatusCode>
    where B: axum::body::HttpBody<Data = Bytes>
    {

        if !headers.contains_key("api")
        {
            return Ok(next.run(request).await)
        }

        let api = match std::str::from_utf8(headers["api"].as_bytes())
        {
            Ok(u) => u,
            Err(_) =>
            {
                crate::debug("no/mangled user agent".to_string(), None);
                return Ok(next.run(request).await)
            }
        };

        match api == "StatsDigest"
        {
            true => {},
            false => { return Ok(next.run(request).await) }
        }

        let body = request.into_body();
        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                return Err(StatusCode::BAD_REQUEST)
            }
        };

        match StatsDigest::is_authentic(headers.clone(), bytes.clone())
        {
            StatusCode::ACCEPTED => {},
            e => { return Ok(e.into_response()) }
        }

        let mut response = StatsDigest::new();

        match response.deserialise_payload(headers, bytes)
        {
            StatusCode::OK => {},
            e => { return Ok(e.into_response()) }
        }

        let (result, status) = if stats.is_none()
        {
            response.into_response(None).await
        }
        else
        {
            let stats_unwrapped = stats.unwrap();
            let stats_lock = stats_unwrapped.lock().await;
            let s = stats_lock.to_owned();
            response.into_response(Some(s)).await
        };
        

        match result
        {
            Some(s) => { Ok((s).into_response()) },
            None => { Err(status) }
        }
    }

}
