use serde::{Serialize, Deserialize};

use crate::web::discord::request::model::Webhook;

pub const CONFIG_PATH: &str = "config.json";

#[derive(Clone, Serialize, Deserialize)]
pub struct Config
{
    port_https: u16,
    port_http: u16,
    stats_endpoint: Webhook,
    cert_path: String,
    key_path: String
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
        self.stats_endpoint.clone()
    }

    pub fn get_cert_path(&self) -> String
    {
        self.cert_path.clone()
    }

    pub fn get_key_path(&self) -> String
    {
        self.key_path.clone()
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