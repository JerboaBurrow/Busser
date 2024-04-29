mod common;

#[cfg(test)]
mod test_page
{
    use std::collections::HashMap;

    use busser::content::pages::{get_pages, page::{insert_tag, Page}};

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

    #[test]
    fn test_page_tag()
    {
        let content = "this is /a".to_string();
        let expected = format!("<!--Hosted by Busser {}, https://github.com/JerboaBurrow/Busser-->\n{}", busser::program_version(), content);
        let actual = insert_tag(content);
        assert_eq!(actual, expected);
    }

}