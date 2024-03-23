mod common;

#[cfg(test)]
mod page_regex
{
    use busser::pages::page::is_page;


    #[test]
    fn test_is_page()
    {
        println!("{}", is_page("abc.html", ""));
        assert!(is_page("abc.html", ""));
        assert!(is_page("html.html", ""));
        assert!(is_page("abc", ""));
        assert!(!is_page("abc.svg", ""));
        assert!(!is_page("abc.png", ""));
        assert!(!is_page("abc.js", ""));
        assert!(!is_page("html.js", ""));
        assert!(!is_page("abc.", ""));
        assert!(!is_page("a.htm", ""));

        assert!(is_page("http://domain", "domain"));
        assert!(is_page("http://domain/", "domain"));
        assert!(is_page("http://domain/something", "domain"));
        assert!(is_page("http://domain/something.html", "domain"));

        assert!(is_page("https://domain", "domain"));
        assert!(is_page("https://domain/", "domain"));
        assert!(is_page("https://domain/something", "domain"));
        assert!(is_page("https://domain/something.html", "domain"));

        assert!(!is_page("http://domain.", "domain"));
        assert!(!is_page("http://domain/a.b", "domain"));
        assert!(!is_page("http://domain/something.h", "domain"));
        assert!(!is_page("http://domain/something.abc", "domain"));

        assert!(!is_page("https://domain.", "domain"));
        assert!(!is_page("https://domain/a.b", "domain"));
        assert!(!is_page("https://domain/something.h", "domain"));
        assert!(!is_page("https://domain/something.abc", "domain"));
    }

}