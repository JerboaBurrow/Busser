mod common;

#[cfg(test)]
mod page_regex
{
    use busser::content::pages::page::is_page;


    #[test]
    fn test_is_page()
    {
        assert!(is_page("http://domain", "domain"));
        assert!(is_page("http://domain/", "domain"));
        assert!(is_page("http://domain/something", "domain"));
        assert!(is_page("http://domain/something.html", "domain"));

        assert!(!is_page("http://other", "domain"));
        assert!(!is_page("http://other/", "domain"));
        assert!(!is_page("http://other/something", "domain"));
        assert!(!is_page("http://other/something.html", "domain"));

        assert!(is_page("http://sub.domain", "sub.domain"));
        assert!(is_page("http://sub.sub.domain", "sub.sub.domain"));
        assert!(is_page("http://sub.domain/", "sub.domain"));
        assert!(is_page("http://sub.sub.domain/", "sub.sub.domain"));

        assert!(is_page("http://sub.domain/a", "sub.domain"));
        assert!(is_page("http://sub.sub.domain/a", "sub.sub.domain"));
        assert!(is_page("http://sub.domain/a", "sub.domain"));
        assert!(is_page("http://sub.sub.domain/a", "sub.sub.domain"));

        assert!(!is_page("http://domain.", "domain"));
        assert!(!is_page("http://domain/a.b", "domain"));
        assert!(!is_page("http://domain/something.h", "domain"));
        assert!(!is_page("http://domain/something.abc", "domain"));

        assert!(!is_page("https://domain.", "domain"));
        assert!(!is_page("https://domain/a.b", "domain"));
        assert!(!is_page("https://domain/something.h", "domain"));
        assert!(!is_page("https://domain/something.abc", "domain"));

        assert!(!is_page("https://domain.", "https://domain"));
        assert!(!is_page("https://domain/a.b", "https://domain"));
        assert!(!is_page("https://domain/something.h", "http://domain"));
        assert!(!is_page("https://domain/something.abc", "http://domain"));

        
        assert!(is_page("domain", "domain"));
        assert!(is_page("domain/", "domain"));
        assert!(is_page("domain/something", "domain"));
        assert!(is_page("domain/something.html", "domain"));

        assert!(!is_page("other", "domain"));
        assert!(!is_page("other/", "domain"));
        assert!(!is_page("other/something", "domain"));
        assert!(!is_page("other/something.html", "domain"));

        assert!(is_page("sub.domain", "sub.domain"));
        assert!(is_page("sub.sub.domain", "sub.sub.domain"));
        assert!(is_page("sub.domain/", "sub.domain"));
        assert!(is_page("sub.sub.domain/", "sub.sub.domain"));

        assert!(is_page("sub.domain/a", "sub.domain"));
        assert!(is_page("sub.sub.domain/a", "sub.sub.domain"));
        assert!(is_page("sub.domain/a", "sub.domain"));
        assert!(is_page("sub.sub.domain/a", "sub.sub.domain"));

        assert!(!is_page("domain.", "domain"));
        assert!(!is_page("domain/a.b", "domain"));
        assert!(!is_page("domain/something.h", "domain"));
        assert!(!is_page("domain/something.abc", "domain"));

        assert!(!is_page("domain.", "domain"));
        assert!(!is_page("domain/a.b", "domain"));
        assert!(!is_page("domain/something.h", "domain"));
        assert!(!is_page("domain/something.abc", "domain"));

        assert!(!is_page("domain.", "http://domain"));
        assert!(!is_page("domain/a.b", "https://domain"));
        assert!(!is_page("domain/something.h", "http://domain"));
        assert!(!is_page("domain/something.abc", "https://domain"));

    }

}