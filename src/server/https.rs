use crate::
{
    config::{read_config, CONFIG_PATH}, content::sitemap::SiteMap, integrations::{git::refresh::GitRefreshTask, github::filter_github}, server::throttle::{handle_throttle, IpThrottler}, task::{schedule_from_option, TaskPool}, CRAB
};

use core::time;
use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, time::SystemTime};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::
{
    middleware, 
    Router
};
use axum_server::{tls_rustls::RustlsConfig, Handle};

use super::{api::{stats::StatsDigest, ApiRequest}, stats::{hits::{log_stats, HitStats}, StatsSaveTask, StatsDigestTask}};

/// An https server that reads a directory configured with [Config]
/// ```.html``` pages and resources, then serves them.
pub struct Server
{
    addr: SocketAddr,
    router: Router,
    handle: Handle,
    domain: String,
    cert_path: String,
    key_path: String
}

/// Checks a uri has a leading /, adds it if not
pub fn parse_uri(uri: String, path: String) -> String
{
    if uri.starts_with(&path)
    {
        uri.replace(&path, "/")
    }
    else if uri.starts_with("/")
    {
        uri
    }
    else
    {
        "/".to_string()+&uri
    }
}

impl Server 
{
    pub fn new 
    (
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        sitemap: SiteMap
    ) 
    -> (Server, TaskPool)
    {

        let config = match read_config(CONFIG_PATH)
        {
            Some(c) => c,
            None =>
            {
                std::process::exit(1)
            }
        };

        let requests: IpThrottler = IpThrottler::new
        (
            config.throttle.max_requests_per_second, 
            config.throttle.timeout_millis,
            config.throttle.clear_period_seconds
        );

        let throttle_state = Arc::new(Mutex::new(requests));
        
        let mut router: Router = sitemap.into();

        let stats = Arc::new(Mutex::new(
            HitStats::new()
        ));

        router = router.layer(middleware::from_fn_with_state(stats.clone(), log_stats));
        router = router.layer(middleware::from_fn_with_state(throttle_state.clone(), handle_throttle));

        router = router.layer(middleware::from_fn_with_state(Some(stats.clone()), StatsDigest::filter));

        let repo_mutex = Arc::new(Mutex::new(SystemTime::now()));

        router = router.layer(middleware::from_fn_with_state(repo_mutex.clone(), filter_github));

        let server = Server
        {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a,b,c,d)), config.port_https),
            router,
            handle: Handle::new(),
            cert_path: config.cert_path,
            key_path: config.key_path,
            domain: config.domain
        };

        let mut tasks = TaskPool::new();

        tasks.add
        (
            Box::new
            (
                StatsSaveTask::new
                (
                    stats.clone(), 
                    schedule_from_option(config.stats.save_schedule.clone())
                ) 
            )
        );

        tasks.add
        (
            Box::new
            (
                StatsDigestTask::new
                (
                    stats.clone(), 
                    schedule_from_option(config.stats.digest_schedule.clone())
                ) 
            )
        );

        if config.git.is_some()
        {
            tasks.add
            (
                Box::new
                (
                    GitRefreshTask::new
                    (
                        repo_mutex,
                        schedule_from_option(config.git.unwrap().checkout_schedule)
                    )
                )
            );
        }

        (server, tasks)
    }

    pub fn get_addr(self: Server) -> SocketAddr
    {
        self.addr
    }

    pub fn get_handle(&self) -> Handle
    {
        self.handle.clone()
    }

    pub async fn serve(self)
    {

        // configure https

        let cert_path = self.cert_path.clone();
        let key_path = self.key_path.clone();

        let config = match RustlsConfig::from_pem_file(
            PathBuf::from(cert_path.clone()),
            PathBuf::from(key_path.clone())
        )
        .await
        {
            Ok(c) => c,
            Err(e) => 
            {
                println!("error while reading certificates in {} and key {}\n{}", cert_path, key_path, e);
                std::process::exit(1);
            }
        };

        let domain = if self.domain.contains("https://")
        {
            self.domain.clone()
        }
        else
        {
            format!("https://{}", self.domain)
        };

        println!("Checkout your cool site, at {} {}!", domain, String::from_utf8(CRAB.to_vec()).unwrap());
        if domain != "https://127.0.0.1"
        {
            println!("(or https://127.0.0.1)");
        }
        
        axum_server::bind_rustls(self.addr, config)
        .handle(self.handle.clone())
        .serve(self.router.clone().into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    }

    pub async fn shutdown(&mut self, graceful: Option<time::Duration>)
    {
        match graceful
        {
            // not sure if graceful_shutdown defaults to shutdown if None is passed
            Some(_) => self.handle.graceful_shutdown(graceful),
            None => self.handle.shutdown()
        }
    }

}