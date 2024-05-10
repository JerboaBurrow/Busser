use std::path::Path;

use serde::{Serialize, Deserialize};

use crate::{filesystem::file::read_file_utf8, web::discord::request::model::Webhook};

/// Configure the stats collection
/// - ```save_period_seconds```: periodically save to disc
/// - ```path```: where to save to disc (time-stamped files)
/// - ```hit_cooloff_seconds```: cooloff period after which the same IP is counted as a new hit
/// - ```clear_period_seconds```: periodcially clear data in memory
/// - ```digest_period_seconds```: periodically send a digts to a Discord webhook
/// - ```log_files_clear_period_seconds```:archive and clear stats log files periodically
/// - ```ignore_regexes```: collect, but do not report, hits on these regexes
/// - ```top_n_digest```: top n listing of pages and resources in API/discord default is 3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig
{
    pub save_period_seconds: u64,
    pub path: String,
    pub hit_cooloff_seconds: u64,
    pub digest_period_seconds: u64,
    pub log_files_clear_period_seconds: u64,
    pub ignore_regexes: Option<Vec<String>>,
    pub top_n_digest: Option<usize>
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

/// Configure content settings
/// - ```path```: path to site data
/// - ```home```: path to home page served on /
/// - ```allow_without_extension```: allow serving without .html
/// - ```browser_cache_period_seconds: u16```: content max cache age in cache-control for users
/// - ```server_cache_period_seconds: u16```: internal cache period if content is not static
/// - ```static_content: Option<bool>```: all content is immutably cached at launch
/// - ```ignore_regexes: Option<Vec<String>>```: do not serve content matching any of these patterns
#[derive(Clone, Serialize, Deserialize)]
pub struct ContentConfig
{
    pub path: String,
    pub home: String,
    pub allow_without_extension: bool,
    pub ignore_regexes: Option<Vec<String>>,
    pub browser_cache_period_seconds: u16,
    pub server_cache_period_seconds: u16,
    pub static_content: Option<bool>
}

/// Configure the server
/// - ```port_https```: https port to serve on
/// - ```port_http```: http port to serve on
/// - ```notification_endpoint```: currently unspported Discord webhook
/// - ```cert_path```: ssl certificate
/// - ```key_path```: ssl key
/// - ```domain```: domain name for https redirect etc.
/// - ```throttle```: [ThrottleConfig]
/// - ```stats```: [StatsConfig]
/// - ```content```: [ContentConfig]
/// - ```api_token```: token to use for the server's POST api
#[derive(Clone, Serialize, Deserialize)]
pub struct Config
{
    pub port_https: u16,
    pub port_http: u16,
    pub notification_endpoint: Webhook,
    pub cert_path: String,
    pub key_path: String,
    pub domain: String,
    pub throttle: ThrottleConfig,
    pub stats: StatsConfig,
    pub content: ContentConfig,
    pub api_token: String
}

#[derive(Clone)]
pub struct AppState
{

}

impl AppState
{
    pub fn new() -> AppState
    {
        AppState {}
    } 
}

pub fn read_config() -> Option<Config>
{
    if Path::new(CONFIG_PATH).exists()
    {
        let data = match read_file_utf8(CONFIG_PATH)
        {
            Some(d) => d,
            None =>
            {
                println!("Error reading configuration file {} no data", CONFIG_PATH);
                return None
            }
        };

        let config: Config = match serde_json::from_str(&data)
        {
            Ok(data) => {data},
            Err(why) => 
            {
                println!("Error reading configuration file {}\n{}", CONFIG_PATH, why);
                return None
            }
        };

        Some(config)
    }
    else 
    {
        println!("Error configuration file {} does not exist", CONFIG_PATH);
        None
    }
}