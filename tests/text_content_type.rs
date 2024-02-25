mod common;

#[cfg(test)]
mod test_Resource_load
{
    use busser::resources::{get_resources, resource::Resource};

    #[test]
    fn test_content_types()
    {
        let resources = get_resources(Some("tests/pages/data"), None);

        assert_eq!(resources.len(), 19);

        println!("{:?}", resources);

        assert!(resources.contains(&Resource::new("tests/pages/data/b.txt", vec![], "text/plain", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/css.css", vec![], "text/css", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/csv.csv", vec![], "text/csv", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/gif.gif", vec![], "image/gif", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/ico.ico", vec![], "image/x-icon", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/jpg.jpg", vec![], "image/jpeg", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/js.js", vec![], "text/javascript", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/mp4.gif", vec![], "image/gif", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/mp4.mp4", vec![], "video/mp4", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/png.jpg", vec![], "image/jpeg", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/png.png", vec![], "image/png", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/qt.mov", vec![], "video/quicktime", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/svg.svg", vec![], "image/svg+xml", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/tiff.tiff", vec![], "image/tiff", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/vid.flv", vec![], "video/x-flv", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/vid.webm", vec![], "video/webm", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/vid.wmv", vec![], "video/x-ms-wmv", 3600)));
        assert!(resources.contains(&Resource::new("tests/pages/data/xml.xml", vec![], "text/xml", 3600)));
    }

}