
use std::{collections::BTreeMap, sync::Arc, time::{Duration, Instant, SystemTime}, vec};
use openssl::sha::Sha256;
use tokio::sync::Mutex;

use axum::{response::IntoResponse, routing::get, Router};
use chrono::{DateTime, Datelike, Utc};
use indicatif::ProgressBar;
use quick_xml::{events::{BytesText, Event}, Error, Writer};
use regex::Regex;
use crate::{config::{read_config, Config, CONFIG_PATH}, content::{filter::ContentFilter, HasUir}, filesystem::file::{write_file_bytes, File, Observed}, util::format_elapsed};

use crate::server::https::parse_uri;

use super::{get_content, mime_type::Mime, Content};

/// A tree structure representing a uri stem and content
///  convertable to a [Router] which monitors the content if
///  [crate::config::ContentConfig::static_content] is false. If
///  so and the server cache has expired ([crate::config::ContentConfig::server_cache_period_seconds])
///  then content is automatically refreshed when served
#[derive(Clone)]
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
            content.refresh();
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

    fn calculate_hash(&self, with_bodies: bool) -> Vec<u8>
    {
        let mut sha = Sha256::new();
        let mut content: Vec<Content> = self.contents.clone().into_iter().map(|(x, _)| x).collect();
        content.sort_by(|a, b| a.get_uri().cmp(&b.get_uri()));
        for mut content in content
        {
            sha.update(content.get_uri().as_bytes());
            if with_bodies && content.is_stale() 
            {
                content.refresh();
                sha.update(&content.byte_body());
            }
        }

        for (_, child) in &self.children
        {
            sha.update(&child.calculate_hash(with_bodies));
        }

        sha.finish().to_vec()
    }

    pub fn collect_uris(&self) -> Vec<String>
    {
        let mut content: Vec<Content> = self.contents.clone().into_iter().map(|(x, _)| x).collect();
        content.sort_by(|a, b| a.get_uri().cmp(&b.get_uri()));
        
        let mut uris: Vec<String> = content.into_iter().map(|c| c.get_uri()).collect();
        for (_, child) in &self.children
        {
            uris.append(&mut child.collect_uris());
        }
        uris
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

    /// Implements writing to an xml conforming to <https://www.sitemaps.org/protocol.html>
    ///  with <http://www.google.com/schemas/sitemap-image/1.1> and <http://www.google.com/schemas/sitemap-video/1.1>
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

/// Represents the structure of a site.
///  If no sitemap.xml or robots.txt is present
///  these will be generated by calling [SiteMap::to_xml]
///  and inserting the resulting sitemap.xml
/// 
/// Convertable to a router, see [ContentTree] for dynamic
///  options
#[derive(Clone)]
pub struct SiteMap
{
    contents: ContentTree,
    content_path: String,
    domain: String,
    hash: Vec<u8>
}

impl SiteMap
{
    pub fn new(domain: String, content_path: String) -> SiteMap
    {
        SiteMap { contents: ContentTree::new("/"), content_path, domain, hash: vec![] }
    }

    pub fn from_config(config: &Config, insert_tag: bool, silent: bool) -> SiteMap
    {
        let mut sitemap = SiteMap::new(config.domain.clone(), config.content.path.clone());

        match config.content.ignore_regexes.clone()
        {
            Some(p) => 
            {
                sitemap.build
                (
                    insert_tag, 
                    silent,
                    Some(&ContentFilter::new(p))
                );
            },
            None => 
            {
                sitemap.build
                (
                    insert_tag, 
                    silent,
                    None
                );
            }
        };

        let mut home = Content::new
        (
            "/", 
            &config.content.home.clone(), 
            config.content.server_cache_period_seconds, 
            config.content.browser_cache_period_seconds, 
            insert_tag
        );
        match home.load_from_file()
        {
            Ok(()) =>
            {
                sitemap.push(home);
            },
            Err(e) => {crate::debug(format!("Error serving home page resource {}", e), None);}
        }

        sitemap
    }

    pub fn push(&mut self, content: Content)
    {
        self.contents.push(content.uri.clone(), content);
        self.calculate_hash();
    }

    /// Hash a sitemap by detected uri's
    pub fn get_hash(&self) -> Vec<u8>
    {
        self.hash.clone()
    }

    fn calculate_hash(&mut self)
    {
        self.hash = self.contents.calculate_hash(false);
    }

    pub fn collect_uris(&self) -> Vec<String>
    {
        self.contents.collect_uris()
    }

    /// Searches the content path from [SiteMap::new] for [Content]
    ///  robots.txt and sitemap.xml can be generated and added here
    pub fn build
    (
        &mut self, 
        tag: bool,
        silent: bool,
        filter: Option<&ContentFilter>
    )
    {

        let config = Config::load_or_default(CONFIG_PATH);
        let server_cache_period = config.content.server_cache_period_seconds;
        let browser_cache_period = config.content.browser_cache_period_seconds;
        let short_urls = config.content.allow_without_extension;

        let mut tic = Instant::now();
        let spinner = if !silent
        {
            let spinner = ProgressBar::new_spinner();
            spinner.set_message("Detecting site files");
            spinner.enable_steady_tick(Duration::from_millis(100));
            Some(spinner)
        }
        else
        {
            None
        };

        let mut contents = get_content
        (
            &self.content_path,
            &self.content_path,
            Some(server_cache_period),
            Some(browser_cache_period),
            Some(tag),
            filter
        );

        if !silent
        {
            spinner.as_ref().unwrap().finish();
            spinner.unwrap().set_message(format!("Detecting site files took {}", format_elapsed(tic)));
        }

        contents.sort_by_key(|x|x.get_uri());

        let mut no_robots = true;

        tic = Instant::now();
        let bar = if !silent 
        {
            println!("Building sitemap");
            Some(ProgressBar::new(contents.len() as u64))
        }
        else
        {
            None
        };

        let generate_sitemap = match config.content.generate_sitemap
        {
            Some(b) => b,
            None => true
        };

        for content in contents
        { 
            if content.get_uri().contains("robots.txt")
            {
                no_robots = false;
            }
            if generate_sitemap && content.get_uri().contains("sitemap.xml")
            {
                continue
            }
            crate::debug(format!("Adding content {:?}", content.preview(64)), None);
            let path = self.content_path.clone()+"/";
            let uri = parse_uri(content.get_uri(), path);

            if short_urls && content.get_content_type().is_html()
            {
                let short_uri = Regex::new(r"\.\S+$").unwrap().replacen(&uri, 1, "");
                crate::debug(format!("Adding content as short url: {}", short_uri), None);
                self.contents.push(short_uri.to_string(), Content::new(&short_uri, &content.path(), server_cache_period, browser_cache_period, tag));
            }

            self.contents.push(content.uri.clone(), content);
            if !silent {bar.as_ref().unwrap().inc(1);}
        }
        if !silent
        {
            bar.as_ref().unwrap().finish();
            println!("Building sitemap took {}", format_elapsed(tic));
        }

        if no_robots
        {
            let path = format!("{}/{}",self.content_path,"robots.txt");
            write_file_bytes(&path, format!("Sitemap: {}/sitemap.xml", self.domain).as_bytes());
            self.contents.push("/robots.txt".to_string(), Content::new("/robots.txt",&path, server_cache_period, browser_cache_period, tag));
            crate::debug(format!("No robots.txt specified, generating robots.txt"), None);
        }

        if generate_sitemap
        {
            let path = format!("{}/{}",self.content_path,"sitemap.xml");
            write_file_bytes(&path, &self.to_xml());
            self.contents.push("/sitemap.xml".to_string(), Content::new("/sitemap.xml", &path, server_cache_period, browser_cache_period, tag));
            crate::debug(format!("No sitemap.xml specified, generating sitemap.xml"), None);
        }
        self.calculate_hash();
    }

    /// Implements writing to an xml conforming to <https://www.sitemaps.org/protocol.html>
    ///  with <http://www.google.com/schemas/sitemap-image/1.1> and <http://www.google.com/schemas/sitemap-video/1.1>
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
        let static_router = match read_config(CONFIG_PATH)
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