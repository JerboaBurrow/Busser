use std::{cmp::min, collections::HashMap};

use axum::response::{Html, IntoResponse, Response};
use regex::Regex;
use serde::{Serialize, Deserialize};

/// An non-HTML resource with a uri, byte body, and MIME type
/// 
/// A resource may also be converted into an Axum HTML response via
/// ```rust resource.into_response()```
/// # Example
/// ```rust
/// use busser::resources::resource::Resource;
/// 
/// pub fn main()
/// {
/// 
///     let res = Resource::new
///     (
///         "index.js", 
///         "console.log(\"Hello, World!\")".as_bytes().to_vec(), 
///         "text/javascript"
///     );
/// 
///     println!("{}",res.preview(64));
/// }
/// ``` 
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource
{
    uri: String,
    body: Vec<u8>,
    content_type: String
}

/// Identifies the MIME type by file extension, no attempt is made to verify the file's content
/// 
/// Supported MIME types in Busser, default is ```not-rust "application/octet-stream"```
/// ```rust
/// use std::collections::HashMap;
/// let content_types = HashMap::from
/// ( 
///     [
///         (r"\.txt$", "text/plain"),
///         (r"\.css$", "text/css"),
///         (r"\.csv$", "text/csv"),
///         (r"\.(javascript|js)$", "text/javascript"),
///         (r"\.xml$", "text/xml"),
///         (r"\.gif$", "image/gif"),   
///         (r"\.(jpg|jpeg)$", "image/jpeg"),   
///         (r"\.png$", "image/png"),   
///         (r"\.tiff$", "image/tiff"),      
///         (r"\.ico$", "image/x-icon"),  
///         (r"\.(djvu)|(djv)$", "image/vnd.djvu"),  
///         (r"\.svg$", "image/svg+xml"),
///         (r"\.(mpeg|mpg|mp2|mpe|mpv|m2v)$", "video/mpeg"),    
///         (r"\.(mp4|m4v)$", "video/mp4"),    
///         (r"\.(qt|mov)$", "video/quicktime"),    
///         (r"\.(wmv)$", "video/x-ms-wmv"),    
///         (r"\.(flv|f4v|f4p|f4a|f4b)$", "video/x-flv"),   
///         (r"\.webm$", "video/webm")    
///     ]
/// );
/// ```
pub fn content_type(extension: String) -> &'static str
{
    let content_types = HashMap::from
    ( 
        [
            (r"\.txt$", "text/plain"),
            (r"\.css$", "text/css"),
            (r"\.csv$", "text/csv"),
            (r"\.(javascript|js)$", "text/javascript"),
            (r"\.xml$", "text/xml"),
            (r"\.gif$", "image/gif"),   
            (r"\.(jpg|jpeg)$", "image/jpeg"),   
            (r"\.png$", "image/png"),   
            (r"\.tiff$", "image/tiff"),      
            (r"\.ico$", "image/x-icon"),  
            (r"\.(djvu)|(djv)$", "image/vnd.djvu"),  
            (r"\.svg$", "image/svg+xml"),
            (r"\.(mpeg|mpg|mp2|mpe|mpv|m2v)$", "video/mpeg"),    
            (r"\.(mp4|m4v)$", "video/mp4"),    
            (r"\.(qt|mov)$", "video/quicktime"),    
            (r"\.(wmv)$", "video/x-ms-wmv"),    
            (r"\.(flv|f4v|f4p|f4a|f4b)$", "video/x-flv"),   
            (r"\.webm$", "video/webm")    
        ]
    );

    for (re, content) in content_types
    {
        if Regex::new(re).unwrap().is_match(&extension)
        {
            return content
        }
    }

    "application/octet-stream"
}

impl Resource
{
    pub fn new(uri: &str, body: Vec<u8>, content_type: &str) -> Resource
    {
        Resource { uri: uri.to_string(), body, content_type: content_type.to_string() }
    }

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }

    pub fn get_bytes(&self) -> Vec<u8>
    {
        self.body.clone()
    }

    pub fn preview(&self, n: usize) -> String
    {
        let preview_body = match self.body.len() > 0
        {
            true => self.body[1..min(n, self.body.len())].to_vec(),
            false => vec![]
        };

        format!("uri: {}, type: {}, bytes: {:?} ...", self.get_uri(), self.content_type, preview_body)
    }
}

/// Serves an Html response with the given MIME type
impl IntoResponse for Resource {
    fn into_response(self) -> Response {
        let mut response = Html(self.body).into_response();
        response.headers_mut().insert("content-type", self.content_type.parse().unwrap());
        response
    }
}