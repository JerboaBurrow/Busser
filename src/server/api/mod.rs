pub mod stats;

use std::sync::Arc;

use axum::{
    body::Bytes, extract::State, http::{HeaderMap, Request}, middleware::Next, response::Response
};
use reqwest::StatusCode;
use tokio::sync::Mutex;

use super::stats::hits::HitStats;

/// A trait representing an API request to the server
///  - For example [crate::server::api::stats::StatsDigest]
pub trait ApiRequest
{
    /// Validate a request's hmac given a token read from config.json
    ///   - See [crate::config::Config] and [crate::integrations::is_authentic]
    fn is_authentic(headers: HeaderMap, body: Bytes) -> StatusCode;
    /// Deserialise the Bytes body from JSON
    fn deserialise_payload(&mut self, headers: HeaderMap, body: Bytes) -> StatusCode;
    /// Formulate a response form the server returned as a String
    ///   - Also perform any actions inherent to this Api call
    fn into_response(&self, stats: Option<HitStats>) -> impl std::future::Future<Output = (Option<String>, StatusCode)> + Send;
    /// Axum middleware to
    ///     1. check headers for an api request type
    ///     2. authenticate the request (HMAC)
    ///     3. respond to it
    ///     4. continue on to the next reqeust
    fn filter
    (
        stats: State<Option<Arc<Mutex<HitStats>>>>,
        headers: HeaderMap,
        request: Request<axum::body::Body>,
        next: Next
    ) -> impl std::future::Future<Output = Result<Response, StatusCode>> + Send;

}