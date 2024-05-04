use std::cmp::min;
use std::time::SystemTime;

use axum::response::{Html, IntoResponse, Response};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::filesystem::file::{file_hash, File, Observed};
use crate::filesystem::file::{read_file_bytes, read_file_utf8, write_file_bytes, FileError};
use crate::filesystem::folder::{list_dir_by, list_sub_dirs};
use crate::program_version;
use crate::util::{dump_bytes, hash};

use self::mime_type::{Mime, MIME};

pub mod mime_type;
pub mod filter;
pub mod sitemap;

/// Store web content 
/// 
/// CF [crate::content::pages::page::Page] and [crate::content::resources::resource::Resource]
/// 
/// - [Content::uri]                   is the served uri of the content (webaddress)
/// - [Content::body]                  is a byte body
/// - [Content::disk_path]             is a path to the locally stored file on disk representing [Content::body]
/// - [Content::cache_period_seconds]  is the cache-control max-age 
/// 
/// - The body is unpopulated until [Content::load_from_file] is called
/// - The body may be converted to a utf8 string using [Content::utf8_body]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content
{
    uri: String,
    body: Vec<u8>,
    content_type: MIME,
    disk_path: String,
    cache_period_seconds: u16,
    hash: Vec<u8>,
    last_refreshed: SystemTime,
    tag_insertion: bool
}

pub trait HasUir 
{
    fn get_uri(&self) -> String;  
}

impl PartialEq for Content
{
    fn eq(&self, other: &Content) -> bool
    {
        return self.uri == other.uri && self.body == other.body &&
               self.content_type == other.content_type &&
               self.disk_path == other.disk_path &&
               self.cache_period_seconds == other.cache_period_seconds &&
               self.hash == other.hash
    }
}

impl File for Content
{
    fn write_bytes(&self)
    {
        write_file_bytes(&self.disk_path, &self.body);
    }

    fn read_bytes(&self) -> Option<Vec<u8>>
    {
        read_file_bytes(&self.disk_path)
    }

    fn read_utf8(&self) -> Option<String>
    {
        read_file_utf8(&self.disk_path)
    }

    fn path(&self) -> String { self.disk_path.clone() }
}

impl Observed for Content
{
    fn is_stale(&self) -> bool
    {
        // this is 4x slower than using the modified date
        //  but the modified date fails when is_stale is called
        //  very soon after creation/modification, plus may
        //  not be guaranteed cross platform, this is.
        //  We can check 100,000 files in 447 millis
        return file_hash(&self.disk_path) != self.hash
    }

    fn refresh(&mut self)
    {
        let _ = self.load_from_file();
    }

    fn last_refreshed(&self) -> SystemTime 
    {
        self.last_refreshed.clone()
    }
}

impl HasUir for Content
{
    fn get_uri(&self) -> String
    {
        self.uri.clone()
    }
}

impl Content
{
    pub fn new(uri: &str, disk_path: &str, cache: u16, tag_insertion: bool) -> Content
    {
        Content 
        { 
            uri: uri.to_string(), 
            body: vec![], 
            disk_path: disk_path.to_string(), 
            content_type: <MIME as Mime>::infer_mime_type(disk_path),
            cache_period_seconds: cache,
            hash: vec![],
            last_refreshed: SystemTime::now(),
            tag_insertion
        }
    }

    pub fn load_from_file(&mut self) -> Result<(), FileError>
    {
        match self.read_bytes()
        {
            Some(data) => 
            {
                self.body = data.clone();
                self.hash = hash(data);
                self.last_refreshed = SystemTime::now();
                Ok(())
            }
            None => 
            {
                Err(FileError { why: format!("Could not read bytes from {}", self.disk_path)})
            }
        }
    }

    pub fn utf8_body(&self) -> Result<String, std::string::FromUtf8Error>
    {
        String::from_utf8(self.body.clone())
    }

    pub fn get_content_type(&self) -> MIME
    {
        self.content_type.clone()
    }

    pub fn preview(&self, n: usize) -> String
    {
        let preview_body = match self.utf8_body()
        {
            Ok(s) => s[0..min(s.len(), n)].to_string(),
            Err(_e) => 
            {
                dump_bytes(&self.body)[0..min(self.body.len(), n)].to_string()
            }
        };
        format!("uri: {}, body: {} ...", self.get_uri(), preview_body)
    }
}

/// Insert a tag indicating the page was served by busser
/// this may be disabled by launching as busser --no-tagging
pub fn insert_tag(body: String)
 -> String
{   
    format!("<!--Hosted by Busser {}, https://github.com/JerboaBurrow/Busser-->\n{}", program_version(), body)
}

impl IntoResponse for Content {
    fn into_response(self) -> Response {
        
        let mut response = if self.content_type == MIME::TextHtml
        {
            let mut string_body = match self.utf8_body()
            {
                Ok(s) => s,
                Err(e) => format!("<html><p>Error parsing html body {}</p></html>", e)
            };

            if self.tag_insertion
            {
                string_body = insert_tag(string_body);
            }

            Html(string_body).into_response()
        }
        else
        {
            Html(self.body).into_response()
        };

        response.headers_mut()
            .insert("content-type", self.content_type.as_str().parse().unwrap());

        let time_stamp = chrono::offset::Utc::now().to_rfc3339();
        response.headers_mut()
            .insert("date", time_stamp.parse().unwrap());

        response.headers_mut()
            .insert("cache-control", format!("public, max-age={}", self.cache_period_seconds).parse().unwrap());
        
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

pub fn get_content(root: &str, path: &str, cache_period_seconds: Option<u16>, tagging: Option<bool>) -> Vec<Content>
{

    let content_paths = list_dir_by(None, path.to_string());
    let mut contents: Vec<Content> = vec![];
    let tag = match tagging
    {
        Some(b) => b,
        None => false
    };

    let cache = match cache_period_seconds
    {
        Some(p) => p,
        None => 3600
    };

    for content_path in content_paths
    {
        let uri = content_path.clone().replace(root,"");
        contents.push(Content::new(&uri, &content_path, cache, tag));
    }

    let dirs = list_sub_dirs(path.to_string());

    if !dirs.is_empty()
    {
        for dir in dirs
        {
            for resource in get_content(root, &dir, cache_period_seconds, tagging)
            {
                contents.push(resource);
            }
        }
    }

    contents

}