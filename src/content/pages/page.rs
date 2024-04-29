use axum::response::{IntoResponse, Response, Html};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{content::Content, filesystem::file::{File, FileError}, program_version};

/// An HTML webpage
/// 
/// CF [crate::content::Content]
/// 
/// The [Page] can be loaded using the method [Page::load_from_file]
/// 
/// A [Page] may also be converted into an Axum HTML response via [Page::into_response]
/// 
/// # Example
/// ```rust
/// use busser::content::pages::page::Page;
/// 
/// pub fn main()
/// {
///     let page = Page::new("/", "/data/path/index.html", 3600);
/// }
/// ``` 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Page
{
    content: Content,
    tag_insertion: bool
}

impl File for Page
{
    fn write_bytes(&self) { self.content.write_bytes(); }
    fn read_bytes(&self) -> Option<Vec<u8>> { self.content.read_bytes() }
    fn read_utf8(&self) -> Option<String> { self.content.read_utf8() }
}

impl Page
{

    pub fn new(uri: &str, disk_path: &str, cache: u16) -> Page
    {
        Page
        {
            content: Content::new(uri, disk_path, cache),
            tag_insertion: true
        }
    }

    pub fn load_from_file(&mut self) -> Result<(), FileError>
    {
        self.content.load_from_file()
    }

    pub fn set_tag_insertion(&mut self, v: bool) { self.tag_insertion = v; }

    pub fn get_uri(&self) -> String { self.content.get_uri() }

    pub fn preview(&self, n: usize) -> String
    {
        self.content.preview(n)
    }

    pub fn utf8_body(&self) -> Result<String, std::string::FromUtf8Error>
    {
        self.content.utf8_body()
    }
}

/// Insert a tag indicating the page was served by busser
/// this may be disabled by launching as busser --no-tagging
pub fn insert_tag(body: String) -> String
{   
    format!("<!--Hosted by Busser {}, https://github.com/JerboaBurrow/Busser-->\n{}", program_version(), body)
}

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let mut string_body = match self.content.utf8_body()
        {
            Ok(s) => s,
            Err(e) => format!("<html><p>Error parsing html body {}</p></html>", e)
        };

        if self.tag_insertion
        {
            string_body = insert_tag(string_body);
        }

        let mut response = Html(string_body).into_response();
        let time_stamp = chrono::offset::Utc::now().to_rfc3339();
        response.headers_mut().insert("date", time_stamp.parse().unwrap());
        response.headers_mut().insert("cache-control", format!("public, max-age={}", self.content.cache_period_seconds).parse().unwrap());
        response
    }
}

pub fn is_page(uri: &str, domain: &str) -> bool
{

    let domain_escaped = domain.replace("https://", "").replace("http://", "").replace(".", r"\.");
    match Regex::new(format!(r"((^|(http)(s|)://){})(/|/[^\.]+|/[^\.]+.html|$)$",domain_escaped).as_str())
    {
        Ok(re) => 
        {
            re.is_match(uri)
        },
        Err(_e) => {false}
    }
}