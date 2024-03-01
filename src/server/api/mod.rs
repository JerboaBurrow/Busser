pub mod stats;

use axum::{
    body::Bytes, http::{HeaderMap, Request}, response::Response, middleware::Next
};
use reqwest::StatusCode;

/// A trait representing an API request to the server
///  - For example [crate::server::api::stats::StatsDigest]
pub trait ApiRequest
{
    /// Validate a request's hmac given a token read from config.json 
    ///   - See [crate::config::Config] and [crate::web::is_authentic]
    fn is_authentic(headers: HeaderMap, body: Bytes) -> StatusCode;
    /// Deserialise the Bytes body from JSON
    fn deserialise_payload(&mut self, headers: HeaderMap, body: Bytes) -> StatusCode;
    /// Formulate a response form the server returned as a String
    ///   - Also perform any actions inherent to this Api call
    async fn into_response(&self) -> (Option<String>, StatusCode);
    /// Axum middleware to 
    ///     1. check headers for an api request type
    ///     2. authenticate the request (HMAC)
    ///     3. respond to it
    ///     4. continue on to the next reqeust
    async fn filter<B>
    (
        headers: HeaderMap,
        request: Request<B>,
        next: Next<B>
    ) -> Result<Response, StatusCode>
    where B: axum::body::HttpBody<Data = Bytes>;

}