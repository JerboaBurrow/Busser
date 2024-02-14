use serde::{Serialize, Deserialize};

use crate::web::discord::request::model::Webhook;

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