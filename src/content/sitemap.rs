
use std::{collections::BTreeMap, sync::Arc, time::{Duration, Instant, SystemTime}, vec};
use tokio::sync::Mutex;

use axum::{response::IntoResponse, routing::get, Router};
use chrono::{DateTime, Datelike, Utc};
use indicatif::ProgressBar;
use quick_xml::{events::{BytesText, Event}, Error, Writer};
use regex::Regex;
use crate::{config::read_config, content::{filter::ContentFilter, HasUir}, filesystem::file::{write_file_bytes, File, Observed}, util::format_elapsed};

use crate::server::https::parse_uri;

use super::{get_content, mime_type::Mime, Content};

pub struct ContentTree
{
    uri_stem: String,
    contents: Vec<(Content, Arc<Mutex<bool>>)>,
    children: BTreeMap<String, ContentTree>
}

impl ContentTree
{
    pub fn new(uri_stem: &str) -> ContentTree
    {
        ContentTree { uri_stem: uri_stem.to_string(), contents: vec![], children: BTreeMap::new() }
    }

    fn route(&self, static_router: bool) -> Router
    {
        let mut router: Router<(), axum::body::Body> = Router::new();
        for (mut content, mutex) in self.contents.clone()
        {
            router = router.route
            (
                &content.get_uri(), 
                get(move || async move 
                    {
                        // check if we should attempt a lock
                        if !static_router && content.server_cache_expired() && content.is_stale()
                        {
                            let _ = mutex.lock().await;
                            if content.server_cache_expired() && content.is_stale()
                            {
                                // got the lock, and still stale
                                content.refresh();
                                crate::debug(format!("Refresh called on Content {}", content.get_uri()), None);
                            }
                        }
                        content.into_response()
                    })
            );
        }

        for (_uri, child) in &self.children
        {
            router = router.merge(child.route(static_router));
        }

        router
    }

    pub fn push(&mut self, uri_stem: String, content: Content)
    {
        if uri_stem == "/"
        {
            self.contents.push((content, Arc::new(Mutex::new(false))));
            return;
        }

        match uri_stem.find("/")
        {
            Some(loc) => 
            {
                if loc < uri_stem.len()-1
                {
                    let child_uri_stem = uri_stem[0..(loc+1)].to_string();
                    let reduced_uri_stem = uri_stem[(loc+1)..uri_stem.len()].to_string();

                    if child_uri_stem.is_empty()
                    {
                        crate::debug(format!("{} error pushing content, ended in empty uri.\nnext child: {}\nreduced uri: {}", content.get_uri(), &child_uri_stem, &reduced_uri_stem), None);
                    }
                    else
                    {
                        if !self.children.contains_key(&child_uri_stem)
                        {
                            self.children.insert(child_uri_stem.clone(), ContentTree::new(&reduced_uri_stem.clone()));    
                        }

                        self.children.get_mut(&child_uri_stem).unwrap().push(reduced_uri_stem, content);
                    }
                }
                else
                {
                    crate::debug(format!("{} error pushing content, ended in empty uri.", content.get_uri()), None);
                }
            }
            None =>
            {
                self.contents.push((content, Arc::new(Mutex::new(false))))
            }
        }
    }

    /// Implements writing to an xml conforming to https://www.sitemaps.org/protocol.html
    pub fn to_xml(&self, domain: String) -> Vec<u8>
    {
        if self.contents.is_empty() && self.children.is_empty()
        {
            return vec![];
        }

        let mut buffer = vec![];
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 8);

        if !self.contents.is_empty()
        {
            match writer.create_element("url")
                .write_inner_content::<_, Error>
                (|writer|
                {
                    for (content, _) in &self.contents
                    {
                        if content.get_uri().contains("sitemap.xml")
                        {
                            continue;
                        }
                        if content.get_content_type().is_image()
                        {
                            writer.create_element("image:image").write_inner_content::<_, Error>(|writer|
                                {
                                    writer.create_element("image:loc").write_text_content(BytesText::new(&format!("{}{}",domain, content.get_uri())))?;
                                    Ok(())
                                })?;
                        }
                        else if content.get_content_type().is_video()
                        {
                            writer.create_element("video:video").write_inner_content::<_, Error>(|writer|
                                {
                                    writer.create_element("video:content_loc").write_text_content(BytesText::new(&format!("{}{}",domain, content.get_uri())))?;
                                    writer.create_element("video:publication_date").write_text_content(BytesText::new(&lastmod(content.last_refreshed)))?;
                                    Ok(())
                                })?;
                        }
                        else if content.get_content_type().is_html()
                        {
                            writer.create_element("loc").write_text_content(BytesText::new(&format!("{}{}",domain, content.get_uri())))?;
                            writer.create_element("lastmod").write_text_content(BytesText::new(&lastmod(content.last_refreshed)))?;
                        }
                    }
                    Ok(())
                })
            {
                Ok(_) => 
                {
                    if buffer.len() > 0
                    {
                        buffer.append(&mut "\n".as_bytes().to_vec());
                    }
                },
                Err(e) => {crate::debug(format!("Error {} writing content for uri stem {} of sitemap to xml", e, self.uri_stem), None)}
            }
        }

        for (_uri, child) in &self.children
        {
            buffer.append(&mut child.to_xml(domain.clone()));
        }
        buffer
    }
}

