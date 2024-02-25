use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr};
use std::sync::Arc;
use std::time::Instant;
use chrono::DateTime;
use tokio::sync::Mutex;

use ipinfo::{IpDetails, IpInfo, IpInfoConfig};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit
{
    count: u64,
    last: String,
    path: String,
    details: IpDetails
}

#[derive(Debug, Clone)]
pub struct Stats
{
    pub hits: HashMap<IpAddr, Hit>,
    pub last_save: Instant
}

impl Stats
{
    pub async fn process_hit
    (
        addr: SocketAddr,
        state: Arc<Mutex<Stats>>,
        uri: String 
    )
    {
        let start_time = Instant::now();

        let mut stats = state.lock().await;

        let compute_start_time = Instant::now();

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
                            let total_time = start_time.elapsed().as_secs_f64();
                            let compute_time = compute_start_time.elapsed().as_secs_f64();

                            crate::debug(format!
                            (
                                "\nTotal stats time:         {} s (Passthrough)\nCompute stats time:       {} s (Passthrough)", 
                                total_time,
                                compute_time
                            ), Some("PERFORMANCE".to_string()));

                            return
                        }
                    },
                    Err(_e) => {}
                }
            }            
        }

        let details = match stats_config.ipinfo_token
        {
            Some(token) => get_ip_info(token, sip).await,
            None => IpDetails { ip: sip, ..Default::default() }
        };

        let last = chrono::offset::Utc::now().to_rfc3339();

        let hit = Hit { details, path: uri, count, last };

        crate::debug(format!("[Hit] {:?}", hit), None);

        stats.hits.insert(addr.ip(), hit);

        let compute_time = compute_start_time.elapsed().as_secs_f64();

        let write_start_time = Instant::now();

        if stats.last_save.elapsed() >= std::time::Duration::from_secs(stats_config.save_period_seconds)
        {
            let file_name = stats_config.path.to_string()+"-"+&chrono::offset::Utc::now().to_rfc3339();
            match serde_json::to_string(&stats.hits)
            {
                Ok(s) => {write_file(&file_name, s.as_bytes())},
                Err(e) => {crate::debug(format!("Error saving stats {}", e), None)}
            }

            if stats.last_save.elapsed() >= std::time::Duration::from_secs(stats_config.clear_period_seconds)
            {
                stats.hits.clear()
            }

            stats.last_save = Instant::now();
        }

        let write_time = write_start_time.elapsed().as_secs_f64();
        let total_time = start_time.elapsed().as_secs_f64();

        crate::debug(format!
        (
            "\nTotal stats time:         {} s\nCompute stats time:       {} s\nWrite stats time:         {} s", 
            total_time,
            compute_time,
            write_time
        ), Some("PERFORMANCE".to_string()));
    }
}

use axum::
{
    http::{Request, StatusCode}, 
    response::Response, 
    extract::{State, ConnectInfo},
    middleware::Next
};

pub async fn get_ip_info(token: String, sip: String) -> IpDetails
{
    let config = IpInfoConfig {
        token: Some(token),
        ..Default::default()
    };

    let mut ipinfo = IpInfo::new(config)
        .expect("should construct");

    let res = ipinfo.lookup(&sip).await;
    
    match res {
        Ok(r) => r,
        Err(e) => 
        {
            crate::debug(format!("Error getting ip details {}", e), None);
            IpDetails { ip: sip, ..Default::default()}
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
    
    let uri = request.uri().to_string();
    tokio::spawn
    (async move
        {
            Stats::process_hit(addr, state, uri).await
        }
    );
           
    Ok(next.run(request).await)
}