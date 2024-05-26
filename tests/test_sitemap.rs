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

        let empty_sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1"></urlset>"#;
        let mut sitemap = SiteMap::new("https://test.domain".to_owned(), "tests/pages".to_owned());
        
        assert_eq!(empty_sitemap, String::from_utf8(sitemap.to_xml()).unwrap());
        assert_eq!(sitemap.collect_uris(), Vec::<String>::new());

        sitemap.build(true, false, None);
        
        assert!(Path::new("tests/pages/robots.txt").exists());
        assert!(Path::new("tests/pages/sitemap.xml").exists());

        let uris = sitemap.collect_uris();
        assert!(uris.contains(&"/a".to_string()));
        assert!(uris.contains(&"/b".to_string()));
        assert!(uris.contains(&"/c/d".to_string()));
        assert!(uris.contains(&"/a.html".to_string()));
        assert!(uris.contains(&"/b.html".to_string()));
        assert!(uris.contains(&"/c/d.html".to_string()));

        let sitemap_disk = read_file_utf8("tests/pages/sitemap.xml").unwrap();
        let robots_disk = read_file_utf8("tests/pages/robots.txt").unwrap();

        for file in vec!["tests/pages/robots.txt", "tests/pages/sitemap.xml"]
        {
            let path = Path::new(file);
            if path.exists()
            {
                let _ = remove_file(file);
            }
        }

        let mut expected_sitemap = read_file_utf8("tests/common/sitemap.xml").unwrap();
        // for windows...
        expected_sitemap = expected_sitemap.replace("\r", "");

        let date: DateTime<Utc> = SystemTime::now().into();
        let today = format!("{}-{:0>2}-{:0>2}",date.year(), date.month(), date.day());
        expected_sitemap = expected_sitemap.replace("TODAY", &today);

        assert_eq!(sitemap_disk, expected_sitemap);
        assert_eq!("Sitemap: https://test.domain/sitemap.xml", robots_disk);

        let mut sitemap = SiteMap::new("https://test.domain".to_owned(), "tests/pages".to_owned());
        
        assert_eq!(sitemap.get_hash(), Vec::<u8>::new());
    
        sitemap.build(true, true, None);
        assert_ne!(sitemap.get_hash(), Vec::<u8>::new());
        assert!(sitemap.get_hash().len() > 0);
        
        for file in vec!["tests/pages/robots.txt", "tests/pages/sitemap.xml"]
        {
            let path = Path::new(file);
            if path.exists()
            {
                let _ = remove_file(file);
            }
        }
    }

}