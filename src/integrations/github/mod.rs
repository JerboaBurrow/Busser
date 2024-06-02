use axum::{body::Bytes, http::{HeaderMap, Request}, middleware::Next, response::{IntoResponse, Response}};
use regex::Regex;
use reqwest::StatusCode;

use crate::config::{Config, CONFIG_PATH};

use super::git::refresh::GitRefreshTask;

/// If user-agent is GitHub-Hookshot, check if
///  x-github-event is push. If so pull the repo if
///  [crate::config::GitConfig] is not None
pub async fn filter_github<B>
(
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>
) -> Result<Response, StatusCode>
where B: axum::body::HttpBody<Data = Bytes>
{
    let user_agent = match std::str::from_utf8(headers["user-agent"].as_bytes())
    {
        Ok(u) => u,
        Err(_) =>
        {
            return Ok(next.run(request).await)
        }
    };

    if Regex::new(r"GitHub-Hookshot").unwrap().captures(user_agent).is_some()
    {
        
        let authentic = is_authentic(&headers, request).await;
        if authentic != StatusCode::ACCEPTED
        {
            return Ok(authentic.into_response());
        }

        crate::debug("Authentic github event".to_string(), Some("GITHUB"));

        if !headers.contains_key("x-github-event")
        {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }

        match std::str::from_utf8(headers["x-github-event"].as_bytes())
        {
            Ok(s) => 
            {
                if s.to_lowercase() == "push"
                {
                    let config = Config::load_or_default(CONFIG_PATH);
                    GitRefreshTask::notify_pull(GitRefreshTask::pull(&config), &config).await;
                }
            },
            Err(e) => 
            {
                crate::debug(format!("Invalid utf8 in x-github-event, {}", e), Some("GITHUB"));
                return Ok(StatusCode::BAD_REQUEST.into_response());
            }
        }

        return Ok(StatusCode::ACCEPTED.into_response());
    }
    else
    {
        return Ok(next.run(request).await)
    }
}

async fn is_authentic<B>(headers: &HeaderMap, request: Request<B>) -> StatusCode
where B: axum::body::HttpBody<Data = Bytes>
{
    let config = Config::load_or_default(CONFIG_PATH);
        
    let token = if config.git.is_some()
    {
        config.git.unwrap().remote_webhook_token
    }
    else
    {
        None
    };

    if token.is_none()
    {
        return StatusCode::METHOD_NOT_ALLOWED;
    }

    let body = request.into_body();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return StatusCode::BAD_REQUEST
        }
    };

    super::is_authentic
    (
        &headers, "x-hub-signature-256", 
        token.unwrap(), 
        &bytes
    )
}