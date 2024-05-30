use std::path::Path;

use serde::{Serialize, Deserialize};

use crate::{filesystem::file::read_file_utf8, integrations::webhook::Webhook};

/// Configure the stats collection
/// - ```path```: where to save to disc (time-stamped files)
/// - ```hit_cooloff_seconds```: cooloff period after which the same IP is counted as a new hit
/// - ```clear_period_seconds```: periodcially clear data in memory
/// - ```save_schedule```: periodically save to disc, cron format: "sec min hour day-of-month month day-of-week year"
/// - ```digest_schedule```: periodically send a digts to a Discord webhook, cron format: "sec min hour day-of-month month day-of-week year"
/// - ```ignore_regexes```: collect, but do not report, hits on these regexes
/// - ```top_n_digest```: top n listing of pages and resources in API/discord default is 3
/// - ```ignore_invalid_paths: Option<bool>```: in digest don't report hits to invalid paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig
{
    pub path: String,
    pub hit_cooloff_seconds: u64,
    pub save_schedule: Option<String>,
    pub digest_schedule: Option<String>,
    pub ignore_regexes: Option<Vec<String>>,
    pub top_n_digest: Option<usize>,
    pub ignore_invalid_paths: Option<bool>
}

impl StatsConfig
{
    pub fn default() -> StatsConfig
    {
        StatsConfig
        {
            path: "stats".to_string(),
            hit_cooloff_seconds: 60,
            save_schedule: None,
            digest_schedule: None,
            ignore_regexes: None,
            top_n_digest: None,
            ignore_invalid_paths: Some(false)
        }
    }
}

pub const CONFIG_PATH: &str = "config.json";

/// Configure the IP throttler
/// - ```max_requests_per_second```: includes all requests to html and resources per second per ip
/// - ```timeout_millis```: a cool off period between IP-blocks
/// - ```clear_period_seconds```: time period to clear all stored IPs
#[derive(Clone, Serialize, Deserialize)]
pub struct ThrottleConfig
{
    pub max_requests_per_second: f64,
    pub timeout_millis: u128,
    pub clear_period_seconds: u64
}

impl ThrottleConfig
{
    pub fn default() -> ThrottleConfig
    {
        ThrottleConfig
        {
            max_requests_per_second: 64.0,
            timeout_millis: 5000,
            clear_period_seconds: 3600
        }
    }
}

/// Configure content settings
/// - ```path```: path to site data
/// - ```home```: path to home page served on /
/// - ```allow_without_extension```: allow serving without .html
/// - ```browser_cache_period_seconds: u16```: content max cache age in cache-control for users
/// - ```server_cache_period_seconds: u16```: internal cache period if content is not static
/// - ```static_content: Option<bool>```: all content is immutably cached at launch
/// - ```ignore_regexes: Option<Vec<String>>```: do not serve content matching any of these patterns
/// - ```generate_sitemap: Option<bool>```: sitemap.xml will be automatically generated (and updated) 
/// - ```message_on_sitemap_reload: Option<bool>```: optionally send Discord notifications when sitemap is reloaded
#[derive(Clone, Serialize, Deserialize)]
pub struct ContentConfig
{
    pub path: String,
    pub home: String,
    pub allow_without_extension: bool,
    pub ignore_regexes: Option<Vec<String>>,
    pub browser_cache_period_seconds: u16,
    pub server_cache_period_seconds: u16,
    pub static_content: Option<bool>,
    pub generate_sitemap: Option<bool>,
    pub message_on_sitemap_reload: Option<bool>
}

impl ContentConfig
{
    pub fn default() -> ContentConfig
    {
        ContentConfig
        {
            path: "./".to_string(),
            home: "index.html".to_string(),
            allow_without_extension: true,
            ignore_regexes: None,
            browser_cache_period_seconds: 3600,
            server_cache_period_seconds: 3600,
            static_content: Some(false),
            generate_sitemap: Some(true),
            message_on_sitemap_reload: Some(false)
        }
    }
}

/// Passphrase or ssh authentication setup (plaintext storage)
/// - ```key_path```: optional location of ssh key (ssh connection will be used)
/// - ```user```: user name for authentication
/// - ```passphrase```: passphrase for ssh key or for user-pass auth
#[derive(Clone, Serialize, Deserialize)]
pub struct GitAuthConfig
{
    pub key_path: Option<String>,
    pub user: String,
    pub passphrase: String
}

/// Git repository busser will track for content
/// - ```remote```: the url (public or private)
/// - ```branch```: the tracked branch
/// - ```auth```: if present either ssh key or passphrase will be used
#[derive(Clone, Serialize, Deserialize)]
pub struct GitConfig
{
    pub remote: String,
    pub branch: String,
    pub checkout_schedule: Option<String>,
    pub auth: Option<GitAuthConfig>
}

/// Configure the server
/// - ```port_https```: https port to serve on
/// - ```port_http```: http port to serve on
/// - ```notification_endpoint```: currently unspported Discord webhook
/// - ```cert_path```: ssl certificate
/// - ```key_path```: ssl key
/// - ```domain```: domain name for https redirect etc.
/// - ```api_token```: token to use for the server's POST api
/// - ```throttle```: [ThrottleConfig]
/// - ```stats```: [StatsConfig]
/// - ```content```: [ContentConfig]
/// - ```git```: [GitConfig] if present busser will track a git repo for content
/// 
/// <div class="warning"><p>The config.json is a sensitive file which may contain plaintext access tokens/ passphrases.
/// Content matching "config.json" is not served.
/// </p>
/// </div>
#[derive(Clone, Serialize, Deserialize)]
pub struct Config
{
    pub port_https: u16,
    pub port_http: u16,
    pub notification_endpoint: Option<Webhook>,
    pub cert_path: String,
    pub key_path: String,
    pub domain: String,
    pub api_token: Option<String>,
    pub throttle: ThrottleConfig,
    pub stats: StatsConfig,
    pub content: ContentConfig,
    pub git: Option<GitConfig>
}

impl Config 
{
    pub fn default() -> Config
    {
        Config
        {
            port_http: 80,
            port_https: 443,
            notification_endpoint: None,
            cert_path: "certs/cert.pem".to_string(),
            key_path: "certs/key.pem".to_string(),
            domain: "127.0.0.1".to_string(),
            api_token: None,
            throttle: ThrottleConfig::default(),
            stats: StatsConfig::default(),
            content: ContentConfig::default(),
            git: None
        }
    }

    pub fn load_or_default(path: &str) -> Config
    {
        match read_config(path)
        {
            Some(c) => c,
            None =>
            {
                Config::default()
            }
        }
    }
}

pub fn read_config(path: &str) -> Option<Config>
{
    if Path::new(&path).exists()
    {
        let data = match read_file_utf8(&path)
        {
            Some(d) => d,
            None =>
            {
                crate::debug(format!("Error reading configuration file {} no data", path), None);
                return None
            }
        };

        let config: Config = match serde_json::from_str(&data)
        {
            Ok(data) => {data},
            Err(why) => 
            {
                crate::debug(format!("Error reading configuration file {}\n{}", path, why), None);
                return None
            }
        };

        Some(config)
    }
    else 
    {
        crate::debug(format!("Error configuration file {} does not exist", path), None);
        None
    }
}