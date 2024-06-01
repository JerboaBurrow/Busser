use std::{collections::{HashMap, HashSet}, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc, time::Instant};

use axum::{extract::{ConnectInfo, State}, http::Request, middleware::Next, response::Response};
use chrono::DateTime;
use openssl::sha::sha512;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, content::sitemap::SiteMap, filesystem::{file::read_file_utf8, folder::list_dir_by}, util::{date_to_rfc3339, dump_bytes}};

use super::digest::Digest;

/// A hit defined by a uri and an ip (hashed) at given times
/// - [Hit::times]   the unique hit times of equivalent events (controlled by [crate::config::StatsConfig::hit_cooloff_seconds])
/// - [Hit::path]    the uri defining this hit
/// - [Hit::ip_hash] the sha512 hash of the ip defining this hit
/// 
/// Further hits inside the [crate::config::StatsConfig::hit_cooloff_seconds] period will be ignored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hit
{
    pub times: Vec<String>,
    pub path: String,
    pub ip_hash: String
}

impl Hit
{
    pub fn count(&self) -> usize
    {
        self.times.len().clone()
    }
}

/// A collections of Hits indexed by a sha512 of the ip and uri
#[derive(Debug, Clone)]
pub struct HitStats
{
    pub hits: HashMap<[u8; 64], Hit>,
    pub summary: Digest
}

impl HitStats
{
    pub fn new() -> HitStats
    {
        HitStats
        {
            hits: HashMap::new(), 
            summary: Digest::new()
        }
    }
}

/// Launches a thread to log statistics for this request
pub async fn log_stats<B>
(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<HitStats>>>,
    request: Request<B>,
    next: Next<B>
) -> Result<Response, StatusCode>
{
    
    let uri = request.uri().to_string();
    tokio::spawn
    (async move
        {
            process_hit(addr, state, uri).await
        }
    );
           
    Ok(next.run(request).await)
}

/// Updates hit statistics with this request
pub async fn process_hit
(
    addr: SocketAddr,
    state: Arc<Mutex<HitStats>>,
    uri: String 
)
{
    let start_time = Instant::now();

    let config = Config::load_or_default(CONFIG_PATH);

    let mut stats = state.lock().await;

    let compute_start_time = Instant::now();

    let stats_config = config.stats;

    let ipv4: Ipv4Addr = match addr.ip()
    {
        IpAddr::V4(ip4) => {ip4}
        IpAddr::V6(_ip6) => {return}
    };
    
    let ip_hash = sha512(&ipv4.octets());
    let hash = sha512(&[uri.as_bytes(), &ipv4.octets()].concat());

    let hit = match stats.hits.contains_key(&hash)
    {
        true =>
        {
            let mut hit = stats.hits[&hash].clone();

            match stats.hits[&hash].times.last()
            {
                None => {hit},
                Some(s) => 
                {
                    match DateTime::parse_from_rfc3339(&s)
                    {
                        Ok(t) => 
                        {
                            if (chrono::offset::Utc::now()-t.to_utc()).num_seconds() < (stats_config.hit_cooloff_seconds as i64)
                            {
                                crate::debug(format!
                                (
                                    "\nTotal stats time:         {} s (Passthrough)\nCompute stats time:       {} s (Passthrough)", 
                                    start_time.elapsed().as_secs_f64(),
                                    compute_start_time.elapsed().as_secs_f64()
                                ), Some("PERFORMANCE"));

                                return
                            }
                            hit.times.push(chrono::offset::Utc::now().to_rfc3339());
                            hit
                        },
                        Err(_e) => {hit}
                    }
                }            
            }
        },
        false => 
        {
            Hit {path: uri, times: vec![chrono::offset::Utc::now().to_rfc3339()], ip_hash: dump_bytes(&ip_hash)}
        }
    };

    crate::debug(format!("{:?}", hit), Some("Statistics"));

    stats.hits.insert(hash, hit);

    crate::debug(format!
    (
        "\nTotal stats time:         {} s\nCompute stats time:       {} s", 
        start_time.elapsed().as_secs_f64(),
        compute_start_time.elapsed().as_secs_f64()
    ), Some("PERFORMANCE"));
}

/// Gathers [Hit]s both from disk and those cached in [HitStats]
pub fn collect_hits
(
    stats: Option<HitStats>, 
    from: Option<DateTime<chrono::Utc>>, 
    to: Option<DateTime<chrono::Utc>>,
    config: &Config
) -> Vec<Hit>
{
    let stats_files = list_dir_by(None, config.stats.path.clone());

    let mut hits: Vec<Hit> = vec![];

    let mut hits_to_filter: Vec<Hit> = vec![];

    for file in stats_files
    {
        crate::debug(format!("Processing stats files: {}", file), None);
        
        let time_string = match file.split("/").last()
        {
            Some(s) => s,
            None => {crate::debug(format!("Could not parse time from stats file name {}",file), None); continue}
        };

        let t = match date_to_rfc3339(time_string)
        {
            Ok(date) => date,
            Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
        };

        if from.is_some_and(|from| t < from) { continue }
        if to.is_some_and(|to| t > to) { continue }

        let data = match read_file_utf8(&file)
        {
            Some(d) => d,
            None => {continue}
        };

        match serde_json::from_str(&data)
        {
            Ok(mut file_hits) => hits_to_filter.append(&mut file_hits),
            Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
        };
    }

    if stats.is_some()
    {
        for (_hash, hit) in &stats.as_ref().unwrap().hits
        {
            hits_to_filter.push(hit.clone());
        }
    }

    let uris: Option<HashSet<String>> = if let Some(true) = config.stats.ignore_invalid_paths
    {
        Some(SiteMap::build(config, false, true).collect_uris().into_iter().collect())
    }
    else
    {
        None
    };

    for hit in hits_to_filter
    {
        // check the cached stats are within the time period, then add
        let mut times: Vec<String> = vec![];
        for i in 0..hit.times.len()
        {
            if uris.as_ref().is_some_and(|u| !u.contains(&hit.path))
            {
                continue
            } 

            let t = match DateTime::parse_from_rfc3339(&hit.times[i])
            {
                Ok(date) => date,
                Err(e) => {crate::debug(format!("Error {}",e), None); continue}
            };
            if !from.is_some_and(|from| t < from) && !to.is_some_and(|to| t > to) 
            {
                times.push(hit.times[i].clone());
            }
        }
        if times.len() > 0
        {
            let h = Hit {times, ip_hash: hit.ip_hash.clone(), path: hit.path.clone()};
            hits.push(h.clone());
        }
    }

    hits
}