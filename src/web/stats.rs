use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr};
use std::sync::Arc;
use std::time::Instant;
use axum::response::IntoResponse;
use chrono::DateTime;
use tokio::sync::Mutex;

use ipinfo::{IpDetails, IpInfo, IpInfoConfig};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit
{
    count: u64,
    last: String,
    details: Option<IpDetails>
}

#[derive(Debug, Clone)]
pub struct Stats
{
    pub hits: HashMap<IpAddr, Hit>,
    pub last_save: Instant
}

use axum::
{
    http::{Request, StatusCode}, 
    response::Response, 
    extract::{State, ConnectInfo},
    middleware::Next
};

pub async fn get_ip_info(token: String, sip: String) -> Option<IpDetails>
{
    let config = IpInfoConfig {
        token: Some(token),
        ..Default::default()
    };

    let mut ipinfo = IpInfo::new(config)
        .expect("should construct");

    let res = ipinfo.lookup(&sip).await;
    
    match res {
        Ok(r) => Some(r),
        Err(e) => 
        {
            crate::debug(format!("Error getting ip details {}", e), None);
            None
        }
    }
}

use crate::config::read_stats_config;
use crate::util::write_file;
pub async fn log_stats<B>
(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<Stats>>>,
    request: Request<B>,
    next: Next<B>
) -> Result<Response, StatusCode>
{

    {
        let mut stats = state.lock().await;

        let stats_config = read_stats_config().unwrap();
        
        let sip = addr.ip().to_string();

        let (count, last_hit) = match stats.hits.contains_key(&addr.ip())
        {
            true => { (stats.hits[&addr.ip()].count+1, Some(stats.hits[&addr.ip()].last.clone()))},
            false => {(1, None)}
        };

        match last_hit 
        {
            None => {},
            Some(s) => 
            {
                let t = DateTime::parse_from_rfc3339(&s);
                match t 
                {
                    Ok(t) => 
                    {
                        let delta = (chrono::offset::Utc::now()-t.to_utc()).num_seconds();
                        if delta < (stats_config.hit_cooloff_seconds as i64)
                        {
                            let response = next.run(request).await;
                            return Ok(response)
                        }
                    },
                    Err(e) => {}
                }
            }            
        }

        let details = match stats_config.ipinfo_token
        {
            Some(token) => get_ip_info(token, sip).await,
            None => None
        };

        let last = chrono::offset::Utc::now().to_rfc3339();

        let hit = Hit { details, count, last };

        crate::debug(format!("[Hit] {:?}", hit), None);

        stats.hits.insert(addr.ip(), hit);

        if stats.last_save.elapsed() >= std::time::Duration::from_secs(stats_config.save_frequency_seconds)
        {
            stats.last_save = Instant::now();
            let file_name = stats_config.path.to_string()+"-"+&chrono::offset::Utc::now().to_rfc3339();
            match serde_json::to_string(&stats.hits)
            {
                Ok(s) => {write_file(&file_name, s.as_bytes())},
                Err(e) => {crate::debug(format!("Error saving stats {}", e), None)}
            }
        }
    }

    let response = next.run(request).await;
    Ok(response)
    
}