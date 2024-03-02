use std::path::Path;

use serde::{Serialize, Deserialize};

use crate::{util::read_file_utf8, web::discord::request::model::Webhook};

/// Configure the stats collection
/// - ```save_period_seconds```: periodically save to disc
/// - ```path```: where to save to disc (time-stamped files)
/// - ```hit_cooloff_seconds```: cooloff period after which the same IP is counted as a new hit
/// - ```clear_period_seconds```: periodcially clear data in memory
/// - ```digest_period_seconds```: periodically send a digts to a Discord webhook
/// - ```log_files_clear_period_seconds```:clear disc log files periodically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig
{
    pub save_period_seconds: u64,
    pub path: String,
    pub hit_cooloff_seconds: u64,
    pub digest_period_seconds: u64,
    pub log_files_clear_period_seconds: u64
}

pub const CONFIG_PATH: &str = "config.json";

/// Configure the IP throttler
/// - ```max_requests_per_second```: includes all requests to html and resources per second per ip
/// - ```timeout_millis```: a cool off period between IP-blocks
/// - ```clear_period_seconds```: time period to clear all stored IPs
#[derive(Clone, Serialize, Deserialize)]
pub struct ThrottleConfig
{
    max_requests_per_second: f64,
    timeout_millis: u128,
    clear_period_seconds: u64
}

impl ThrottleConfig 
{
    pub fn get_max_requests_per_second(&self) -> f64
    {
        self.max_requests_per_second
    }

    pub fn get_timeout_millis(&self) -> u128
    {
        self.timeout_millis
    }

    pub fn get_clear_period_seconds(&self) -> u64
    {
        self.clear_period_seconds
    }
}

/// Configure the server
/// - ```port_https```: https port to serve on
/// - ```port_http```: http port to serve on
/// - ```path```: path to site data
/// - ```home```: path to home page served on /
/// - ```notification_endpoint```: currently unspported Discord webhook
/// - ```cert_path```: ssl certificate
/// - ```key_path```: ssl key
/// - ```domain```: domain name for https redirect etc.
/// - ```throttle```: [ThrottleConfig]
/// - ```allow_without_extension```: allow serving without .html
/// - ```stats```: [StatsConfig]
/// - ```cache_period_seconds: u16```: page/resource max cache age
/// - ```api_token```: token to use for the server's POST api
#[derive(Clone, Serialize, Deserialize)]
pub struct Config
{
    port_https: u16,
    port_http: u16,
    path: String,
    home: String,
    notification_endpoint: Webhook,
    cert_path: String,
    key_path: String,
    domain: String,
    throttle: ThrottleConfig,
    stats: StatsConfig,
    allow_without_extension: bool,
    cache_period_seconds: u16,
    api_token: String
}

impl Config 
{
    pub fn get_port_http(&self) -> u16
    {
        self.port_http
    }

    pub fn get_port_https(&self) -> u16
    {
        self.port_https
    }

    pub fn get_end_point(&self) -> Webhook
    {
        self.notification_endpoint.clone()
    }

    pub fn get_cert_path(&self) -> String
    {
        self.cert_path.clone()
    }

    pub fn get_key_path(&self) -> String
    {
        self.key_path.clone()
    }

    pub fn get_path(&self) -> String
    {
        self.path.clone()
    }

    pub fn get_home(&self) -> String
    {
        self.home.clone()
    }
    
    pub fn get_throttle_config(&self) -> ThrottleConfig
    {
        self.throttle.clone()
    }

    pub fn get_stats_config(&self) -> StatsConfig
    {
        self.stats.clone()
    }

    pub fn get_domain(&self) -> String
    {
        self.domain.clone()
    }

    pub fn get_allow_without_extension(&self) -> bool
    {
        self.allow_without_extension.clone()
    }

    pub fn get_cache_period_seconds(&self) -> u16
    {
        self.cache_period_seconds.clone()
    }

    pub fn get_api_token(&self) -> String
    {
        self.api_token.clone()
    }
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