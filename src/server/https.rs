use crate::
{
    config::{read_config, Config}, content::{filter::ContentFilter, sitemap::SiteMap, Content}, web::{stats::{log_stats, Digest, Stats}, 
    throttle::{handle_throttle, IpThrottler}}, CRAB
};

use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr}};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::{spawn, sync::Mutex};

use axum::
{
    middleware, 
    Router
};
use axum_server::tls_rustls::RustlsConfig;

use super::api::{stats::StatsDigest, ApiRequest};

/// An https server that reads a directory configured with [Config]
/// ```.html``` pages and resources, then serves them.
/// # Example
/// ```no_run
/// use busser::server::https::Server;
/// #[tokio::main]
/// async fn main() 
/// {
///     let server = Server::new(0,0,0,0,true);
///     server.serve().await;
/// }
/// ```
pub struct Server
{
    addr: SocketAddr,
    router: Router,
    config: Config
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
        tag: bool
    ) 
    -> Server
    {

        let config = match read_config()
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

        let mut sitemap = SiteMap::new(config.domain.clone(), config.content.path.clone());

        match config.content.ignore_regexes.clone()
        {
            Some(p) => 
            {
                sitemap.build
                (
                    config.content.browser_cache_period_seconds,
                    config.content.server_cache_period_seconds,
                    tag, 
                    config.content.allow_without_extension, 
                    Some(&ContentFilter::new(p))
                );
            },
            None => 
            {
                sitemap.build
                (
                    config.content.browser_cache_period_seconds,
                    config.content.server_cache_period_seconds,
                    tag, 
                    config.content.allow_without_extension, 
                    None
                );
            }
        };

        let mut home = Content::new("/", &config.content.home.clone(), config.content.server_cache_period_seconds, config.content.browser_cache_period_seconds, tag);
        match home.load_from_file()
        {
            Ok(()) =>
            {
                sitemap.push(home);
            },
            Err(e) => {crate::debug(format!("Error serving home page resource {}", e), None);}
        }
        
        let mut router: Router<(), axum::body::Body> = sitemap.into();

        let stats = Arc::new(Mutex::new(
            Stats 
            {
                hits: HashMap::new(), 
                last_save: chrono::offset::Utc::now(),
                last_digest: chrono::offset::Utc::now(),
                last_clear: chrono::offset::Utc::now(),
                summary: Digest::new()
            }
        ));

        let _stats_thread = spawn(Stats::stats_thread(stats.clone()));

        router = router.layer(middleware::from_fn_with_state(stats.clone(), log_stats));
        router = router.layer(middleware::from_fn_with_state(throttle_state.clone(), handle_throttle));

        router = router.layer(middleware::from_fn_with_state(Some(stats), StatsDigest::filter));

        Server
        {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a,b,c,d)), config.port_https),
            router,
            config
        }
    }

    pub fn get_addr(self: Server) -> SocketAddr
    {
        self.addr
    }

    pub async fn serve(self: Server)
    {

        // configure https

        let cert_path = self.config.cert_path;
        let key_path = self.config.key_path;

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

        let domain = if self.config.domain.contains("https://")
        {
            self.config.domain.clone()
        }
        else
        {
            format!("https://{}", self.config.domain)
        };

        println!("Checkout your cool site, at {} {}!", domain, String::from_utf8(CRAB.to_vec()).unwrap());
        if domain != "https://127.0.0.1"
        {
            println!("(or https://127.0.0.1)");
        }
        
        axum_server::bind_rustls(self.addr, config)
        .serve(self.router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    }

}