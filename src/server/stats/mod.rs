use std::{cmp::min, sync::Arc};

use chrono::{Datelike, TimeZone};
use tokio::sync::Mutex;

use crate::{config::{read_config, CONFIG_PATH}, web::discord::request::post::post};

use self::{digest::{digest_message, process_hits}, hits::{archive, save, HitStats}};

pub mod hits;
pub mod digest;

pub async fn stats_thread(state: Arc<Mutex<HitStats>>)
{
    loop
    {

        let t = chrono::offset::Utc::now();
        
        {
            let mut stats = state.lock().await;

            let config = match read_config(CONFIG_PATH)
            {
                Some(c) => c,
                None =>
                {
                    std::process::exit(1)
                }
            };

            let stats_config = config.stats;

            if (t - stats.last_save).num_seconds() > stats_config.save_period_seconds as i64
            {
                save(&mut stats);
            }

            if (t - stats.last_digest).num_seconds() > stats_config.digest_period_seconds as i64
            {
                stats.summary = process_hits(stats_config.path.clone(), Some(stats.last_digest), None, stats_config.top_n_digest, Some(stats.to_owned()));
                let msg = digest_message(stats.summary.clone(), Some(stats.last_digest), None);
                match post(&config.notification_endpoint, msg).await
                {
                    Ok(_s) => (),
                    Err(e) => {crate::debug(format!("Error posting to discord\n{}", e), None);}
                }
                stats.last_digest = t;
            }

            if (t - stats.last_clear).num_seconds() > stats_config.log_files_clear_period_seconds as i64
            {
                archive();
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