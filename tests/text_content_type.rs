mod common;

#[cfg(test)]
mod test_Resource_load
{
    use busser::resources::{get_resources, resource::Resource};

    #[test]
    fn test_content_types()
    {
        let resources = get_resources(Some("tests/pages/data"));

        assert_eq!(resources.len(), 19);

        println!("{:?}", resources);

        assert!(resources.contains(&Resource::new("tests/pages/data/b.txt", vec![], "text/plain")));
        assert!(resources.contains(&Resource::new("tests/pages/data/css.css", vec![], "text/css")));
        assert!(resources.contains(&Resource::new("tests/pages/data/csv.csv", vec![], "text/csv")));
        assert!(resources.contains(&Resource::new("tests/pages/data/gif.gif", vec![], "image/gif")));
        assert!(resources.contains(&Resource::new("tests/pages/data/ico.ico", vec![], "image/x-icon")));
        assert!(resources.contains(&Resource::new("tests/pages/data/jpg.jpg", vec![], "image/jpeg")));
        assert!(resources.contains(&Resource::new("tests/pages/data/js.js", vec![], "text/javascript")));
        assert!(resources.contains(&Resource::new("tests/pages/data/mp4.gif", vec![], "image/gif")));
        assert!(resources.contains(&Resource::new("tests/pages/data/mp4.mp4", vec![], "video/mp4")));
        assert!(resources.contains(&Resource::new("tests/pages/data/png.jpg", vec![], "image/jpeg")));
        assert!(resources.contains(&Resource::new("tests/pages/data/png.png", vec![], "image/png")));
        assert!(resources.contains(&Resource::new("tests/pages/data/qt.mov", vec![], "video/quicktime")));
        assert!(resources.contains(&Resource::new("tests/pages/data/svg.svg", vec![], "image/svg+xml")));
        assert!(resources.contains(&Resource::new("tests/pages/data/tiff.tiff", vec![], "image/tiff")));
        assert!(resources.contains(&Resource::new("tests/pages/data/vid.flv", vec![], "video/x-flv")));
        assert!(resources.contains(&Resource::new("tests/pages/data/vid.webm", vec![], "video/webm")));
        assert!(resources.contains(&Resource::new("tests/pages/data/vid.wmv", vec![], "video/x-ms-wmv")));
        assert!(resources.contains(&Resource::new("tests/pages/data/xml.xml", vec![], "text/xml")));
    }

}