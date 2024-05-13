mod common;

#[cfg(test)]
mod config
{
    use busser::config::read_config;

    #[test]
    fn test_read_config()
    {
        let config_option = read_config("tests/config.json");

        assert!(config_option.is_some());

        let config = config_option.unwrap();

        assert_eq!(config.port_https, 443);
        assert_eq!(config.port_http, 80);
        assert_eq!(config.domain, "127.0.0.1");
        assert_eq!(config.api_token, "some_secure_secret_token");
        assert_eq!(config.notification_endpoint.get_addr(), "https://discord.com/api/webhooks/abc/xyz");
        assert_eq!(config.cert_path, "certs/cert.pem");
        assert_eq!(config.key_path, "certs/key.pem");

        assert_eq!(config.throttle.max_requests_per_second, 64.0);
        assert_eq!(config.throttle.timeout_millis, 5000);
        assert_eq!(config.throttle.clear_period_seconds, 3600);

        assert_eq!(config.stats.save_period_seconds, 10);
        assert_eq!(config.stats.path, "stats");
        assert_eq!(config.stats.hit_cooloff_seconds, 60);
        assert_eq!(config.stats.digest_period_seconds, 86400);
        assert_eq!(config.stats.log_files_clear_period_seconds, 2419200);
        assert_eq!(config.stats.ignore_regexes.unwrap(), vec!["/favicon.ico".to_string()]);

        assert_eq!(config.content.path, "/home/jerboa/Website/");
        assert_eq!(config.content.home, "/home/jerboa/Website/jerboa.html");
        assert_eq!(config.content.allow_without_extension, true);
        assert_eq!(config.content.browser_cache_period_seconds, 3600);
        assert_eq!(config.content.server_cache_period_seconds, 1);
        assert_eq!(config.content.ignore_regexes.unwrap(), vec!["/.git", "workspace"]);
    }

    #[test]
    fn test_config_error()
    {
        let missing_config = read_config("not_a_config");

        assert!(missing_config.is_none());

        let not_a_config = read_config("test/pages/b.html");

        assert!(not_a_config.is_none());
    }
}