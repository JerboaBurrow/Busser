use std::{path::Path, sync::Arc, time::SystemTime};

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use git2::Repository;

use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, integrations::discord::post::try_post, task::{next_job_time, schedule_from_option, Task}};

use super::{clean_and_clone, fast_forward_pull, HeadInfo};

pub struct GitRefreshTask
{
    pub lock: Arc<Mutex<SystemTime>>,
    pub last_run: DateTime<Utc>,
    pub next_run: Option<DateTime<Utc>>,
    pub schedule: Option<Schedule>
}

impl GitRefreshTask
{
    pub fn new
    (
        lock: Arc<Mutex<SystemTime>>,
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

    /// Attempt a fast forward pull of the repo in [crate::config::ContentConfig::path]
    ///   clone if it is not there. Does nothing if [crate::config::GitConfig] is none
    pub fn pull(config: &Config) -> Option<HeadInfo>
    {
        
        if config.git.is_some()
        {
            let git = config.git.clone().unwrap();
            let path = Path::new(&config.content.path);
            if path.is_dir()
            {
                let result = match Repository::open(path)
                {
                    Ok(repo) => fast_forward_pull(repo, git),
                    Err(e) =>
                    {
                        crate::debug(format!("{}, {:?} is not a git repo", e, path), Some("GIT"));
                        match clean_and_clone(&config.content.path, git.clone())
                        {
                            Ok(_) => Ok(None),
                            Err(e) => Err(e)
                        }
                    }
                };

                if result.is_err()
                {
                    crate::debug(format!("{:?}", result.err()), Some("GIT"));
                }
                else
                {
                    return result.unwrap()
                }
            }
            else
            {
                let result = match clean_and_clone(&config.content.path, git.clone())
                {
                    Ok(repo) => fast_forward_pull(repo, git),
                    Err(e) => Err(e)
                };
                if result.is_err()
                {
                    crate::debug(format!("{:?}", result.err()), Some("GIT"));
                }
            }
        }
        None
    }

    /// Send a discord message with [HeadInfo] if it is Some
    pub async fn notify_pull(info: Option<HeadInfo>, config: &Config)
    {
        match Self::head_info_to_message(info, config)
        {
            Some(msg) =>
            {
                crate::debug(msg.clone(), Some("GIT"));
                try_post
                (
                    config.notification_endpoint.clone(), 
                    &msg
                ).await;
            },
            None => {}
        }
        
    }

    /// Format a notification from a head info object and config. None if info is None
    pub fn head_info_to_message(info: Option<HeadInfo>, config: &Config) -> Option<String>
    {
        match info
        {
            Some(info) =>
            {
                Some(format!
                (
                    "Checked out new commit for {}:\n```\n {}\n by {}\n at {}\n```",
                    config.domain,
                    info.hash,
                    info.author_name,
                    info.datetime
                ))
            },
            None => None
        }
    }
}

#[async_trait]
impl Task for GitRefreshTask
{
    async fn run(&mut self) -> Result<(), crate::task::TaskError> 
    {
        let mut time = self.lock.lock().await;
        let config = Config::load_or_default(CONFIG_PATH);
        GitRefreshTask::notify_pull(GitRefreshTask::pull(&config), &config).await;
        *time = SystemTime::now();

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