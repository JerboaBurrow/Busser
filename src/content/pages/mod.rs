use regex::Regex;

use crate::{filesystem::folder::{list_dir_by, list_sub_dirs}, HTML_REGEX};
use self::page::Page;

pub mod page;

/// Scan the path (if None the current dir) for .html pages
///   note Busser can be configured to server .html pages without
///   the extension, but for scanning the extension is required.
/// 
/// # Example
/// ```rust
/// // with files pages/index.html, pages/animation.js
/// 
/// use busser::content::pages::{get_pages, page::Page};
/// 
/// pub fn main()
/// {
///     let pages = get_pages(Some("pages"), Some(3600));
/// 
///     // assert_eq!(pages.len(), 1);
///     // assert!(pages.contains(&Page::new("pages/index.html", "")));
///     // assert!(!pages.contains(&Page::new("pages/animation.js", "")));
/// }
/// ``` 
pub fn get_pages(path: Option<&str>, cache_period_seconds: Option<u16>) -> Vec<Page>
{
    let scan_path = match path
    {
        Some(s) => s,
        None => ""
    };

    let html_regex = Regex::new(HTML_REGEX).unwrap();
    let page_paths = list_dir_by(Some(html_regex), scan_path.to_string());
    let mut pages: Vec<Page> = vec![];

    let cache = match cache_period_seconds
    {
        Some(p) => p,
        None => 3600
    };

    for page_path in page_paths
    {
        pages.push(Page::new(page_path.as_str(), &page_path, cache));
    }

    let dirs = list_sub_dirs(scan_path.to_string());

    if !dirs.is_empty()
    {
        for dir in dirs
        {
            for page in get_pages(Some(&dir), cache_period_seconds)
            {
                pages.push(page);
            }
        }
    }

    pages

}