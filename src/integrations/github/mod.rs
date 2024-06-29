use std::{collections::HashMap, sync::Arc, time::SystemTime};

use axum::{body::Bytes, extract::State, http::{HeaderMap, Request}, middleware::Next, response::{IntoResponse, Response}};
use regex::Regex;
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, util::{extract_bytes, strip_control_characters}};

use super::git::refresh::GitRefreshTask;

/// If user-agent is GitHub-Hookshot, check if
///  x-github-event is push. If so pull the repo if
///  [crate::config::GitConfig] is not None
pub async fn filter_github
(
    State(repo_lock): State<Arc<Mutex<SystemTime>>>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next
) -> Result<Response, StatusCode>
{
    let config = Config::load_or_default(CONFIG_PATH);
    let remote = match config.git
    {
        Some(git) => git.remote,
        None => {return Ok(next.run(request).await)}
    };
    let token = get_token();
    if token.is_none()
    {
        return Ok(next.run(request).await)
    }
    match is_push(&headers).await
    {
        StatusCode::CONTINUE => Ok(next.run(request).await),
        StatusCode::OK =>
        {
            Ok(handle_push(repo_lock, headers, request, remote, token.unwrap()).await.into_response())
        },
        status => Ok(status.into_response())
    }
}

/// Checks the Github webhook event is authentic and
///   matches remote. If so tries to pull
pub async fn handle_push
(
    repo_lock: Arc<Mutex<SystemTime>>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    remote: String,
    token: String
) -> StatusCode
{
    let bytes = match extract_bytes(request).await
    {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST
    };

    if !is_watched_repo(&bytes, &remote)
    {
        return StatusCode::OK;
    }

    match super::is_authentic
    (
        &headers, "x-hub-signature-256",
        token,
        &bytes
    )
    {
        StatusCode::ACCEPTED =>
        {
            crate::debug("Github push event is authentic".to_string(), Some("GITHUB"));
            pull(repo_lock).await;
            return StatusCode::OK;
        },
        status =>
        {
            crate::debug(format!("Authentication error: {}", status), Some("GITHUB"));
            return status;
        }
    }
}

fn get_token() -> Option<String>
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

/// Check the url in json path ["repository"]["html_url"] or
///   ["repository"]["ssh_url"] of a [Bytes] body matches url
pub fn is_watched_repo(body: &Bytes, url: &str) -> bool
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
    if !parsed_data.contains_key("repository") { return false }
    let repo = match parsed_data["repository"].is_object()
    {
        true => parsed_data["repository"].as_object().unwrap(),
        false => return false
    };
    if repo.contains_key("html_url")
    {
        if repo["html_url"].as_str() == Some(url) { return true }
    }
    if repo.contains_key("ssh_url")
    {
        if repo["ssh_url"].as_str() == Some(url) { return true }
    }
    false
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