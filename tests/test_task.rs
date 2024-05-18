mod common;

#[cfg(test)]
mod task
{
    use std::sync::Arc;

    use busser::{server::stats::{hits::HitStats, StatsSaveTask}, task::{TaskPool, DEFAULT_WAIT}};
    use tokio::sync::Mutex;


    #[tokio::test]
    async fn test_taskpool()
    {
        let mut pool = TaskPool::new();

        assert_eq!(pool.ntasks(), 0);

        let (wait, _info) = pool.waiting_for().await;
        assert_eq!(wait, DEFAULT_WAIT);

        let stats = Arc::new(Mutex::new(HitStats::new()));
        let task = StatsSaveTask{ state: stats, last_run: chrono::offset::Utc::now() };

        pool.add(Box::new(task));

        assert_eq!(pool.ntasks(), 1);

        let (wait, _info) = pool.waiting_for().await;
        assert!(wait > tokio::time::Duration::ZERO);
    }
}