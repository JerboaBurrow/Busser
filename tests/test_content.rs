mod common;

#[cfg(test)]
mod test_content
{
    use std::{collections::HashMap, fs::remove_file, path::Path, thread::sleep, time};

    use busser::{content::{filter::ContentFilter, get_content, insert_tag, is_page, mime_type::MIME, Content, HasUir}, filesystem::file::{file_hash, write_file_bytes, Observed}, util::read_bytes};

    #[test]
    fn test_load_content()
    {
        let mut content = Content::new("tests/pages/a.html", "tests/pages/a.html", 60, 3600, false);

        assert_eq!(content.get_uri(), "tests/pages/a.html".to_string());
        assert!(content.utf8_body().is_ok_and(|b| b == "".to_string()));

        assert!(content.load_from_file().is_ok());
        assert!(content.utf8_body().is_ok_and(|b| b == "this is /a".to_string()));

        let file = "test_load_content";
        let path = Path::new("file");
        if path.exists()
        {
            let _ = remove_file(file);
        }
        let mut content_missing = Content::new(file, file, 60, 3600, false);
        assert!(content_missing.load_from_file().is_err());
    }

    #[test]
    fn test_observed_content()
    {
        let path = "test_observed_content";
        let test_content = "this is some test content";
        let test_content_hash = "2d5bb7c3afbe68c05bcd109d890dca28ceb0105bf529ea1111f9ef8b44b217b9".to_string();
        let modified_test_content = "this is some modified content";
        let modified_test_content_hash = "c4ea4898725c3390549d40a19a26a57730730b42050def80f1d157581e33b2db".to_string();

        write_file_bytes(path, test_content.as_bytes());

        let mut content = Content::new(path, path, 60, 3600, false);

        assert!(content.load_from_file().is_ok());
        assert!(!content.is_stale());
        assert_eq!(file_hash(path), read_bytes(test_content_hash));
        assert!(content.utf8_body().is_ok_and(|b| b == test_content.to_string()));
        write_file_bytes(path, modified_test_content.as_bytes());

        assert!(content.is_stale());
        assert_eq!(file_hash(path), read_bytes(modified_test_content_hash));
        content.refresh();
        assert!(content.utf8_body().is_ok_and(|b| b == modified_test_content.to_string()));

        let _ = remove_file(path);
    }

    #[test]
    fn test_last_refreshed()
    {
        let mut content = Content::new("tests/pages/a.html", "tests/pages/a.html", 60, 3600, false);
        assert!(content.load_from_file().is_ok());
        let a = content.last_refreshed();
        sleep(time::Duration::from_secs(2));
        assert!(content.load_from_file().is_ok());
        let b = content.last_refreshed();
        assert!(a < b);
        assert!(!content.server_cache_expired())
    }

