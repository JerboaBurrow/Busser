use std::{path::Path, sync::Arc};

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use git2::Repository;
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, task::{next_job_time, schedule_from_option, Task}};

use super::{clean_and_clone, fast_forward_pull};

pub struct GitRefreshTask
{
    pub lock: Arc<Mutex<()>>,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub schedule: Option<Schedule>
}

impl GitRefreshTask
{
    pub fn new
    (
        lock: Arc<Mutex<()>>,
        schedule: Option<Schedule>
    ) -> GitRefreshTask
    {
        GitRefreshTask 
        {
            lock, 
            last_run: chrono::offset::Utc::now(), 
            next_run: if schedule.is_none() { None } else { next_job_time(schedule.clone().unwrap()) },
            schedule
        }
    }
}

#[async_trait]
impl Task for GitRefreshTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        let _ = self.lock.lock().await;
        let config = Config::load_or_default(CONFIG_PATH);
        if config.git.is_some()
        {
            let git = config.git.unwrap();
            let path = Path::new(&config.content.path);
            if path.is_dir()
            {
                let result = match Repository::open(path)
                {
                    Ok(repo) => fast_forward_pull(repo, &git.branch),
                    Err(e) =>
                    {
                        crate::debug(format!("{}, {:?} is not a git repo", e, path), None);
                        match clean_and_clone(&config.content.path, git.clone())
                        {
                            Ok(repo) => fast_forward_pull(repo, &git.branch),
                            Err(e) => Err(e)
                        }
                    }
                };

                if result.is_err()
                {
                    crate::debug(format!("{:?}", result.err()), None);
                }
            }
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
        "Git refresh".to_string()
    }
}