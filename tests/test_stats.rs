mod common;

#[cfg(test)]
mod test_stats_graph
{
    use std::str::FromStr;

    use busser::web::stats::{hits_by_hour_text_graph, Hit, Stats};
    use chrono::DateTime;

    const GRAPH: &str = r#"00:00
01:00-
02:00--
03:00---
04:00----
05:00-----
06:00------
07:00-------
08:00--------
09:00---------
10:00----------
11:00------------
12:00-------------
13:00--------------
14:00---------------
15:00----------------
16:00-----------------
17:00------------------
18:00-------------------
19:00--------------------
20:00---------------------
21:00----------------------
22:00------------------------
23:00-------------------------
"#;

    #[test]
    fn test_text_graph()
    {
        let hits: [u16; 24] = core::array::from_fn(|i| i as u16);
        let graph = hits_by_hour_text_graph(hits, '-', 24);
        println!("{}", graph);
        assert_eq!(graph, GRAPH);
    }

    #[test]
    fn test_collect_hits()
    {
        let mut hits = Stats::collect_hits("tests/stats".to_owned(), None, None, None);
        assert_eq!(hits.len(), 17);

        let mut hit = Hit 
        {
            count: 1,
            times: vec!["2024-03-25T04:12:44.736120969+00:00".to_string()],
            path: "/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_owned(),
            ip_hash: "75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_owned()
        };

        assert!(hits.contains(&hit));

        hits = Stats::collect_hits("tests/stats".to_owned(), None, Some(DateTime::parse_from_rfc3339("2024-03-25T00:00:00.000000000+00:00").unwrap().to_utc()), None);
        
        assert_eq!(hits.len(), 1);

        assert!(hits.contains(&hit));

        hits = Stats::collect_hits("tests/stats".to_owned(), None, None, Some(DateTime::parse_from_rfc3339("2024-03-24T23:12:44.736120969+00:00").unwrap().to_utc()));
        
        assert_eq!(hits.len(), 16);

        hit = Hit 
        {
            count: 1,
            times: vec!["2024-03-24T04:12:44.736120969+00:00".to_string()],
            path: "/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_owned(),
            ip_hash: "75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_owned()
        };

        assert!(hits.contains(&hit));

    }

    #[test]
    fn test_stats_digest()
    {
        let digest = Stats::process_hits("tests/stats".to_owned(), None, None, None, None);
        
        assert_eq!(digest.unique_hits, 9);
        assert_eq!(digest.total_hits, 21);
        assert_eq!(digest.hits_by_hour_utc, [1, 0, 0, 0, 2, 4, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0, 0, 3, 1, 0, 2, 1, 0, 0]);
    
        assert_eq!(digest.top_hitters.first().unwrap(), &("75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_string(), 6 as u16));
        assert_eq!(digest.top_pages.first().unwrap(), &("/".to_string(), 5));
        assert!(digest.top_resources.contains(&("/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_string(), 2 as u16))); 
        assert!(digest.top_resources.contains(&("https://jerboa.app/console.js".to_string(), 2 as u16))); 
        assert!(digest.top_resources.contains(&("/admin/.env".to_string(), 2 as u16)));
    }

}