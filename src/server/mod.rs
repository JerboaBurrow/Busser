use std::{sync::Arc, time::Duration};

use axum::async_trait;
use chrono::{DateTime, Utc};
use cron::Schedule;
use tokio::sync::Mutex;

use crate::{config::{Config, CONFIG_PATH}, content::sitemap::{self, SiteMap}, task::{next_job_time, schedule_from_option, Task}};

use self::https::Server;

pub mod https;
pub mod http;
pub mod api;
pub mod throttle;
pub mod stats;

// pub struct ServerRefresh
// {
//     pub state: Server,
//     pub last_run: DateTime<Utc>,
//     pub next_run: Option<DateTime<Utc>>,
//     pub schedule: Option<Schedule>,
//     insert_tag: bool
// }

// impl ServerRefresh
// {
//     pub fn new
//     (
//         state: Server,
//         schedule: Option<Schedule>,
//         insert_tag: bool
//     ) -> ServerRefresh
//     {
//         ServerRefresh 
//         { 
//             state, 
//             last_run: chrono::offset::Utc::now(), 
//             next_run: if schedule.is_none() { None } else { next_job_time(schedule.clone().unwrap()) },
//             schedule,
//             insert_tag
//         }
//     }
// }

// #[async_trait]
// impl Task for ServerRefresh
// {
//     async fn run(&mut self) -> Result<(), crate::task::TaskError> 
//     {
//         let config = Config::load_or_default(CONFIG_PATH);
//         let sitemap = SiteMap::from_config(&config, self.insert_tag);

//         if sitemap.get_hash() != self.state.get_hash()
//         {
//             let new_server = Server::new
//             (
//                 0,0,0,0,
//                 SiteMap::from_config(&config, self.insert_tag)
//             );
//             self.state.shutdown(Some(Duration::from_secs(60))).await;
//             self.state = new_server;
//             self.state.serve().await;
//         }

//         self.schedule = schedule_from_option(config.stats.save_schedule.clone());
//         self.next_run = match &self.schedule
//         {
//             Some(s) => next_job_time(s.clone()),
//             None => None
//         };
//         self.last_run = chrono::offset::Utc::now();
//         Ok(())
//     }

//     fn next(&mut self) -> Option<chrono::prelude::DateTime<chrono::prelude::Utc>> 
//     {
//         self.next_run
//     }

//     fn runnable(&self) -> bool 
//     {
//         match self.next_run
//         {
//             Some(t) => chrono::offset::Utc::now() > t,
//             None => false
//         }
//     }

//     fn info(&self) -> String 
//     {
//         "Server refresh".to_string()
//     }
// }