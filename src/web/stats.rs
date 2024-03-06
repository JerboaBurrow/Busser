use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::create_dir;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Datelike, TimeZone, Timelike};
use openssl::sha::sha512;
use tokio::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use axum::
{
    http::{Request, StatusCode}, 
    response::Response, 
    extract::{State, ConnectInfo},
    middleware::Next
};

use crate::config::read_config;
use crate::util::{dump_bytes, list_dir_by, read_file_utf8, write_file};

use crate::web::discord::request::post::post;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit
{
    count: u16,
    times: Vec<String>,
    path: String,
    ip_hash: String
}

#[derive(Debug, Clone)]
pub struct Digest
{
    pub top_three_hitters: [(String, u16); 3],
    pub top_three_paths: [(String, u16); 3],
    pub hits_by_hour_utc: [u16; 24],
    pub total_hits: u16,
    pub unique_hits: u16
}

impl Digest
{
    pub fn new() -> Digest
    {
        Digest 
        {
            top_three_hitters: [(String::new(),0), (String::new(),0), (String::new(),0)],
            top_three_paths: [(String::new(),0), (String::new(),0), (String::new(),0)],
            hits_by_hour_utc: [0;24],
            total_hits: 0,
            unique_hits: 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stats
{
    pub hits: HashMap<[u8; 64], Hit>,
    pub last_save: DateTime<chrono::Utc>,
    pub last_digest: DateTime<chrono::Utc>,
    pub last_clear: DateTime<chrono::Utc>,
    pub summary: Digest
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

        let config = match read_config()
        {
            Some(c) => c,
            None =>
            {
                std::process::exit(1)
            }
        };

        let mut stats = state.lock().await;

        let compute_start_time = Instant::now();

        let stats_config = config.get_stats_config();

        let ip = addr.ip();
        let ipv4: Ipv4Addr;
    
        match ip 
        {
            IpAddr::V4(ip4) => {ipv4 = ip4}
            IpAddr::V6(_ip6) => {return}
        }
        
        let ip_hash = sha512(&ipv4.octets());
        let hash = sha512(&[uri.as_bytes(), &ipv4.octets()].concat());

        let hit = match stats.hits.contains_key(&hash)
        {
            true =>
            {
                let mut hit = stats.hits[&hash].clone();
                let last_hit = stats.hits[&hash].times.last();

                match last_hit 
                {
                    None => {hit},
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

        let compute_time = compute_start_time.elapsed().as_secs_f64();

        let total_time = start_time.elapsed().as_secs_f64();

        crate::debug(format!
        (
            "\nTotal stats time:         {} s\nCompute stats time:       {} s", 
            total_time,
            compute_time
        ), Some("PERFORMANCE".to_string()));
    }

    pub fn clear_logs(path: String, before: DateTime<chrono::Utc>)
    {
        let stats_files = list_dir_by(None, path);

        for file in stats_files
        {
            let t = match DateTime::parse_from_rfc3339(&file)
            {
                Ok(date) => date,
                Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
            };

            if t > before
            {
                continue
            }

            match std::fs::remove_file(file.clone())
            {
                Ok(()) => {},
                Err(e) => {crate::debug(format!("Could not delete stats files {}\n {}", file, e), None);}
            }
        }
    }

    pub fn process_hits(path: String, from: DateTime<chrono::Utc>, stats: Option<Stats>) -> Digest
    {

        let mut digest = Digest::new();

        let stats_files = list_dir_by(None, path);

        let mut hitters: HashMap<String, u16> = HashMap::new();
        let mut paths: HashMap<String, u16> = HashMap::new();

        for file in stats_files
        {
            crate::debug(format!("Processing stats files: {}", file), None);
            
            let time_string = match file.split("/").last()
            {
                Some(s) => s,
                None => {crate::debug(format!("Could not parse time from stats file name {}",file), None); continue}
            };

            let t = match DateTime::parse_from_rfc3339(&time_string)
            {
                Ok(date) => date,
                Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
            };

            if t < from
            {
                continue
            }

            let data = match read_file_utf8(&file)
            {
                Some(d) => d,
                None => {continue}
            };

            let mut hits: Vec<Hit> = match serde_json::from_str(&data)
            {
                Ok(s) => s,
                Err(e) => {crate::debug(format!("Error {} loading stats file {}",e,file), None); continue}
            };

            if stats.is_some()
            {
                for (_hash, hit) in &stats.as_ref().unwrap().hits
                {
                    hits.push(hit.clone());
                }
            }

            for hit in hits
            {
                match hitters.contains_key(&hit.ip_hash)
                {
                    true => {hitters.insert(hit.ip_hash.clone(), hit.count+hitters[&hit.ip_hash]);},
                    false => 
                    {
                        hitters.insert(hit.ip_hash, hit.count);
                        digest.unique_hits += 1;
                    }
                }

                match paths.contains_key(&hit.path)
                {
                    true => {paths.insert(hit.path.clone(), hit.count+paths[&hit.path]);},
                    false => {paths.insert(hit.path, hit.count);}
                }

                digest.total_hits += hit.count;

                for time in hit.times
                {
                    match DateTime::parse_from_rfc3339(&time)
                    {
                        Ok(t) => 
                        {
                            if (0..23).contains(&t.hour()) { digest.hits_by_hour_utc[t.hour() as usize]+= 1; }
                        },
                        Err(_e) => {}
                    }
                }
            }
        }

        let mut all_hitters: Vec<(String, u16)> = hitters.into_iter().collect();
        let mut all_paths: Vec<(String, u16)> = paths.into_iter().collect();

        all_hitters.sort_by(|a: &(String, u16), b: &(String, u16)| a.1.cmp(&b.1));

        for i in 0..3
        {
            if i < all_hitters.len()
            {
                digest.top_three_hitters[i] = all_hitters[i].clone();
            }
            else
            {
                digest.top_three_hitters[i] = ("".to_string(), 0);
            }
        }

        all_paths.sort_by(|a: &(String, u16), b: &(String, u16)| a.1.cmp(&b.1));

        for i in 0..3
        {
            if i < all_paths.len()
            {
                digest.top_three_paths[i] = all_paths[i].clone();
            }
            else
            {
                digest.top_three_paths[i] = ("".to_string(), 0);
            }
        }

        digest

    }

    pub fn save(stats: &mut MutexGuard<'_, Stats>)
    {
        let config = match read_config()
        {
            Some(c) => c,
            None =>
            {
                std::process::exit(1)
            }
        };

        let stats_config = config.get_stats_config();

        let write_start_time = Instant::now();

        if !std::path::Path::new(&stats_config.path).exists()
        {
            match create_dir(stats_config.path.to_string())
            {
                Ok(_s) => {},
                Err(e) => {crate::debug(format!("Error creating stats dir {}",e), None)}
            }
        }

        let file_name = stats_config.path.to_string()+"/"+&chrono::offset::Utc::now().to_rfc3339();
        let hits: Vec<Hit> = stats.hits.values().cloned().collect();
        match serde_json::to_string(&hits)
        {
            Ok(s) => {write_file(&file_name, s.as_bytes())},
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

    pub fn digest_message(digest: Digest, from: DateTime<chrono::Utc>) -> String
    {
        let mut msg = String::new(); 

        msg.push_str(format!("Hits since {}\n", from).as_str());
        msg.push_str(format!("Total / Unique: {} / {}\n", digest.total_hits, digest.unique_hits).as_str());

        let mut top_content = String::new();
        for i in 0..3
        {
            if digest.top_three_paths[i].1 > 0
            {
                top_content.push_str(format!("  {} : {}\n", digest.top_three_paths[i].0, digest.top_three_paths[i].1).as_str());
            }
        }
        msg.push_str(format!("Top 3 content:\n{}\n", top_content).as_str());
        msg.push_str(format!("Hits by hour (UTC):\n\n{}", hits_by_hour_text_graph(digest.hits_by_hour_utc, '-', 10)).as_str());

        msg
    }

    pub async fn stats_thread(state: Arc<Mutex<Stats>>)
    {
        loop
        {

            let t = chrono::offset::Utc::now();
            
            {
                let mut stats = state.lock().await;

                let config = match read_config()
                {
                    Some(c) => c,
                    None =>
                    {
                        std::process::exit(1)
                    }
                };

                let stats_config = config.get_stats_config();

                if (t - stats.last_save).num_seconds() > stats_config.save_period_seconds as i64
                {
                    Stats::save(&mut stats);
                }

                if (t - stats.last_digest).num_seconds() > stats_config.digest_period_seconds as i64
                {
                    stats.summary = Self::process_hits(stats_config.path.clone(), stats.last_digest, Some(stats.to_owned()));
                    let msg = Stats::digest_message(stats.summary.clone(), stats.last_digest);
                    match post(config.get_end_point(), msg).await
                    {
                        Ok(_s) => (),
                        Err(e) => {crate::debug(format!("Error posting to discord\n{}", e), None);}
                    }
                    stats.last_digest = t;
                }

                if (t - stats.last_clear).num_seconds() > stats_config.log_files_clear_period_seconds as i64
                {
                    Self::clear_logs(stats_config.path, t);
                    stats.last_clear = t;
                }
            }

            let wait = min(3600, (chrono::Utc::with_ymd_and_hms
            (
                &chrono::Utc, 
                t.year(), 
                t.month(), 
                t.day(), 
                1, 
                0, 
                0
            ).unwrap() + chrono::Duration::days(1) - t).num_seconds()) as u64;
            crate::debug(format!("Sleeping for {}", wait), Some("Statistics".to_string()));
            tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
        }
    }
}

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

pub fn hits_by_hour_text_graph(hits: [u16; 24], symbol: char, size: u8) -> String
{
    let mut graph = String::new();

    let mut top_hour = hits[0];
    for i in 1..23
    {
        top_hour = max(top_hour, hits[i]);
    }

    for (i, h) in hits.iter().enumerate()
    {
        let s = ((size as f64) * (*h as f64) / (top_hour as f64)) as usize;

        graph.push_str(format!("{:0>2}:00", i).as_str());
        graph.push_str(std::iter::repeat(symbol).take(s).collect::<String>().as_str());
        graph.push_str("\n");
    }

    graph
}