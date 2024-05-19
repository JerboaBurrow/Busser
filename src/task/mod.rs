use core::fmt;
use std::{cmp::min, collections::HashMap, sync::Arc};

use axum::async_trait;
use chrono::{DateTime, Local, Utc};
use tokio::{spawn, sync::Mutex};
use uuid::Uuid;

pub const DEFAULT_WAIT: tokio::time::Duration = tokio::time::Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct TaskError
{
    pub why: String
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.why)
    }
}

/// A trait for running asynchronous tasks
/// See usage in [crate::task::TaskPool]
#[async_trait]
pub trait Task
{
    async fn run(&mut self) -> Result<(), TaskError>;
    fn next(&self) -> Option<DateTime<Utc>>;
    fn runnable(&self) -> bool;
    fn info(&self) -> String;
}

/// A pool of tasks to be executed 
/// - [Task]s are added to the pool using [TaskPool::add] 
/// - [TaskPool::run] loops continuously (with sleeps) running tasks when they are available
pub struct TaskPool
{
    tasks: HashMap<Uuid, Arc<Mutex<Box<dyn Task + Send>>>>,
    closing: Arc<Mutex<bool>>
}

impl TaskPool
{
    pub fn new() -> TaskPool
    {
        TaskPool { tasks: HashMap::new(), closing: Arc::new(Mutex::new(false)) }
    }

    pub fn ntasks(&self) -> usize { self.tasks.len() }

    pub fn add(&mut self, task: Box<dyn Task + Send>) -> Uuid
    {
        let id = Uuid::new_v4();
        self.tasks.insert(id, Arc::new(Mutex::new(task)));
        id
    }

    pub fn remove(&mut self, id: &Uuid)
    {
        if self.tasks.contains_key(id)
        {
            self.tasks.remove(id);
        }
    }

    pub async fn stop(&mut self)
    {
        *self.closing.lock().await = true; 
    }
    
    /// Returns a duration to wait for the next runnable process
    ///  and an information string about that process including
    ///  a [DateTime<Utc>] when it will be run
    pub async fn waiting_for(&self) -> (tokio::time::Duration, String) 
    {
        if self.tasks.len() == 0
        {
            return (DEFAULT_WAIT, "None".to_string())
        }

        let now = chrono::offset::Utc::now();
        let mut wait = u64::MAX;
        let mut info = String::new();

        for (id, task_lock) in &self.tasks
        {
            let task = task_lock.lock().await;
            match task.next()
            {
                Some(d) => 
                {
                    let dt = (d-now).num_seconds();
                    if dt <= 0
                    {
                        return (tokio::time::Duration::ZERO, format!("Task {}, {}. Now", id, task.info()));
                    }
                    else
                    {
                        if (dt as u64) < wait
                        {
                            let local_time: DateTime<Local> = DateTime::from(d);
                            info = format!("Task {}, {}. At {}", id, task.info(), local_time);
                            wait = min(wait, dt as u64);
                        }
                    }
                },
                None => continue
            }
        }

        (tokio::time::Duration::from_secs(wait), info)
    }

    pub fn run(self)
    {
        spawn(
            async move {
                loop
                {
                    if self.closing.lock().await.to_owned()
                    {
                        break;
                    }
                    for (id, task_lock) in &self.tasks
                    {
                        let mut task = task_lock.lock().await;
                        match task.runnable()
                        {
                            true => 
                            {
                                crate::debug(format!("Running task {}\n {}", id, task.info()), None); 
                                match task.run().await
                                {
                                    Ok(()) => (),
                                    Err(e) => {crate::debug(format!("Task {}, exited with error {}", task.info(), e), None)}
                                }
                            },
                            false => continue
                        }
                    }
                    let (wait, info) = self.waiting_for().await;
                    crate::debug(format!("Next task\n  {}\n Waiting for {}s", info, wait.as_secs()), None);
                    tokio::time::sleep(wait).await;
                }
            }
        );
    }
}