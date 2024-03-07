use std::cmp::min;

use axum::response::{IntoResponse, Response, Html};
use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::util::read_file_utf8;

const re_is_page: &str = r"^[^\.]+$|\.html";

/// An HTML webpage with a uri and body
/// 
/// A Page may also be converted into an Axum HTML response via
/// ```rust page.into_response()```
/// # Example
/// ```rust
/// use busser::pages::page::Page;
/// 
/// pub fn main()
/// {
/// 
///     let page = Page::new("index.html", "<p>Welcome!</p>", 3600);
/// 
///     println!("{}",page.preview(64));
/// }
/// ``` 
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Page
{
    uri: String,
    body: String,
    cache_period_seconds: u16
}

impl Page
{
    pub fn new(uri: &str, body: &str, cache: u16) -> Page
    {
        Page { uri: uri.to_string(), body: body.to_string(), cache_period_seconds: cache }
    }

    pub fn from_file(path: String, cache_period_seconds: u16) -> Option<Page>
    {
        match read_file_utf8(&path)
        {
            Some(data) => Some(Page::new(path.as_str(), data.as_str(), cache_period_seconds)),
            None => None
        }
    }

    pub fn error(text: &str) -> Page
    {
        Page::new("/", text, 3600)
    }

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }

    pub fn preview(&self, n: usize) -> String
    {
        format!("uri: {}, body: {} ...", self.get_uri(), self.body[1..min(n, self.body.len())].to_string())
    }

    /// Insert a tag indicating the page was served by busser
    /// this may be disabled by launching as busser --no-tagging
    pub fn insert_tag(&mut self)
    {   
        let head = Regex::new(r"<head>").unwrap();
        let tag = r#"<head><meta name="hostedby" content="Busser, https://github.com/JerboaBurrow/Busser">"#;
        let tag_no_head = r#"<html><head><meta name="hostedby" content="Busser, https://github.com/JerboaBurrow/Busser"></head>"#;
        match head.clone().captures_iter(&self.body).count()
        {
            0 => 
            {
                self.body = self.body.replacen("<html>", tag_no_head, 1);
            },
            _ => 
            {
                self.body = self.body.replacen("<head>", tag, 1);
            }
        }
    }
}

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let mut response = Html(self.body).into_response();
        let time_stamp = chrono::offset::Utc::now().to_rfc3339();
        response.headers_mut().insert("date", time_stamp.parse().unwrap());
        response.headers_mut().insert("cache-control", format!("public, max-age={}", self.cache_period_seconds).parse().unwrap());
        response
    }
}

pub fn is_page(uri: &str) -> bool
{
    match Regex::new(re_is_page)
    {
        Ok(re) => 
        {
            re.is_match(uri)
        },
        Err(_e) => {false}
    }
}