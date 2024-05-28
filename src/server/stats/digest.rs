use std::{cmp::{max, min}, collections::HashMap};

use chrono::{DateTime, Timelike};

use crate::{config::Config, content::is_page, util::matches_one};

use super::hits::{collect_hits, HitStats};

/// A digest of hit statistics
#[derive(Debug, Clone, PartialEq)]
pub struct Digest
{
    pub top_hitters: Vec<(String, usize)>,
    pub top_pages: Vec<(String, usize)>,
    pub top_resources: Vec<(String, usize)>,
    pub hits_by_hour_utc: [usize; 24],
    pub total_hits: usize,
    pub unique_hits: usize
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

/// Collect hits cached and from local files into a [Digest]
pub fn process_hits
(
    from: Option<DateTime<chrono::Utc>>, 
    to: Option<DateTime<chrono::Utc>>, 
    config: &Config,
    stats: Option<HitStats>
) -> Digest
{

    let n = match config.stats.top_n_digest
    {
        Some(n) => n,
        None => 3
    };

    let mut digest = Digest::new();

    let (ignore_patterns, domain) = match config.stats.ignore_regexes.clone()
    {
        Some(r) => (r, config.domain.clone()),
        None => (vec![], config.domain.clone())
    };

    let mut hitters: HashMap<String, usize> = HashMap::new();
    let mut pages: HashMap<String, usize> = HashMap::new();
    let mut resources: HashMap<String, usize> = HashMap::new();

    for hit in collect_hits(stats, from, to, &config)
    {
        if matches_one(&hit.path, &ignore_patterns)
        {
            continue
        }  

        match hitters.contains_key(&hit.ip_hash)
        {
            true => {hitters.insert(hit.ip_hash.clone(), hit.count()+hitters[&hit.ip_hash]);},
            false => 
            {
                hitters.insert(hit.ip_hash.clone(), hit.count());
                digest.unique_hits += 1;
            }
        }

        if is_page(&hit.path, &domain)
        {
            match pages.contains_key(&hit.path)
            {
                true => {pages.insert(hit.path.clone(), hit.count()+pages[&hit.path]);},
                false => {pages.insert(hit.path.clone(), hit.count());}
            }
        }
        else
        {
            match resources.contains_key(&hit.path)
            {
                true => {resources.insert(hit.path.clone(), hit.count()+resources[&hit.path]);},
                false => {resources.insert(hit.path.clone(), hit.count());}
            }
        }

        digest.total_hits += hit.count();

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

    let mut all_hitters: Vec<(String, usize)> = hitters.into_iter().collect();
    let mut all_pages: Vec<(String, usize)> = pages.into_iter().collect();
    let mut all_resources: Vec<(String, usize)> = resources.into_iter().collect();

    for data in vec![&mut all_hitters, &mut all_pages, &mut all_resources]
    {
        data.sort_by(|a: &(String, usize), b: &(String, usize)| a.1.cmp(&b.1));
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

/// Post a [Digest] as a formatted message to Discord
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

pub fn hits_by_hour_text_graph(hits: [usize; 24], symbol: char, size: u8) -> String
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