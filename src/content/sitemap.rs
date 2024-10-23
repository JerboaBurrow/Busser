
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

use super::{get_content, mime_type::{Mime, MIME}, Content};

/// A tree structure representing a uri stem and content
///  convertable to a [Router] which monitors the content if
///  [crate::config::ContentConfig::static_content] is false. If
///  so and the server cache has expired ([crate::config::ContentConfig::server_cache_period_seconds])
///  then content is automatically refreshed when served
#[derive(Clone)]
pub struct ContentTree
{
    uri_stem: String,
    contents: BTreeMap<String, Arc<Mutex<Content>>>,
    content_types: BTreeMap<String, MIME>,
    children: BTreeMap<String, ContentTree>,
    sitmap_content: bool
}

impl ContentTree
{
    pub fn new(uri_stem: &str) -> ContentTree
    {
        ContentTree { uri_stem: uri_stem.to_string(), contents: BTreeMap::new(), children: BTreeMap::new(), sitmap_content: false, content_types: BTreeMap::new() }
    }

    fn collect(&self) -> Vec<Arc<Mutex<Content>>>
    {
        let mut contents: Vec<Arc<Mutex<Content>>> = self.contents.values().cloned().collect();
        for (_, child) in &self.children
        {
            contents.append(&mut child.collect());
        }
        contents
    }

    /// Load all content
    pub async fn refresh_all(&self)
    {
        for content in self.collect()
        {
            content.lock().await.refresh();
        }
    }

    /// Build a [Router] to serve the content. If static_router then
    ///   content is never refreshed within the router
    fn route(&self, static_router: bool) -> Router
    {
        let mut router = Router::new();
        for (uri, content) in self.contents.clone()
        {
            router = router.route
            (
                &uri,
                get(move || async move
                    {
                        let mut content = content.lock().await;
                        if !static_router && content.server_cache_expired() && content.is_stale()
                        {
                            content.refresh();
                            crate::debug(format!("Refresh called on Content {}", content.get_uri()), None);
                        }
                        content.clone().into_response()
                    })
            );
        }

        for (_uri, child) in &self.children
        {
            router = router.merge(child.route(static_router));
        }

        router
    }

    /// [Sha256] the uri's of all content in [ContentTree::contents]
    ///   and in children
    fn calculate_path_hash(&self) -> Vec<u8>
    {
        let mut sha = Sha256::new();
        for (uri, _) in self.contents.clone().into_iter()
        {
            sha.update(uri.as_bytes());
        }

        for (_, child) in &self.children
        {
            sha.update(&child.calculate_path_hash());
        }

        sha.finish().to_vec()
    }

    /// List all uris
    pub fn collect_uris(&self) -> Vec<String>
    {
        let mut uris: Vec<String> = self.contents.keys().cloned().collect();
        for (_, child) in &self.children
        {
            uris.append(&mut child.collect_uris());
        }
        uris
    }

