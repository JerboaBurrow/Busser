use std::cmp::min;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::filesystem::file::{file_hash, File, Observed};
use crate::filesystem::file::{read_file_bytes, read_file_utf8, write_file_bytes, FileError};
use crate::util::{dump_bytes, hash};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content
{
    uri: String,
    body: Vec<u8>,
    content_type: String,
    disk_path: String,
    cache_period_seconds: u16,
    hash: Vec<u8>,
    last_refreshed: SystemTime
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
            cache_period_seconds: cache,
            hash: vec![],
            last_refreshed: SystemTime::now()
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

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }

    pub fn get_last_refreshed(&self) -> SystemTime
    {
        self.last_refreshed.clone()
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