use std::cmp::min;

use axum::response::{IntoResponse, Response, Html};
use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::util::read_file_utf8;

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

    pub fn from_file(path: String) -> Option<Page>
    {
        match read_file_utf8(&path)
        {
            Some(data) => Some(Page::new(path.as_str(), data.as_str())),
            None => None
        }
    }

    pub fn error(text: &str) -> Page
    {
        Page::new("/", text)
    }

    pub fn get_uri(&self) -> String
    {
        self.uri.clone()
    }

    pub fn preview(&self, n: usize) -> String
    {
        format!("uri: {}, body: {} ...", self.get_uri(), self.body[1..min(n, self.body.len())].to_string())
    }

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
        Html(self.body).into_response()
    }
}