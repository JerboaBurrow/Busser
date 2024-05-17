use std::{cmp::min, sync::Arc};

use tokio::sync::Mutex;

use crate::{config::{read_config, CONFIG_PATH}, integrations::discord::post::post};

use self::{digest::{digest_message, process_hits}, hits::{save, HitStats}};

pub mod hits;
pub mod digest;

pub async fn stats_thread(state: Arc<Mutex<HitStats>>)
{
    loop
    {

        let time: chrono::prelude::DateTime<chrono::prelude::Utc> = chrono::offset::Utc::now();

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

        let time_until_save = (time - stats.last_save).num_seconds() - stats_config.save_period_seconds as i64;
        let time_until_digest = (time - stats.last_digest).num_seconds() - stats_config.digest_period_seconds as i64;

        if time_until_save <= 0
        {
            save(&mut stats);
        }

        if time_until_digest <= 0
        {
            stats.summary = process_hits(stats_config.path.clone(), Some(stats.last_digest), None, stats_config.top_n_digest, Some(stats.to_owned()));
            let msg = digest_message(stats.summary.clone(), Some(stats.last_digest), None);
            match config.notification_endpoint 
            {
                Some(endpoint) => match post(&endpoint, msg).await
                    {
                        Ok(_s) => (),
                        Err(e) => {crate::debug(format!("Error posting to discord\n{}", e), None);}
                    },
                None => ()
            }
            stats.last_digest = time;
        }

        let time_left = min(time_until_digest, time_until_save);

        let wait_seconds = match time_left > 0
        {
            true => time_left as u64,
            false => min(stats_config.save_period_seconds, stats_config.digest_period_seconds)
        };

        crate::debug(format!("Sleeping for {}", wait_seconds), Some("Statistics".to_string()));
        tokio::time::sleep(std::time::Duration::from_secs(wait_seconds)).await;
    }
}