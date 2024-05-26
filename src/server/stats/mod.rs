use std::{fs::create_dir, sync::Arc};

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, filesystem::file::File, integrations::discord::post::try_post, task::{next_job_time, schedule_from_option, Task}};

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
            next_run: if schedule.is_none() { None } else { next_job_time(schedule.clone().unwrap()) },
            schedule
        }
    }
}

#[async_trait]
impl Task for StatsSaveTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        let config = Config::load_or_default(CONFIG_PATH);
        {
            let stats = self.state.lock().await;

            if !std::path::Path::new(&config.stats.path).exists()
            {
                match create_dir(config.stats.path.to_string())
                {
                    Ok(_s) => {},
                    Err(e) => {crate::debug(format!("Error creating stats dir {}",e), None)}
                }
            }

            let mut file = StatsFile::new();
            file.load(&stats);
            file.write_bytes();
        }

        self.schedule = schedule_from_option(config.stats.save_schedule.clone());

        self.next_run = match &self.schedule
        {
            Some(s) => next_job_time(s.clone()),
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
            next_run: if schedule.is_none() { None } else { next_job_time(schedule.clone().unwrap()) },
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

            let config = Config::load_or_default(CONFIG_PATH);
            
            stats.summary = process_hits
            (
                config.stats.path.clone(), 
                Some(self.last_run), 
                None, 
                config.stats.top_n_digest, 
                Some(stats.to_owned())
            );

            try_post
            (
                config.notification_endpoint,
                &digest_message(stats.summary.clone(), Some(self.last_run), None)
            ).await;
        }

        let config = Config::load_or_default(CONFIG_PATH);
        self.schedule = schedule_from_option(config.stats.digest_schedule.clone());

        self.next_run = match &self.schedule
        {
            Some(s) => next_job_time(s.clone()),
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