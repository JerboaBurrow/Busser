use crate::
{
    config::{read_config, Config}, 
    pages::{get_pages, page::Page}, 
    resources::get_resources, 
    web::{stats::{log_stats, Digest, Stats}, 
    throttle::{handle_throttle, IpThrottler}},
    server::api::stats
};

use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr}};
use std::path::PathBuf;
use std::sync::Arc;
use regex::Regex;
use tokio::{spawn, sync::Mutex};

use axum::
{
    middleware, response::IntoResponse, routing::get, 
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
            config.get_throttle_config().get_max_requests_per_second(), 
            config.get_throttle_config().get_timeout_millis(),
            config.get_throttle_config().get_clear_period_seconds()
        );

        let throttle_state = Arc::new(Mutex::new(requests));

        let pages = get_pages(Some(&config.get_path()), Some(config.get_cache_period_seconds()));
        let resources = get_resources(Some(&config.get_path()), Some(config.get_cache_period_seconds()));

        let mut router: Router<(), axum::body::Body> = Router::new();

        for mut page in pages
        {
            crate::debug(format!("Adding page {:?}", page.preview(64)), None);

            let path = config.get_path()+"/";

            let uri = parse_uri(page.get_uri(), path);

            crate::debug(format!("Serving: {}", uri), None);

            if tag { page.insert_tag(); }

            if config.get_allow_without_extension()
            {
                let extension_regex = Regex::new(r"\.\S+$").unwrap();
                let short_uri = extension_regex.replacen(&uri, 1, "");

                crate::debug(format!("Serving as short url: {}",short_uri), None);

                let page_short = page.clone();

                router = router.route
                (
                    &short_uri, 
                    get(|| async move {page_short.clone().into_response()})
                );
            }

            router = router.route
            (
                &uri, 
                get(|| async move {page.into_response()})
            );
        }

        for resource in resources
        {
            crate::debug(format!("Adding resource {:?}", resource.preview(8)), None);

            let path = config.get_path()+"/";

            let uri = parse_uri(resource.get_uri(), path);

            crate::debug(format!("Serving: {}", uri), None);
            
            router = router.route
            (
                &uri, 
                get(|| async move {resource.clone().into_response()})
            )
        }

        match Page::from_file(config.get_home(), config.get_cache_period_seconds())
        {
            Some(mut page) => 
            { 
                if tag { page.insert_tag(); }
                crate::debug(format!("Serving home page, /, {}", page.preview(64)), None);
                router = router.route("/", get(|| async move {page.clone().into_response()}))
            },
            None => {}
        }

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
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a,b,c,d)), config.get_port_https()),
            router: router,
            config: config
        }
    }

    pub fn get_addr(self: Server) -> SocketAddr
    {
        self.addr
    }

    pub async fn serve(self: Server)
    {

        // configure https

        let cert_path = self.config.get_cert_path();
        let key_path = self.config.get_key_path();

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

        axum_server::bind_rustls(self.addr, config)
        .serve(self.router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    }

}