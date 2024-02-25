mod common;

#[cfg(test)]
mod test_page_load
{
    use busser::pages::{page::Page, get_pages};

    #[test]
    fn test_read_pages()
    {
        let pages = get_pages(Some("tests/pages"), None);

        assert_eq!(pages.len(), 3);

        assert!(pages.contains(&Page::new("tests/pages/a.html", "this is /a", 3600)));

        assert!(pages.contains(&Page::new("tests/pages/b.html", "this is /b", 3600)));

        assert!(pages.contains(&Page::new("tests/pages/c/d.html", "this is /c/d", 3600)));
    }

}