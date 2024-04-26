use std::cmp::min;

use serde::{Deserialize, Serialize};

use crate::filesystem::file::File;
use crate::filesystem::file::{read_file_bytes, read_file_utf8, write_file_bytes, FileNotReadError};
use crate::util::dump_bytes;

use self::mime_type::infer_mime_type;

pub mod pages;
pub mod resources;
pub mod mime_type;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content
{
    uri: String,
    body: Vec<u8>,
    content_type: String,
    disk_path: String,
    cache_period_seconds: u16
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
}

impl Content
{
    pub fn new(uri: &str, disk_path: &str, cache: u16) -> Content
    {
        Content 
        { 
            uri: uri.to_string(), 
            body: vec![], 
            disk_path: disk_path.to_string(), 
            content_type: infer_mime_type(disk_path).to_string(),
            cache_period_seconds: cache 
        }
    }

    pub fn load_from_file(&mut self) -> Result<(), FileNotReadError>
    {
        match self.read_bytes()
        {
            Some(data) => {self.body = data; Ok(())}
            None => 
            {
                Err(FileNotReadError { why: format!("Could not read bytes from {}", self.disk_path)})
            }
        }
    }

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }

    pub fn utf8_body(&self) -> Result<String, std::string::FromUtf8Error>
    {
        String::from_utf8(self.body.clone())
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