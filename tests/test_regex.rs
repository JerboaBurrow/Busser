mod common;

#[cfg(test)]
mod page_regex
{
    use busser::pages::page::is_page;


    #[test]
    fn test_is_page()
    {
        println!("{}", is_page("abc.html"));
        assert!(is_page("abc.html"));
        assert!(is_page("html.html"));
        assert!(is_page("abc"));
        assert!(!is_page("abc.svg"));
        assert!(!is_page("abc.png"));
        assert!(!is_page("abc.js"));
        assert!(!is_page("html.js"));
        assert!(!is_page("abc."));
    }

}