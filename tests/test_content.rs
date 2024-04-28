mod common;

#[cfg(test)]
mod test_content
{
    use std::{fs::remove_file, path::Path};

    use busser::{content::Content, filesystem::file::{file_hash, write_file_bytes, Observed}, util::read_bytes};

    #[test]
    fn test_load_content()
    {
        let mut content = Content::new("tests/pages/a.html", "tests/pages/a.html", 3600);

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
        let mut content_missing = Content::new(file, file, 3600);
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

        let mut content = Content::new(path, path, 3600);

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

}

