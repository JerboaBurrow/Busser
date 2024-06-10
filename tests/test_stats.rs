mod common;

#[cfg(test)]
mod test_stats_graph
{
    use std::{collections::HashMap, fs::remove_file, path::Path};

    use busser::{config::Config, filesystem::file::File, server::stats::{digest::{digest_message, hits_by_hour_text_graph, process_hits, Digest}, file::StatsFile, hits::{collect_hits, Hit, HitStats}}};
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
        let hits: [usize; 24] = core::array::from_fn(|i| i);
        let graph = hits_by_hour_text_graph(hits, '-', 24);
        println!("{}", graph);
        assert_eq!(graph, GRAPH);
    }

    #[test]
    fn test_collect_hits()
    {

        let mut config = Config::load_or_default("tests/config.json");

        let mut hits = collect_hits(None, None, None, &config);

        config.stats.ignore_invalid_paths = Some(false);

        assert_eq!(hits.len(), 17);

        let mut hit = Hit 
        {
            times: vec!["2024-03-25T04:12:44.736120969+00:00".to_string()],
            path: "/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_owned(),
            ip_hash: "75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_owned()
        };

        assert!(hits.contains(&hit));

        hits = collect_hits( None, Some(DateTime::parse_from_rfc3339("2024-03-25T00:00:00.000000000+00:00").unwrap().to_utc()), None, &config);
        
        assert_eq!(hits.len(), 1);

        assert!(hits.contains(&hit));

        hits = collect_hits(None, None, Some(DateTime::parse_from_rfc3339("2024-03-24T23:12:44.736120969+00:00").unwrap().to_utc()), &config);
        
        assert_eq!(hits.len(), 16);

        hit = Hit 
        {
            times: vec!["2024-03-24T04:12:44.736120969+00:00".to_string()],
            path: "/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_owned(),
            ip_hash: "75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_owned()
        };

        assert!(hits.contains(&hit));

    }

    #[test]
    fn test_collect_hits_ignore_invalid()
    {
        let mut config = Config::load_or_default("tests/config.json");
        config.stats.ignore_invalid_paths = Some(true);
        let hits = collect_hits(None, None, None, &config);
        let hit = Hit 
        {
            times: vec!["2024-03-25T04:12:44.736120969+00:00".to_string()],
            path: "/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_owned(),
            ip_hash: "75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_owned()
        };
        
        assert_eq!(hits.len(), 2);
        assert!(!hits.contains(&hit));
    }

    #[test]
    fn test_stats_digest()
    {
        let mut config = Config::load_or_default("tests/config.json");
        config.stats.ignore_invalid_paths = Some(false);
        let digest = process_hits(None, None, &config, None);
        
        assert_eq!(digest.unique_hits, 9);
        assert_eq!(digest.total_hits, 21);
        assert_eq!(digest.hits_by_hour_utc, [1, 0, 0, 0, 2, 4, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0, 0, 3, 1, 0, 2, 1, 0, 0]);
    
        assert_eq!(digest.top_hitters.first().unwrap(), &("75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_string(), 6));
        assert_eq!(digest.top_pages.first().unwrap(), &("/".to_string(), 5));
        assert!(digest.top_resources.contains(&("/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_string(), 2))); 
        assert!(digest.top_resources.contains(&("https://jerboa.app/console.js".to_string(), 2))); 
        assert!(digest.top_resources.contains(&("/admin/.env".to_string(), 2)));

        let msg = digest_message(&digest, None, None);
        let msg_at_epoch = digest_message(&digest, DateTime::UNIX_EPOCH.into(), DateTime::UNIX_EPOCH.into());
        assert!(msg != msg_at_epoch);
    }

    #[test]
    fn test_new()
    {
        let stats = HitStats::new();

        assert_eq!(stats.hits, HashMap::new());
        assert_eq!(stats.summary, Digest::new());        
    }

    #[test]
    fn test_stats_file()
    {
        let mut file = StatsFile::new();

        assert_eq!(file.path, None);
        assert_eq!(file.hits, vec![]);

        let hits = HitStats::new();

        if Path::exists(Path::new(&file.path()))
        {
            let _ = remove_file(Path::new(&file.path()));
        }

        file.load(&hits);

        assert_eq!(file.path, None);
        assert_eq!(file.hits, vec![]);

        let hit = Hit 
        {
            times: vec!["2024-03-24T04:12:44.736120969+00:00".to_string()],
            path: "/login.php/'%3E%3Csvg/onload=confirm%60xss%60%3E".to_owned(),
            ip_hash: "75A05052881EA1D68995532845978B4090012883F99354EFF67AD4E1ED5FF1833F4A2EC893181EAA00B94B9CD35E1E1DD581B7F80FEF2EFF45B75D529A080BD8".to_owned()
        };

        file.path = Some("tests/stats/2024-03-24".to_owned());
        file.load(&hits);
        
        assert_eq!(file.hits.len(), 16);
        assert!(file.hits.contains(&hit));

        file.path = Some("test_stats_file".to_owned());
        file.write_bytes();
        file.load(&hits);
        
        assert_eq!(file.hits.len(), 16);
        assert!(file.hits.contains(&hit));

        if Path::exists(Path::new("test_stats_file"))
        {
            let _ = remove_file(Path::new("test_stats_file"));
        }
    }

}