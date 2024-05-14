use std::{collections::HashMap, fs::create_dir, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc, time::Instant};

use axum::{extract::{ConnectInfo, State}, http::Request, middleware::Next, response::Response};
use chrono::DateTime;
use openssl::sha::sha512;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};

use crate::{config::{read_config, CONFIG_PATH}, filesystem::{file::{read_file_utf8, write_file_bytes}, folder::list_dir_by}, util::{compress, date_to_rfc3339, dump_bytes}};

use super::digest::Digest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hit
{
    pub count: u16,
    pub times: Vec<String>,
    pub path: String,
    pub ip_hash: String
}

#[derive(Debug, Clone)]
pub struct HitStats
{
    pub hits: HashMap<[u8; 64], Hit>,
    pub last_save: DateTime<chrono::Utc>,
    pub last_digest: DateTime<chrono::Utc>,
    pub last_clear: DateTime<chrono::Utc>,
    pub summary: Digest
}

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

pub async fn process_hit
(
    addr: SocketAddr,
    state: Arc<Mutex<HitStats>>,
    uri: String 
)
{
    let start_time = Instant::now();

    let config = match read_config(CONFIG_PATH)
    {
        Some(c) => c,
        None =>
        {
            std::process::exit(1)
        }
    };

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
                                ), Some("PERFORMANCE".to_string()));

                                return
                            }
                            hit.times.push(chrono::offset::Utc::now().to_rfc3339());
                            hit.count += 1;
                            hit
                        },
                        Err(_e) => {hit}
                    }
                }            
            }
        },
        false => 
        {
            Hit {path: uri, count: 1, times: vec![chrono::offset::Utc::now().to_rfc3339()], ip_hash: dump_bytes(&ip_hash)}
        }
    };

    crate::debug(format!("{:?}", hit), Some("Statistics".to_string()));

    stats.hits.insert(hash, hit);

    crate::debug(format!
    (
        "\nTotal stats time:         {} s\nCompute stats time:       {} s", 
        start_time.elapsed().as_secs_f64(),
        compute_start_time.elapsed().as_secs_f64()
    ), Some("PERFORMANCE".to_string()));
}

pub fn collect_hits(path: String, stats: Option<HitStats>, from: Option<DateTime<chrono::Utc>>, to: Option<DateTime<chrono::Utc>>) -> Vec<Hit>
{
    let stats_files = list_dir_by(None, path);

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

    for hit in hits_to_filter
    {
        // check the cached stats are within the time period, then add
        let mut count = 0;
        let mut times: Vec<String> = vec![];
        for i in 0..hit.times.len()
        {
            let t = match DateTime::parse_from_rfc3339(&hit.times[i])
            {
                Ok(date) => date,
                Err(e) => {crate::debug(format!("Error {}",e), None); continue}
            };
            if !from.is_some_and(|from| t < from) && !to.is_some_and(|to| t > to) 
            {
                count += 1;
                times.push(hit.times[i].clone());
            }
        }
        if count > 0
        {
            let h = Hit {count, times, ip_hash: hit.ip_hash.clone(), path: hit.path.clone()};
            hits.push(h.clone());
        }
    }

    hits
}

pub fn save(stats: &mut MutexGuard<'_, HitStats>)
{
    let config = match read_config(CONFIG_PATH)
    {
        Some(c) => c,
        None =>
        {
            std::process::exit(1)
        }
    };

    let stats_config = config.stats;

    let write_start_time = Instant::now();

    if !std::path::Path::new(&stats_config.path).exists()
    {
        match create_dir(stats_config.path.to_string())
        {
            Ok(_s) => {},
            Err(e) => {crate::debug(format!("Error creating stats dir {}",e), None)}
        }
    }

    let file_name = stats_config.path.to_string()+"/"+&crate::util::date_now();

    let mut old_hits: Vec<Hit> = if std::path::Path::new(&file_name).exists()
    {
        match read_file_utf8(&file_name)
        {
            Some(d) => 
            {
                match serde_json::from_str(&d)
                {
                    Ok(s) => s,
                    Err(_e) => vec![]
                }
            },
            None => vec![]
        }
    }
    else
    {
        vec![]
    };

    let mut hits: Vec<Hit> = stats.hits.values().cloned().collect();

    hits.append(&mut old_hits);

    match serde_json::to_string(&hits)
    {
        Ok(s) => {write_file_bytes(&file_name, s.as_bytes())},
        Err(e) => {crate::debug(format!("Error saving stats {}", e), None)}
    }

    let write_time = write_start_time.elapsed().as_secs_f64();

    stats.last_save = chrono::offset::Utc::now();
    stats.hits.clear();

    crate::debug(format!
    (
        "Write stats time:       {} s", 
        write_time
    ), Some("PERFORMANCE".to_string()));

}

pub fn archive()
{
    let config = match read_config(CONFIG_PATH)
    {
        Some(c) => c,
        None =>
        {
            std::process::exit(1)
        }
    };

    let stats_files = list_dir_by(None, config.stats.path.clone());

    for file in stats_files
    {
        let time_string = match file.split("/").last()
        {
            Some(s) => s,
            None => {crate::debug(format!("Could not parse time from stats file name {}",file), None); continue}
        };

        let _t = match date_to_rfc3339(time_string)
        {
            Ok(date) => date,
            Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
        };

        let data = match read_file_utf8(&file)
        {
            Some(d) => d,
            None => {continue}
        };

        let _file_hits: Vec<Hit> = match serde_json::from_str(&data)
        {
            Ok(s) => s,
            Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
        };

        let zip = match compress(data.as_bytes())
        {
            Ok(z) => z,
            Err(e) => {crate::debug(format!("{e}, compressing stats file {}", file), None); continue}
        };

        let mut name = String::new();
        name.push_str(config.stats.path.as_str());
        name.push_str(&time_string);
        name.push_str(".zip");
        write_file_bytes(&name, &zip);

        match std::fs::remove_file(file.clone())
        {
            Ok(_) => {},
            Err(e) => {crate::debug(format!("{e}, while deleting stats file {}", file), None);}
        }
    }
}