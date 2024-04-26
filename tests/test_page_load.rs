mod common;

#[cfg(test)]
mod test_page_load
{
    use std::collections::HashMap;

    use busser::content::pages::{page::Page, get_pages};

    #[test]
    fn test_read_pages()
    {
        let pages = get_pages(Some("tests/pages"), None);

        assert_eq!(pages.len(), 3);
        
        let paths = HashMap::from(
            [
                ("tests/pages/a.html", "this is /a"),
                ("tests/pages/b.html", "this is /b"),
                ("tests/pages/c/d.html", "this is /c/d"),
            ]
        );

        for (path, expected_body) in paths
        {
            let mut page = Page::new(path, path, 3600);
            assert!(pages.contains(&page));
            page.load_from_file().unwrap();
            let actual_body = page.utf8_body().unwrap();
            assert_eq!(actual_body, expected_body)
        }

       
    }

}