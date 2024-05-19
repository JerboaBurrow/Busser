use std::sync::Arc;

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use tokio::sync::Mutex;

use crate::{config::{read_config, Config, CONFIG_PATH}, filesystem::file::File, integrations::discord::post::post, task::{next_job_time, schedule_from_option, Task}};

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
    pub next_run: Option<DateTime<Utc>>,
    pub schedule: Option<Schedule>
}

impl StatsSaveTask
{
    pub fn new
    (
        state: Arc<Mutex<HitStats>>,
        schedule: Option<Schedule>
    ) -> StatsSaveTask
    {
        StatsSaveTask 
        { 
            state, 
            last_run: chrono::offset::Utc::now(), 
            next_run: if schedule.is_none() { None } else { next_job_time(None, schedule.clone().unwrap()) },
            schedule
        }
    }
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

        let config = Config::load_or_default(CONFIG_PATH);
        self.schedule = schedule_from_option(config.stats.save_schedule.clone());

        self.next_run = match &self.schedule
        {
            Some(s) => next_job_time(self.next_run, s.clone()),
            None => None
        };

        self.last_run = chrono::offset::Utc::now();
        Ok(())
    }

    fn next(&mut self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
    {
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
    pub schedule: Option<Schedule>,
    pub next_run: Option<DateTime<Utc>>
}

impl StatsDigestTask
{
    pub fn new
    (
        state: Arc<Mutex<HitStats>>,
        schedule: Option<Schedule>
    ) -> StatsDigestTask
    {
        StatsDigestTask 
        { 
            state, 
            last_run: chrono::offset::Utc::now(), 
            next_run: if schedule.is_none() { None } else { next_job_time(None, schedule.clone().unwrap()) },
            schedule
        }
    }
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

        let config = Config::load_or_default(CONFIG_PATH);
        self.schedule = schedule_from_option(config.stats.digest_schedule.clone());

        self.next_run = match &self.schedule
        {
            Some(s) => next_job_time(self.next_run, s.clone()),
            None => None
        };

        self.last_run = chrono::offset::Utc::now();
        Ok(())
    }

    fn next(&mut self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
    {
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