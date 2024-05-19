mod common;

#[cfg(test)]
mod task
{
    use std::{str::FromStr, sync::Arc};

    use busser::{server::stats::{hits::HitStats, StatsDigestTask, StatsSaveTask}, task::{Task, TaskPool, DEFAULT_WAIT}};
    use chrono::Timelike;
    use cron::Schedule;
    use tokio::sync::Mutex;


    #[tokio::test]
    async fn test_taskpool()
    {
        let mut pool = TaskPool::new();

        assert_eq!(pool.ntasks(), 0);

        let (wait, _info) = pool.waiting_for().await;
        assert_eq!(wait, DEFAULT_WAIT);

        let stats = Arc::new(Mutex::new(HitStats::new()));
        let task = StatsSaveTask{ state: stats.clone(), last_run: chrono::offset::Utc::now(), next_run: None, schedule: None};
        assert_eq!(task.runnable(), false);
        assert_eq!(task.info(), "Statistics saving".to_string());

        pool.add(Box::new(task));

        assert_eq!(pool.ntasks(), 1);

        let (wait, _info) = pool.waiting_for().await;
        assert!(wait > tokio::time::Duration::ZERO);
        assert_eq!(wait, DEFAULT_WAIT);

        let hour = chrono::offset::Utc::now().hour();
        let schedule = format!("0 0 {} * * * *", (hour+2)%24);

        let task = StatsDigestTask::new
        (
            stats, 
            Some(Schedule::from_str(&schedule).unwrap())
        );

        assert_eq!(task.runnable(), false);
        assert_eq!(task.info(), "Statistics digest".to_string());

        pool.add(Box::new(task));

        assert_eq!(pool.ntasks(), 2);

        let (wait, _info) = pool.waiting_for().await;
        println!("{:?}", wait);
        assert!(wait > DEFAULT_WAIT);
    }
}