    #[test]
    fn test_content_filter()
    {
        let ignore_patterns = vec![".gif".to_string(), ".ico".to_string()];

        let content = vec![
            Content::new("tests/pages/a.html", "tests/pages/a.html", 60, 3600, false),
            Content::new("tests/pages/data/b.txt", "tests/pages/data/b.txt", 60, 3600, false),
            Content::new("tests/pages/data/ico.ico", "tests/pages/data/ico.ico", 60, 3600, false),
            Content::new("tests/pages/data/gif.gif", "tests/pages/data/gif.gif", 60, 3600, false),
            Content::new("tests/pages/data/mp4.gif", "tests/pages/data/mp4.gif", 60, 3600, false),
            Content::new("tests/pages/data/png.jpg", "tests/pages/data/png.jpg", 60, 3600, false),
        ];

        let filter = ContentFilter::new(ignore_patterns);

        let filtered = filter.filter::<Content>(content.clone());

        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&content[0]));
        assert!(filtered.contains(&content[1]));
        assert!(!filtered.contains(&content[2]));
        assert!(!filtered.contains(&content[3]));
        assert!(!filtered.contains(&content[4]));
        assert!(filtered.contains(&content[5]));

    }

    #[test]
    fn test_content_types()
    {
        let contents = get_content("tests/pages", "tests/pages/data", None, None, None);

        assert_eq!(contents.len(), 19);

        let paths = HashMap::from(
            [
                ("tests/pages/data/b.txt", ("/data/b.txt", MIME::TextPlain)),
                ("tests/pages/data/css.css", ("/data/css.css", MIME::TextCSS)),
                ("tests/pages/data/csv.csv", ("/data/csv.csv", MIME::TextCSV)),
                ("tests/pages/data/gif.gif", ("/data/gif.gif", MIME::ImageGIF)),
                ("tests/pages/data/ico.ico", ("/data/ico.ico", MIME::ImageXICON)),
                ("tests/pages/data/jpg.jpg", ("/data/jpg.jpg", MIME::ImageJPEG)),
                ("tests/pages/data/mp4.mp4", ("/data/mp4.mp4", MIME::VideoMP4)),
                ("tests/pages/data/mpeg.mpeg", ("/data/mpeg.mpeg", MIME::VideoMPEG)),
                ("tests/pages/data/js.js", ("/data/js.js", MIME::TextJavascript)),
                ("tests/pages/data/mp4.gif", ("/data/mp4.gif", MIME::ImageGIF)),
                ("tests/pages/data/png.jpg", ("/data/png.jpg", MIME::ImageJPEG)),
                ("tests/pages/data/qt.mov", ("/data/qt.mov", MIME::VideoQuicktime)),
                ("tests/pages/data/svg.svg", ("/data/svg.svg", MIME::ImageSVG)),
                ("tests/pages/data/tiff.tiff", ("/data/tiff.tiff", MIME::ImageTIFF)),
                ("tests/pages/data/vid.flv", ("/data/vid.flv", MIME::VideoFLV)),
                ("tests/pages/data/vid.webm", ("/data/vid.webm", MIME::VideoWEBM)),
                ("tests/pages/data/vid.wmv", ("/data/vid.wmv", MIME::VideoWMV)),
                ("tests/pages/data/xml.xml", ("/data/xml.xml", MIME::TextXML))
            ]
        );

        for (path, (expected_uri, expected_mime_type)) in paths
        {
            assert!(contents.contains(&Content::new(expected_uri, path, 60, 3600, false)));
            let res = Content::new(path, &path, 60, 3600, false);
            assert_eq!(res.get_content_type(), expected_mime_type)
        }
    }

    #[test]
    fn test_read_contents()
    {
        let contents = get_content("tests/pages", "tests/pages", None, None, None);

        assert_eq!(contents.len(), 24);

        let paths = HashMap::from(
            [
                ("tests/pages/a.html", ("/a.html", "this is /a")),
                ("tests/pages/b.html", ("/b.html", "this is /b")),
                ("tests/pages/c/d.html", ("/c/d.html", "this is /c/d")),
            ]
        );

        for (path, (expected_uri, expected_body)) in paths
        {
            let mut content = Content::new(&expected_uri, path, 60, 3600, false);
            assert!(contents.contains(&content));
            content.load_from_file().unwrap();
            let actual_body = content.utf8_body().unwrap();
            assert_eq!(actual_body, expected_body)
        }

    }

    #[test]
    fn test_page_tag()
    {
        let content = "this is /a".to_string();
        let expected = format!("<!--Hosted by Busser {}, https://github.com/JerboaBurrow/Busser-->\n{}", busser::program_version(), content);
        let actual = insert_tag(content);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_is_page()
    {
        let domain_a = "https://test.domain.one".to_string();
        let domain_b = "test.domain".to_string();

        let content = vec!
        [
            ("a.html".to_string(), false, false),
            (domain_a.clone(), true, false),
            (domain_b.clone(), false, true),
            (format!("{}/{}", domain_a, "a.html"), true, false),
            (format!("{}/{}", domain_b, "b.html"), false, true),
            (format!("{}/{}", domain_a, "a.js"), false, false),
            (format!("{}/{}", domain_b, "a.csv"), false, false)
        ];

        for (uri, a, b) in content
        {
            println!("{}, {}, {}", uri, a, b);
            assert_eq!(is_page(&uri, &domain_a), a);
            assert_eq!(is_page(&uri, &domain_b), b);
        }
    }

}

