mod common;

#[cfg(test)]
mod test_page_load
{
    use busser::pages::{page::Page, read_pages};

    #[test]
    fn test_read_pages()
    {
        let pages = read_pages(Some("tests/config.json"));

        assert_eq!(pages.len(), 2);

        assert!(pages.contains(&Page::new("a", "this is a")));

        assert!(pages.contains(&Page::new("path/to/b", "this is b")));
    }

}