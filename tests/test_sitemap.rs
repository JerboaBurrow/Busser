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

        let mut built_sitemap = r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
            <loc>https://test.domain/a</loc>
            <lastmod>TODAY</lastmod>
            <loc>https://test.domain/a.html</loc>
            <lastmod>TODAY</lastmod>
            <loc>https://test.domain/b</loc>
            <lastmod>TODAY</lastmod>
            <loc>https://test.domain/b.html</loc>
            <lastmod>TODAY</lastmod>
    </url>
    <url>
            <loc>https://test.domain/c/d</loc>
            <lastmod>TODAY</lastmod>
            <loc>https://test.domain/c/d.html</loc>
            <lastmod>TODAY</lastmod>
    </url>
    <url>
            <video:video>
                    <video:content_loc>https://test.domain/data/vid</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/vid.wmv</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/vid</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/vid.webm</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <image:image>
                    <image:loc>https://test.domain/data/png</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/png.png</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/png</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/png.jpg</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/gif</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/gif.gif</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/svg</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/svg.svg</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <video:video>
                    <video:content_loc>https://test.domain/data/mpeg</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/mpeg.mpeg</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/qt</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/qt.mov</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/mp4</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/mp4.mp4</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <image:image>
                    <image:loc>https://test.domain/data/tiff</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/tiff.tiff</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <video:video>
                    <video:content_loc>https://test.domain/data/vid</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <video:video>
                    <video:content_loc>https://test.domain/data/vid.flv</video:content_loc>
                    <video:publication_date>TODAY</video:publication_date>
            </video:video>
            <image:image>
                    <image:loc>https://test.domain/data/jpg</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/jpg.jpg</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/ico</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/ico.ico</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/mp4</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
            <image:image>
                    <image:loc>https://test.domain/data/mp4.gif</image:loc>
                    <lastmod>TODAY</lastmod>
            </image:image>
    </url>
</urlset>"#.to_string();

        let date: DateTime<Utc> = SystemTime::now().into();
        let today = format!("{}-{:0>2}-{:0>2}",date.year(), date.month(), date.day());
        built_sitemap = built_sitemap.replace("TODAY", &today);

        assert_eq!(sitemap_disk, built_sitemap);
        assert_eq!("Sitemap: https://test.domain/sitemap.xml", robots_disk);
    }
}