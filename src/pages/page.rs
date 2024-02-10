use axum::response::{IntoResponse, Response, Html};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Page
{
    uri: String,
    body: String
}

impl Page
{
    pub fn new(uri: &str, body: &str) -> Page
    {
        Page { uri: uri.to_string(), body: body.to_string() }
    }

    pub fn error(text: &str) -> Page
    {
        Page::new("/", text)
    }

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }
}

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        Html(self.body).into_response()
    }
}