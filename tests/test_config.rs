mod common;

#[cfg(test)]
mod config
{
    use busser::{config::{read_config, Config, ContentConfig, StatsConfig, ThrottleConfig}, filesystem::file::write_file_bytes};
    use uuid::Uuid;

    use crate::common::BAD_UTF8;

    #[test]
    fn test_read_config()
    {
        let config_option = read_config("tests/config.json");

        assert!(config_option.is_some());

        let config = config_option.unwrap();

        assert_eq!(config.port_https, 443);
        assert_eq!(config.port_http, 80);
        assert_eq!(config.domain, "127.0.0.1");
        assert_eq!(config.api_token, Some("some_secure_secret_token".to_string()));
        assert_eq!(config.notification_endpoint.unwrap().get_addr(), "https://discord.com/api/webhooks/abc/xyz");
        assert_eq!(config.cert_path, "certs/cert.pem");
        assert_eq!(config.key_path, "certs/key.pem");

        assert_eq!(config.throttle.max_requests_per_second, 64.0);
        assert_eq!(config.throttle.timeout_millis, 5000);
        assert_eq!(config.throttle.clear_period_seconds, 3600);

        assert_eq!(config.stats.save_schedule, Some("0 0 1 * * Wed *".to_string()));
        assert_eq!(config.stats.path, "tests/stats");
        assert_eq!(config.stats.hit_cooloff_seconds, 60);
        assert_eq!(config.stats.digest_schedule, Some("0 0 1 * * Fri *".to_string()));
        assert_eq!(config.stats.ignore_regexes.unwrap(), vec!["/favicon.ico".to_string()]);
        assert_eq!(config.stats.top_n_digest, None);

        assert_eq!(config.content.path, "tests/pages");
        assert_eq!(config.content.home, "tests/pages/a.html");
        assert_eq!(config.content.allow_without_extension, true);
        assert_eq!(config.content.browser_cache_period_seconds, 3600);
        assert_eq!(config.content.server_cache_period_seconds, 1);
        assert_eq!(config.content.ignore_regexes.unwrap(), vec!["/.git", "workspace"]);
        assert_eq!(config.content.static_content, None);
        assert_eq!(config.content.generate_sitemap, Some(false));
    }

    #[test]
    fn test_config_error()
    {
        let missing_config = read_config("not_a_config");

        assert!(missing_config.is_none());

        let not_a_config = read_config("test/pages/b.html");

        assert!(not_a_config.is_none());
    }

    #[test]
    fn test_defaults()
    {
        let stats = StatsConfig::default();

        assert_eq!(stats.save_schedule, None);
        assert_eq!(stats.path, "stats");
        assert_eq!(stats.hit_cooloff_seconds, 60);
        assert_eq!(stats.digest_schedule, None);
        assert_eq!(stats.ignore_regexes, None);
        assert_eq!(stats.top_n_digest, None);
        assert_eq!(stats.ignore_invalid_paths, Some(false));

        let throttle = ThrottleConfig::default();

        assert_eq!(throttle.max_requests_per_second, 64.0);
        assert_eq!(throttle.timeout_millis, 5000);
        assert_eq!(throttle.clear_period_seconds, 3600);

        let content = ContentConfig::default();

        assert_eq!(content.path, "./");
        assert_eq!(content.home, "index.html");
        assert_eq!(content.allow_without_extension, true);
        assert_eq!(content.ignore_regexes, None);
        assert_eq!(content.browser_cache_period_seconds, 3600);
        assert_eq!(content.server_cache_period_seconds, 3600);
        assert_eq!(content.static_content, Some(false));
        assert_eq!(content.message_on_sitemap_reload, Some(false));

        let config = Config::default();

        assert_eq!(config.port_https, 443);
        assert_eq!(config.port_http, 80);
        assert!(config.notification_endpoint.is_none());
        assert_eq!(config.cert_path, "certs/cert.pem");
        assert_eq!(config.key_path, "certs/key.pem");
        assert_eq!(config.domain, "127.0.0.1");

    }

    #[test]
    fn test_load_or_default()
    {

        let mut config = Config::load_or_default("not_a_config");

        let stats = config.stats;

        assert_eq!(stats.save_schedule, None);
        assert_eq!(stats.path, "stats");
        assert_eq!(stats.hit_cooloff_seconds, 60);
        assert_eq!(stats.digest_schedule, None);
        assert_eq!(stats.ignore_regexes, None);
        assert_eq!(stats.top_n_digest, None);

        let throttle = config.throttle;

        assert_eq!(throttle.max_requests_per_second, 64.0);
        assert_eq!(throttle.timeout_millis, 5000);
        assert_eq!(throttle.clear_period_seconds, 3600);

        let content = config.content;

        assert_eq!(content.path, "./");
        assert_eq!(content.home, "index.html");
        assert_eq!(content.allow_without_extension, true);
        assert_eq!(content.ignore_regexes, None);
        assert_eq!(content.browser_cache_period_seconds, 3600);
        assert_eq!(content.server_cache_period_seconds, 3600);
        assert_eq!(content.static_content, Some(false));

        assert_eq!(config.port_https, 443);
        assert_eq!(config.port_http, 80);
        assert!(config.notification_endpoint.is_none());
        assert_eq!(config.cert_path, "certs/cert.pem");
        assert_eq!(config.key_path, "certs/key.pem");
        assert_eq!(config.domain, "127.0.0.1");

        config = read_config("tests/config.json").unwrap();

        assert_eq!(config.port_https, 443);
        assert_eq!(config.port_http, 80);
        assert_eq!(config.domain, "127.0.0.1");
        assert_eq!(config.api_token, Some("some_secure_secret_token".to_string()));
        assert_eq!(config.notification_endpoint.unwrap().get_addr(), "https://discord.com/api/webhooks/abc/xyz");
        assert_eq!(config.cert_path, "certs/cert.pem");
        assert_eq!(config.key_path, "certs/key.pem");

        assert_eq!(config.throttle.max_requests_per_second, 64.0);
        assert_eq!(config.throttle.timeout_millis, 5000);
        assert_eq!(config.throttle.clear_period_seconds, 3600);

        assert_eq!(config.stats.save_schedule, Some("0 0 1 * * Wed *".to_string()));
        assert_eq!(config.stats.path, "tests/stats");
        assert_eq!(config.stats.hit_cooloff_seconds, 60);
        assert_eq!(config.stats.digest_schedule, Some("0 0 1 * * Fri *".to_string()));
        assert_eq!(config.stats.ignore_regexes.unwrap(), vec!["/favicon.ico".to_string()]);

        assert_eq!(config.content.path, "tests/pages");
        assert_eq!(config.content.home, "tests/pages/a.html");
        assert_eq!(config.content.allow_without_extension, true);
        assert_eq!(config.content.browser_cache_period_seconds, 3600);
        assert_eq!(config.content.server_cache_period_seconds, 1);
        assert_eq!(config.content.ignore_regexes.unwrap(), vec!["/.git", "workspace"]);
    }

    #[test]
    fn test_bad_utf8()
    {
        let file_name = format!("tests/bad_utf8-{}", Uuid::new_v4());
        write_file_bytes(&file_name, &BAD_UTF8);
        assert!(read_config(&file_name).is_none());
        std::fs::remove_file(file_name).unwrap();
    }

    #[test]
    fn test_not_json()
    {
        let file_name = format!("tests/not_json-{}", Uuid::new_v4());
        write_file_bytes(&file_name, "not_json{".as_bytes());
        assert!(read_config(&file_name).is_none());
        std::fs::remove_file(file_name).unwrap();
    }
}