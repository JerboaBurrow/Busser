mod common;

#[cfg(test)]
mod sitemap
{
    use std::{fs::remove_file, path::Path, time::SystemTime};

    use busser::{content::sitemap::{lastmod, SiteMap}, filesystem::file::read_file_utf8};
    use chrono::{DateTime, Datelike, Utc};

    #[test]
    fn test_lastmod()
    {
        assert_eq!(lastmod(SystemTime::UNIX_EPOCH),"1970-01-01");
    }

    #[test]
    fn test_build()
    {

        for file in vec!["tests/pages/robots.txt", "tests/pages/sitemap.xml"]
        {
            let path = Path::new(file);
            if path.exists()
            {
                let _ = remove_file(file);
            }
        }

        let empty_sitemap = r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></urlset>"#;
        let mut sitemap = SiteMap::new("https://test.domain".to_owned(), "tests/pages".to_owned());
        assert_eq!(empty_sitemap, String::from_utf8(sitemap.to_xml()).unwrap());
        sitemap.build(3600, true, true, None);
        
        assert!(Path::new("tests/pages/robots.txt").exists());
        assert!(Path::new("tests/pages/sitemap.xml").exists());

        let robots_disk = read_file_utf8("tests/pages/robots.txt").unwrap();

        for file in vec!["tests/pages/robots.txt", "tests/pages/sitemap.xml"]
        {
            let path = Path::new(file);
            if path.exists()
            {
                let _ = remove_file(file);
            }
        }

        assert_eq!("Sitemap: https://test.domain/sitemap.xml", robots_disk);
    }
}