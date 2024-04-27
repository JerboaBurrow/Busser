use axum::response::{Html, IntoResponse, Response};
use serde::{Serialize, Deserialize};

use crate::{content::Content, filesystem::file::FileError};

/// A non-HTML resource
/// 
/// CF [Content]
/// 
/// The resource can be loaded using the method [Resource::load_from_file]
/// 
/// A resource may also be converted into an Axum HTML response via [Resource::into_response]
/// 
/// # Example
/// ```rust
/// use busser::content::resources::resource::Resource;
/// 
/// pub fn main()
/// {
/// 
///     let res = Resource::new
///     (
///         "/index.js", 
///         "/path/to/scripts/", 
///         3600
///     );
///
/// }
/// ``` 
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource
{
    content: Content
}

impl Resource
{
    pub fn new(uri: &str, disk_path: &str, cache: u16) -> Resource
    {
        Resource
        {
            content: Content::new(uri, disk_path, cache),
        }
    }

    pub fn load_from_file(&mut self) -> Result<(), FileError>
    {
        self.content.load_from_file()
    }

    pub fn get_uri(&self) -> String { self.content.get_uri() }

    pub fn preview(&self, n: usize) -> String
    {
        self.content.preview(n)
    }

    pub fn get_content_type(&self) -> String { self.content.content_type.clone() }
}

/// Serves an Html response with the given MIME type
impl IntoResponse for Resource {
    fn into_response(self) -> Response {
        let mut response = Html(self.content.body).into_response();
        response.headers_mut().insert("content-type", self.content.content_type.parse().unwrap());
        let time_stamp = chrono::offset::Utc::now().to_rfc3339();
        response.headers_mut().insert("date", time_stamp.parse().unwrap());
        response.headers_mut().insert("cache-control", format!("public, max-age={}", self.content.cache_period_seconds).parse().unwrap());
        response
    }
}