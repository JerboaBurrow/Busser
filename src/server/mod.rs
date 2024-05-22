use std::sync::Arc;

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, task::{next_job_time, schedule_from_option, Task}};

use self::https::Server;

pub mod https;
pub mod http;
pub mod api;
pub mod throttle;
pub mod stats;

pub struct ServerRefresh
{
    pub state: Arc<Mutex<Server>>,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub schedule: Option<Schedule>
}

impl ServerRefresh
{
    pub fn new
    (
        state: Arc<Mutex<Server>>,
        schedule: Option<Schedule>
    ) -> ServerRefresh
    {
        ServerRefresh 
        { 
            state, 
            last_run: chrono::offset::Utc::now(), 
            next_run: if schedule.is_none() { None } else { next_job_time(schedule.clone().unwrap()) },
            schedule
        }
    }
}

#[async_trait]
impl Task for ServerRefresh
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        let config = Config::load_or_default(CONFIG_PATH);
        {
            let server = self.state.lock().await;
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
        "Server refresh".to_string()
    }
}