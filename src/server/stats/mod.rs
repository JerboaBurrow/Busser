use std::{str::FromStr, sync::Arc};

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use tokio::sync::Mutex;

use crate::{config::{read_config, Config, CONFIG_PATH}, filesystem::file::File, integrations::discord::post::post, task::Task};

use self::{digest::{digest_message, process_hits}, file::StatsFile, hits::HitStats};

pub mod hits;
pub mod digest;
pub mod file;

/// A task to periodically save HitStats to disk
/// See [crate::task::Task] and [crate::task::TaskPool]
pub struct StatsSaveTask
{
    pub state: Arc<Mutex<HitStats>>,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>> 
}

#[async_trait]
impl Task for StatsSaveTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        {
            let stats = self.state.lock().await;

            let mut file = StatsFile::new();
            file.load(&stats);
            file.write_bytes();
        }

        self.last_run = chrono::offset::Utc::now();
        self.next_run = None;
        Ok(())
    }

    fn next(&mut self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
    {
        if self.next_run.is_none()
        {
            let config = Config::load_or_default(CONFIG_PATH);

            self.next_run = match config.stats.save_schedule.clone()
            {
                Some(s) => next_job_time(&s),
                None => None
            };
            
        }
        self.next_run
    }

    fn runnable(&self) -> bool 
    {
        match self.next_run
        {
            Some(t) => chrono::offset::Utc::now() > t,
            None => false
        }
    }

    fn info(&self) -> String 
    {
        "Statistics saving".to_string()
    }
}

/// A task to periodically send HitStats digests discord
/// See [crate::task::Task] and [crate::task::TaskPool]
pub struct StatsDigestTask
{
    pub state: Arc<Mutex<HitStats>>,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>> 
}

#[async_trait]
impl Task for StatsDigestTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
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
                Some(self.last_run), 
                None, 
                config.stats.top_n_digest, 
                Some(stats.to_owned())
            );

            let msg = digest_message(stats.summary.clone(), Some(self.last_run), None);
            match config.notification_endpoint 
            {
                Some(endpoint) => match post(&endpoint, msg).await
                    {
                        Ok(_s) => (),
                        Err(e) => {crate::debug(format!("Error posting to discord\n{}", e), None);}
                    },
                None => ()
            }
        }

        self.next_run = None;
        self.last_run = chrono::offset::Utc::now();
        Ok(())
    }

    fn next(&mut self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
    {
        if self.next_run.is_none()
        {
            let config = Config::load_or_default(CONFIG_PATH);

            self.next_run = match config.stats.save_schedule.clone()
            {
                Some(s) => next_job_time(&s),
                None => None
            };
            
        }
        self.next_run
    }

    fn runnable(&self) -> bool 
    {
        match self.next_run
        {
            Some(t) => chrono::offset::Utc::now() > t,
            None => false
        }
    }

    fn info(&self) -> String 
    {
        "Statistics digest".to_string()
    }
}

pub fn next_job_time(cron: &str) -> Option<DateTime<Utc>>
{
    match Schedule::from_str(cron) 
    {
        Ok(s) => 
        {
            let jobs: Vec<DateTime<Utc>> = s.upcoming(Utc).take(1).collect();
            jobs.first().copied()
        }, 
        Err(e) => {crate::debug(format!("Could not parse cron string, {:?}, for digest schedule\n{}", cron, e), None); None}
    }
}