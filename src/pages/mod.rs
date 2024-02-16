use regex::Regex;

use crate::{util::{list_dir_by, list_sub_dirs, read_file_utf8}, HTML_REGEX};

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
/// use busser::pages::{get_pages, page::Page};
/// 
/// pub fn main()
/// {
///     let pages = get_pages(Some("pages"));
/// 
///     // assert_eq!(pages.len(), 1);
///     // assert!(pages.contains(&Page::new("pages/index.html", "")));
///     // assert!(!pages.contains(&Page::new("pages/animation.js", "")));
/// }
/// ``` 
pub fn get_pages(path: Option<&str>) -> Vec<Page>
{
    let scan_path = match path
    {
        Some(s) => s,
        None => ""
    };

    let html_regex = Regex::new(HTML_REGEX).unwrap();
    let page_paths = list_dir_by(html_regex, scan_path.to_string());
    let mut pages: Vec<Page> = vec![];

    for page_path in page_paths
    {
        let data = match read_file_utf8(&page_path)
        {
            Some(data) => data,
            None => continue
        };

        pages.push(Page::new(page_path.as_str(), data.as_str()));
    }

    let dirs = list_sub_dirs(scan_path.to_string());

    if !dirs.is_empty()
    {
        for dir in dirs
        {
            for page in get_pages(Some(&dir))
            {
                pages.push(page);
            }
        }
    }

    pages

}