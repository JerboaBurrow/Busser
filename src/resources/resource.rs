use axum::response::{Html, IntoResponse, Response};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource
{
    uri: String,
    body: Vec<u8>
}

impl Resource
{
    pub fn new(uri: &str, body: Vec<u8>) -> Resource
    {
        Resource { uri: uri.to_string(), body }
    }

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }

    pub fn get_bytes(&self) -> Vec<u8>
    {
        self.body.clone()
    }
}

impl IntoResponse for Resource {
    fn into_response(self) -> Response {
        Html(self.body).into_response()
    }
}