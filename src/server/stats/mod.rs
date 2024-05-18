use std::sync::Arc;

use axum::async_trait;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::Mutex;

use crate::{config::{read_config, Config, CONFIG_PATH}, integrations::discord::post::post, task::Task};

use self::{digest::{digest_message, process_hits}, hits::{save, HitStats}};

pub mod hits;
pub mod digest;

/// A task to periodically save HitStats to disk
/// See [crate::task::Task] and [crate::task::TaskPool]
pub struct StatsSaveTask
{
    pub state: Arc<Mutex<HitStats>>,
    pub last_run: DateTime<Utc>
}

#[async_trait]
impl Task for StatsSaveTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        let mut stats = self.state.lock().await;
        save(&mut stats);
        self.last_run = chrono::offset::Utc::now();
        Ok(())
    }

    fn next(&self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
    {
        let config = match read_config(CONFIG_PATH)
        {
            Some(c) => c,
            None =>
            {
                Config::default()
            }
        };

        let time: chrono::prelude::DateTime<chrono::prelude::Utc> = chrono::offset::Utc::now();
        let time_until_save = config.stats.save_period_seconds as i64 - (time - self.last_run).num_seconds();

        if time_until_save < 0
        {
            Some(time)
        }
        else
        {
            Some(self.last_run + Duration::seconds(time_until_save))
        }
    }

    fn runnable(&self) -> bool 
    {
        chrono::offset::Utc::now() > self.next().unwrap()
    }

    fn info(&self) -> String 
    {
        "Statistics saveing".to_string()
    }
}

/// A task to periodically send HitStats digests discord
/// See [crate::task::Task] and [crate::task::TaskPool]
pub struct StatsDigestTask
{
    pub state: Arc<Mutex<HitStats>>,
    pub last_run: DateTime<Utc>
}

#[async_trait]
impl Task for StatsDigestTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        let mut stats = self.state.lock().await;

        let config = match read_config(CONFIG_PATH)
        {
            Some(c) => c,
            None =>
            {
                Config::default()
            }
        };
        
        stats.summary = process_hits
        (
            config.stats.path.clone(), 
            Some(stats.last_digest), 
            None, 
            config.stats.top_n_digest, 
            Some(stats.to_owned())
        );

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

        self.last_run = chrono::offset::Utc::now();
        Ok(())
    }

    fn next(&self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
    {
        let config = match read_config(CONFIG_PATH)
        {
            Some(c) => c,
            None =>
            {
                Config::default()
            }
        };

        let time: chrono::prelude::DateTime<chrono::prelude::Utc> = chrono::offset::Utc::now();
        let time_until_digest = config.stats.digest_period_seconds as i64 - (time - self.last_run).num_seconds();

        if time_until_digest < 0
        {
            Some(time)
        }
        else
        {
            Some(self.last_run + Duration::seconds(time_until_digest))
        }
    }

    fn runnable(&self) -> bool 
    {
        chrono::offset::Utc::now() > self.next().unwrap()
    }

    fn info(&self) -> String 
    {
        "Statistics digest".to_string()
    }
}