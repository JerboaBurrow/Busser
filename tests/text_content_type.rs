mod common;

#[cfg(test)]
mod test_resource_load
{
    use std::collections::HashMap;

    use busser::content::resources::{get_resources, resource::Resource};

    #[test]
    fn test_content_types()
    {
        let resources = get_resources(Some("tests/pages/data"), None);

        assert_eq!(resources.len(), 19);

        let paths = HashMap::from(
            [
                ("tests/pages/data/b.txt", "text/plain"),
                ("tests/pages/data/css.css", "text/css"),
                ("tests/pages/data/csv.csv", "text/csv"),
                ("tests/pages/data/gif.gif", "image/gif"),
                ("tests/pages/data/ico.ico", "image/x-icon"),
                ("tests/pages/data/jpg.jpg", "image/jpeg"),
                ("tests/pages/data/mp4.mp4", "video/mp4"),
                ("tests/pages/data/mpeg.mpeg", "video/mpeg"),
                ("tests/pages/data/js.js", "text/javascript"),
                ("tests/pages/data/mp4.gif", "image/gif"),
                ("tests/pages/data/png.jpg", "image/jpeg"),
                ("tests/pages/data/qt.mov", "video/quicktime"),
                ("tests/pages/data/svg.svg", "image/svg+xml"),
                ("tests/pages/data/tiff.tiff", "image/tiff"),
                ("tests/pages/data/vid.flv", "video/x-flv"),
                ("tests/pages/data/vid.webm","video/webm") ,
                ("tests/pages/data/vid.wmv", "video/x-ms-wmv"),
                ("tests/pages/data/xml.xml", "text/xml")
            ]
        );

        for (path, expected_mime_type) in paths
        {
            assert!(resources.contains(&Resource::new(path, path, 3600)));
            let res = Resource::new(path, &path, 3600);
            assert_eq!(res.get_content_type(), expected_mime_type)
        }
    }

}