use std::{cmp::{max, min}, collections::HashMap};

use chrono::{DateTime, Timelike};

use crate::{config::{read_config, CONFIG_PATH}, content::is_page, util::matches_one};

use super::hits::{collect_hits, HitStats};

#[derive(Debug, Clone)]
pub struct Digest
{
    pub top_hitters: Vec<(String, u16)>,
    pub top_pages: Vec<(String, u16)>,
    pub top_resources: Vec<(String, u16)>,
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
            top_hitters: vec![],
            top_pages: vec![],
            top_resources: vec![],
            hits_by_hour_utc: [0;24],
            total_hits: 0,
            unique_hits: 0
        }
    }
}

pub fn process_hits(path: String, from: Option<DateTime<chrono::Utc>>, to: Option<DateTime<chrono::Utc>>, top_n: Option<usize>, stats: Option<HitStats>) -> Digest
{

    let n = match top_n
    {
        Some(n) => n,
        None => 3
    };

    let mut digest = Digest::new();

    let (ignore_patterns, domain) = match read_config(CONFIG_PATH)
    {
        Some(c) => 
        {
            match c.stats.ignore_regexes
            {
                Some(i) => (i, c.domain),
                None => (vec![], c.domain)
            }
        },
        None => (vec![], "127.0.0.1".to_string())
    };
    let mut hitters: HashMap<String, u16> = HashMap::new();
    let mut pages: HashMap<String, u16> = HashMap::new();
    let mut resources: HashMap<String, u16> = HashMap::new();

    for hit in collect_hits(path, stats, from, to)
    {
        if matches_one(&hit.path, &ignore_patterns)
        {
            continue
        }  

        match hitters.contains_key(&hit.ip_hash)
        {
            true => {hitters.insert(hit.ip_hash.clone(), hit.count+hitters[&hit.ip_hash]);},
            false => 
            {
                hitters.insert(hit.ip_hash, hit.count);
                digest.unique_hits += 1;
            }
        }

        if is_page(&hit.path, &domain)
        {
            match pages.contains_key(&hit.path)
            {
                true => {pages.insert(hit.path.clone(), hit.count+pages[&hit.path]);},
                false => {pages.insert(hit.path, hit.count);}
            }
        }
        else
        {
            match resources.contains_key(&hit.path)
            {
                true => {resources.insert(hit.path.clone(), hit.count+resources[&hit.path]);},
                false => {resources.insert(hit.path, hit.count);}
            }
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

    let mut all_hitters: Vec<(String, u16)> = hitters.into_iter().collect();
    let mut all_pages: Vec<(String, u16)> = pages.into_iter().collect();
    let mut all_resources: Vec<(String, u16)> = resources.into_iter().collect();

    for data in vec![&mut all_hitters, &mut all_pages, &mut all_resources]
    {
        data.sort_by(|a: &(String, u16), b: &(String, u16)| a.1.cmp(&b.1));
        data.reverse();
    }

    digest.top_hitters = (0..n).map(|_i| ("".to_string(), 0)).collect();

    for i in 0..n
    {
        if i < all_hitters.len()
        {
            digest.top_hitters[i] = all_hitters[i].clone();
        }
        else
        {
            digest.top_hitters[i] = ("".to_string(), 0);
        }
    }

    digest.top_pages = (0..n).map(|_i| ("".to_string(), 0)).collect();
    digest.top_resources = (0..n).map(|_i| ("".to_string(), 0)).collect();

    for i in 0..n
    {
        if i < all_pages.len()
        {
            digest.top_pages[i] = all_pages[i].clone();
        }
        else
        {
            digest.top_pages[i] = ("".to_string(), 0);
        }

        if i < all_resources.len()
        {
            digest.top_resources[i] = all_resources[i].clone();
        }
        else
        {
            digest.top_resources[i] = ("".to_string(), 0);
        }
    }

    digest

}

pub fn digest_message(digest: Digest, from: Option<DateTime<chrono::Utc>>, to: Option<DateTime<chrono::Utc>>) -> String
{
    let mut msg = String::new(); 

    match from
    {
        Some(s) => 
        {
            match to
            {
                Some(t) =>
                {
                    msg.push_str(format!("Hits from {} to {}\n", s, t).as_str());
                },
                None => 
                {
                    msg.push_str(format!("Hits since {}\n", s).as_str());
                }
            }  
        },
        None => {msg.push_str("All hits\n");}
    };
    
    msg.push_str(format!("Total / Unique: {} / {}\n", digest.total_hits, digest.unique_hits).as_str());

    let mut top_resources = String::new();
    let mut top_pages = String::new();
    let n = min(digest.top_resources.len(), digest.top_pages.len());
    for i in 0..n
    {
        if digest.top_resources[i].1 > 0
        {
            top_resources.push_str(format!("  {} : {}\n", digest.top_resources[i].0, digest.top_resources[i].1).as_str());
        }

        if digest.top_pages[i].1 > 0
        {
            top_pages.push_str(format!("  {} : {}\n", digest.top_pages[i].0, digest.top_pages[i].1).as_str());
        }
    }
    msg.push_str(format!("Top {n} pages:\n{}\n", top_pages).as_str());
    msg.push_str(format!("Top {n} resources:\n{}\n", top_resources).as_str());
    msg.push_str(format!("Hits by hour (UTC):\n\n{}", hits_by_hour_text_graph(digest.hits_by_hour_utc, '-', 10)).as_str());

    msg
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