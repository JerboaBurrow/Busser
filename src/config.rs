use std::path::Path;

use serde::{Serialize, Deserialize};

use crate::{util::read_file_utf8, web::discord::request::model::Webhook};

pub const STATS_CONFIG_PATH: &str = "stats_config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig
{
    pub save_period_seconds: u64,
    pub path: String,
    pub hit_cooloff_seconds: u64,
    pub clear_period_seconds: u64,
    pub ipinfo_token: Option<String>
}

pub const CONFIG_PATH: &str = "config.json";

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
    throttle: ThrottleConfig
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

    pub fn get_domain(&self) -> String
    {
        self.domain.clone()
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

pub fn read_stats_config() -> Option<StatsConfig>
{
    if Path::new(STATS_CONFIG_PATH).exists()
    {
        let data = match read_file_utf8(STATS_CONFIG_PATH)
        {
            Some(d) => d,
            None =>
            {
                println!("Error reading configuration file {} no data", CONFIG_PATH);
                return None
            }
        };

        let config: StatsConfig = match serde_json::from_str(&data)
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