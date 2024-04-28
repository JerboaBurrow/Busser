mod common;

#[cfg(test)]
mod filesystem
{

    use std::{fs::remove_file, path::Path};

    use busser::filesystem::{file::{read_file_bytes, read_file_utf8, write_file_bytes}, folder::list_dir_by};
    use regex::Regex;


    #[test]
    fn test_read_bytes()
    {
        let expected = "this is /a".as_bytes();
        let actual = read_file_bytes("tests/pages/a.html").unwrap();
        assert_eq!(actual, expected);

        let path = Path::new("test_file_error");
        if path.exists()
        {
            let _ = remove_file(path);
        }
        assert!(read_file_bytes(path.to_str().unwrap()).is_none());
    }

    #[test]
    fn test_read_utf8()
    {
        let expected = "this is /a";
        let actual = read_file_utf8("tests/pages/a.html").unwrap();
        assert_eq!(actual, expected);

        let path = Path::new("test_file_error");
        if path.exists()
        {
            let _ = remove_file(path);
        }
        assert!(read_file_utf8(path.to_str().unwrap()).is_none());
    }

    #[test]
    fn test_write_bytes()
    {
        let expected = "this is a file written by busser";

        write_file_bytes("test_write_bytes", expected.as_bytes());

        let actual = read_file_utf8("test_write_bytes").unwrap();
        assert_eq!(actual, expected);

        let _ = remove_file("test_write_bytes");
    }

    #[test]
    fn test_list_dir()
    {
        let r = Regex::new(r"\.(jpg|jpeg)$").unwrap();
        let actual = list_dir_by(Some(r), "tests/pages/data".to_owned());

        assert!(actual.contains(&"tests/pages/data/jpg.jpg".to_string()));
        assert!(actual.contains(&"tests/pages/data/png.jpg".to_string()));
        assert_eq!(actual.len(), 2);
    }
}