pub struct SiteMap
{
    contents: ContentTree,
    content_path: String,
    domain: String
}

impl SiteMap
{
    pub fn new(domain: String, content_path: String) -> SiteMap
    {
        SiteMap { contents: ContentTree::new("/"), content_path, domain }
    }

    pub fn push(&mut self, content: Content)
    {
        self.contents.push(content.uri.clone(), content);
    }

    pub fn build
    (
        &mut self, 
        browser_cache_period: u16,
        server_cache_period: u16, 
        tag: bool,
        short_urls: bool,
        filter: Option<&ContentFilter>
    )
    {
        let mut tic = Instant::now();
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Detecting site files");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let mut contents = get_content
        (
            &self.content_path,
            &self.content_path,
            Some(server_cache_period),
            Some(browser_cache_period),
            Some(tag)
        );
        spinner.finish();
        spinner.set_message(format!("Detecting site files took {}", format_elapsed(tic)));

        if filter.is_some()
        {
            contents = filter.unwrap().filter::<Content>(contents);
        }
        contents.sort_by_key(|x|x.get_uri());

        let mut no_sitemap = true;
        let mut no_robots = true;

        tic = Instant::now();
        println!("Building sitemap");
        let bar = ProgressBar::new(contents.len() as u64);
        for mut content in contents
        { 
            if content.get_uri().contains("sitemap.xml")
            {
                no_sitemap = false;
            }
            if content.get_uri().contains("robots.txt")
            {
                no_robots = false;
            }
            crate::debug(format!("Attempting to add content {:?}", content.preview(64)), None);
            let path = self.content_path.clone()+"/";
            let uri = parse_uri(content.get_uri(), path);

            match content.load_from_file()
            {
                Ok(()) =>
                {
                    if short_urls && content.get_content_type().is_html()
                    {
                        let extension_regex = Regex::new(r"\.\S+$").unwrap();
                        let short_uri = extension_regex.replacen(&uri, 1, "");

                        crate::debug(format!("Adding content as short url: {}", short_uri), None);

                        let content_short = Content::new(&short_uri, &content.path(), server_cache_period, browser_cache_period, tag);

                        self.contents.push(content_short.uri.clone(), content_short);
                    }
                }
                Err(e) => {crate::debug(format!("Error adding content {}\n{}", content.get_uri(), e), None);}
            }

            self.contents.push(content.uri.clone(), content);
            bar.inc(1);
        }
        bar.finish();
        println!("Building sitemap took {}", format_elapsed(tic));

        if no_robots
        {
            let robots = format!("Sitemap: {}/sitemap.xml", self.domain);
            let path = format!("{}/{}",self.content_path,"robots.txt");
            write_file_bytes(&path, robots.as_bytes());
            let robots = Content::new("/robots.txt",&path, server_cache_period, browser_cache_period, tag);
            self.contents.push(robots.uri.clone(), robots);
            crate::debug(format!("No robots.txt specified, generating robots.txt"), None);
        }

        if no_sitemap
        {
            let path = format!("{}/{}",self.content_path,"sitemap.xml");
            write_file_bytes(&path, &self.to_xml());
            let sitemap = Content::new("/sitemap.xml", &path, server_cache_period, browser_cache_period, tag);
            self.contents.push(sitemap.uri.clone(), sitemap);
            crate::debug(format!("No sitemap.xml specified, generating sitemap.xml"), None);
        }
    }

    /// Implements writing to an xml conforming to https://www.sitemaps.org/protocol.html
    pub fn to_xml(&self) -> Vec<u8>
    {
        let mut buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 4);

        match writer.write_event(Event::Text(BytesText::from_escaped("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")))
        {
            Ok(_) => (),
            Err(e) => {crate::debug(format!("Error {} writing content of sitemap to xml", e), None)}
        }

        match writer.create_element("urlset")
            .with_attributes
            (
                vec!
                [
                    ("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"),
                    ("xmlns:image", "http://www.google.com/schemas/sitemap-image/1.1"),
                    ("xmlns:video", "http://www.google.com/schemas/sitemap-video/1.1")
                ].into_iter()
            )
            .write_inner_content::<_, Error>
            (|writer|
            {
                let mut content_buffer = self.contents.to_xml(self.domain.clone());

                if content_buffer.len() > 0
                {
                    let mut content = "\n".as_bytes().to_vec();
                    content.append(&mut content_buffer);
                    content_buffer = content;
                }
                
                let mut str_content = String::from_utf8(content_buffer)?;
                let lines = str_content.matches("\n").count();
                if lines > 1
                {
                    str_content = str_content.replacen("\n", "\n    ", lines-1);
                }

                writer.write_event(Event::Text(BytesText::from_escaped(str_content)))?;
                Ok(())
            })
        {
            Ok(_) => (),
            Err(e) => {crate::debug(format!("Error {} writing content of sitemap to xml", e), None)}
        }

        buffer
    }
}

impl Into<Router> for SiteMap
{
    fn into(self) -> Router
    {
        let static_router = match read_config()
        {
            Some(config) =>
            {
                match config.content.static_content
                {
                    Some(b) => b,
                    None => false
                }
            },
            None => false
        };

        self.contents.route(static_router)
    }
}

pub fn lastmod(t: SystemTime) -> String
{
    let date: DateTime<Utc> = t.into();
    format!("{}-{:0>2}-{:0>2}",date.year(), date.month(), date.day())
}