mod common;

#[cfg(test)]
mod sitemap
{
    use std::{fs::remove_file, path::Path, time::SystemTime};

    use busser::{config::Config, content::sitemap::{lastmod, SiteMap}, filesystem::file::read_file_utf8};
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

        let mut config = Config::load_or_default("tests/config.json");
        config.domain = "https://test.domain".to_string();

        let sitemap = SiteMap::build(&config, false, false);
        
        sitemap.write_robots();
        assert!(Path::new("tests/pages/robots.txt").exists());
        sitemap.write_sitemap_xml();
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
    }

}