    /// Push some content into [ContentTree::contents]. Each are
    ///   grouped by a path, uri_stem.
    pub fn push(&mut self, uri_stem: String, content: Content)
    {
        if uri_stem == "/"
        {
            if content.content_type.in_sitemap() { self.sitmap_content = true; }
            self.content_types.insert(content.get_uri(), content.content_type);
            self.contents.insert(content.get_uri(),Arc::new(Mutex::new(content)));

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
                if content.content_type.in_sitemap() { self.sitmap_content = true; }
                self.content_types.insert(content.get_uri(), content.content_type);
                self.contents.insert(content.get_uri(), Arc::new(Mutex::new(content)));
            }
        }
    }

    /// If [ContentTree::contents] has any html, image, or video content
    pub fn has_sitemap_content(&self) -> bool
    {
        (!self.contents.is_empty()) && self.sitmap_content
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

        if self.has_sitemap_content()
        {
            match writer.create_element("url")
                .write_inner_content::<_, Error>
                (|writer|
                {
                    for (uri, content) in &self.content_types
                    {
                        if uri.contains("sitemap.xml")
                        {
                            continue;
                        }
                        if content.is_image()
                        {
                            writer.create_element("image:image").write_inner_content::<_, Error>(|writer|
                                {
                                    writer.create_element("image:loc").write_text_content(BytesText::new(&format!("{}{}",domain, uri)))?;
                                    Ok(())
                                })?;
                        }
                        else if content.is_video()
                        {
                            writer.create_element("video:video").write_inner_content::<_, Error>(|writer|
                                {
                                    writer.create_element("video:content_loc").write_text_content(BytesText::new(&format!("{}{}",domain, uri)))?;
                                    writer.create_element("video:publication_date").write_text_content(BytesText::new(&lastmod(SystemTime::now())))?;
                                    Ok(())
                                })?;
                        }
                        else if content.is_html()
                        {
                            writer.create_element("loc").write_text_content(BytesText::new(&format!("{}{}",domain, uri)))?;
                            writer.create_element("lastmod").write_text_content(BytesText::new(&lastmod(SystemTime::now())))?;
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
    domain: String,
    path: String,
    hash: Vec<u8>
}

impl SiteMap
{
    /// Searches the content path from [SiteMap::new] for [Content]
    ///  robots.txt and sitemap.xml can be generated and added here
    pub fn build
    (
        config: &Config,
        tag: bool,
        silent: bool
    ) -> SiteMap
    {

        let server_cache_period = config.content.server_cache_period_seconds;
        let browser_cache_period = config.content.browser_cache_period_seconds;
        let short_urls = config.content.allow_without_extension;
        let filter = match config.content.ignore_regexes.clone()
        {
            Some(p) => Some(ContentFilter::new(p.clone())),
            None => None
        };

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
            &config.content.path,
            &config.content.path,
            Some(server_cache_period),
            Some(browser_cache_period),
            Some(tag),
            filter.as_ref()
        );

        if !silent
        {
            spinner.as_ref().unwrap().finish();
            spinner.unwrap().set_message(format!("Detecting site files took {}", format_elapsed(tic)));
        }

        contents.sort_by_key(|x|x.get_uri());

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

        let mut content_tree = ContentTree::new("/");

        for content in contents
        {
            if content.get_uri().contains("config.json") { continue }
            crate::debug(format!("Adding content {:?}", content.preview(64)), None);
            let path = config.content.path.clone()+"/";
            let uri = parse_uri(content.get_uri(), path);

            if short_urls && content.get_content_type().is_html()
            {
                let short_uri = Regex::new(r"\.\S+$").unwrap().replacen(&uri, 1, "");
                crate::debug(format!("Adding content as short url: {}", short_uri), None);
                content_tree.push(short_uri.to_string(), Content::new(&short_uri, &content.path(), server_cache_period, browser_cache_period, tag));
            }

            content_tree.push(content.uri.clone(), content);
            if !silent {bar.as_ref().unwrap().inc(1);}
        }
        if !silent
        {
            bar.as_ref().unwrap().finish();
            println!("Building sitemap took {}", format_elapsed(tic));
        }

        let mut sitemap = SiteMap
        {
            contents: content_tree,
            domain: config.domain.clone(),
            path: config.content.path.clone(),
            hash: vec![]
        };

        let home = Content::new
        (
            "/",
            &config.content.home.clone(),
            config.content.server_cache_period_seconds,
            config.content.browser_cache_period_seconds,
            tag
        );

        sitemap.push(home, Some("/"));

        if let Some(true) = config.content.generate_sitemap
        {
            sitemap.write_robots();
            sitemap.write_sitemap_xml();
        }

        sitemap.calculate_hash();

        sitemap
    }

    pub fn write_robots(&self)
    {
        write_file_bytes(&format!("{}/{}",self.path,"robots.txt"), format!("Sitemap: {}/sitemap.xml", self.domain).as_bytes());
    }

    pub fn write_sitemap_xml(&self)
    {
        write_file_bytes(&format!("{}/{}",self.path,"sitemap.xml"), &self.to_xml());
    }

    /// Push to [SiteMap::contents] and update the (path) hash
    pub fn push(&mut self, content: Content, uri: Option<&str>)
    {
        let uri = match uri { Some(s) => s.to_string(), None => content.uri.clone() };
        self.contents.push(uri, content);
        self.calculate_hash();
    }

    /// Load all content
    pub async fn refresh_all(&self)
    {
        self.contents.refresh_all().await;
    }

    /// Hash a sitemap by detected uri's
    pub fn get_hash(&self) -> Vec<u8>
    {
        self.hash.clone()
    }

    fn calculate_hash(&mut self)
    {
        self.hash = self.contents.calculate_path_hash();
    }

    /// Returns all uris in the [SiteMap]
    pub fn collect_uris(&self) -> Vec<String>
    {
        self.contents.collect_uris()
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

/// Format for lastmod (t) in an xml sitemap
pub fn lastmod(t: SystemTime) -> String
{
    let date: DateTime<Utc> = t.into();
    format!("{}-{:0>2}-{:0>2}",date.year(), date.month(), date.day())
}