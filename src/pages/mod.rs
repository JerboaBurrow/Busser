use regex::Regex;

use crate::{util::{list_dir_by, list_sub_dirs, read_file_utf8}, HTML_REGEX};

use self::page::Page;

pub mod page;

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