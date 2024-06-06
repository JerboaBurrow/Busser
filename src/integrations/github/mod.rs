use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{body::Bytes, extract::State, http::{HeaderMap, Request}, middleware::Next, response::{IntoResponse, Response}};
use openssl::conf;
use regex::Regex;
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, util::strip_control_characters};

use super::git::refresh::GitRefreshTask;

/// If user-agent is GitHub-Hookshot, check if
///  x-github-event is push. If so pull the repo if
///  [crate::config::GitConfig] is not None
pub async fn filter_github<B>
(
    State(repo_lock): State<Arc<Mutex<SystemTime>>>,
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>
) -> Result<Response, StatusCode>
where B: axum::body::HttpBody<Data = Bytes>
{
    let config = Config::load_or_default(CONFIG_PATH);
    let remote = match config.git
    {
        Some(git) => git.remote,
        None => {return Ok(next.run(request).await)}
    };
    match is_push(&headers).await
    {
        StatusCode::CONTINUE => Ok(StatusCode::OK.into_response()),
        StatusCode::OK =>
        {
            let token = get_token();
            if token.is_none()
            {
                return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
            }

            let body = request.into_body();
            let bytes = match body.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => {
                    return Ok(StatusCode::BAD_REQUEST.into_response())
                }
            };

            if !is_watched_repo(&bytes, &remote)
            {
                return Ok(StatusCode::OK.into_response())
            }

            match super::is_authentic
            (
                &headers, "x-hub-signature-256",
                token.unwrap(),
                &bytes
            )
            {
                StatusCode::ACCEPTED =>
                {
                    crate::debug("Github push event is authentic".to_string(), Some("GITHUB"));
                    pull(repo_lock).await;
                    return Ok(StatusCode::OK.into_response())
                },
                status => 
                {
                    crate::debug(format!("Authentication error: {}", status), Some("GITHUB"));
                    return Ok(status.into_response())
                }
            }
        },
        status => Ok(status.into_response())
    }
}

pub fn get_token() -> Option<String>
{
    let config = Config::load_or_default(CONFIG_PATH);
    if config.git.is_some()
    {
        config.git.unwrap().remote_webhook_token
    }
    else
    {
        None
    }
}

/// Perform the pull updating the mutex
async fn pull(repo_lock: Arc<Mutex<SystemTime>>)
{
    let mut lock = repo_lock.lock().await;
    let config = Config::load_or_default(CONFIG_PATH);
    GitRefreshTask::notify_pull(GitRefreshTask::pull(&config), &config).await;
    *lock = SystemTime::now();
}

fn is_watched_repo(body: &Bytes, url: &str) -> bool
{
    let utf8_body = match std::str::from_utf8(&body)
    {
        Ok(s) => s.to_owned(),
        Err(e) => { crate::debug(format!("Error parsing body: {}", e), Some("GITHUB")); return false;}
    };
    let parsed_data: HashMap<String, serde_json::Value> = match serde_json::from_str(&strip_control_characters(utf8_body))
    {
        Ok(d) => d,
        Err(e) => 
        {
            crate::debug(format!("Error parsing body: {}", e), Some("GITHUB"));
            return false;
        }
    };
    parsed_data["repository"]["html_url"].as_str() == Some(url) || parsed_data["repository"]["ssh_url"].as_str() == Some(url)
}

/// Check if the headers conform to a github push webhook event
///  without checking it is legitimate
pub async fn is_push(headers: &HeaderMap) -> StatusCode
{
    if !headers.contains_key("user-agent")
    {
        return StatusCode::CONTINUE
    }

    let user_agent = match std::str::from_utf8(headers["user-agent"].as_bytes())
    {
        Ok(u) => u,
        Err(_) =>
        {
            return StatusCode::CONTINUE
        }
    };

    if Regex::new(r"GitHub-Hookshot").unwrap().captures(user_agent).is_some()
    {
        if !headers.contains_key("x-github-event")
        {
            return StatusCode::BAD_REQUEST;
        }

        match std::str::from_utf8(headers["x-github-event"].as_bytes())
        {
            Ok(s) => 
            {
                if s.to_lowercase() == "push"
                {
                    crate::debug("Recieving github push event".to_string(), Some("GITHUB"));
                    return StatusCode::OK
                }
            }
            Err(e) => 
            {
                crate::debug(format!("Invalid utf8 in x-github-event, {}", e), Some("GITHUB"));
                return StatusCode::BAD_REQUEST;
            }
        }
    }
    StatusCode::CONTINUE
}