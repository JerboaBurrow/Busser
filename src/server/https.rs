use crate::
{
    pages::get_pages, util::read_file_utf8, web::throttle::{handle_throttle, IpThrottler}
};

use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, path::Path};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::
{
    middleware, response::{IntoResponse, Redirect}, routing::{get, post}, Router
};
use axum_server::tls_rustls::RustlsConfig;

use super::model::{AppState, Config, CONFIG_PATH};

pub struct Server
{
    addr: SocketAddr,
    router: Router,
    config: Config
}

impl Server 
{
    pub fn new 
    (
        a: u8,
        b: u8,
        c: u8,
        d: u8,
    ) 
    -> Server
    {

        let config = if Path::new(CONFIG_PATH).exists()
        {
            let data = match read_file_utf8(CONFIG_PATH)
            {
                Some(d) => d,
                None =>
                {
                    println!("Error reading configuration file {} no data", CONFIG_PATH);
                    std::process::exit(1);
                }
            };

            let config: Config = match serde_json::from_str(&data)
            {
                Ok(data) => {data},
                Err(why) => 
                {
                    println!("Error reading configuration file {}\n{}", CONFIG_PATH, why);
                    std::process::exit(1);
                }
            };

            config
        }
        else 
        {
            println!("Error configuration file {} does not exist", CONFIG_PATH);
            std::process::exit(1);
        };

        let requests: IpThrottler = IpThrottler::new
        (
            config.get_throttle_config().get_max_requests_per_second(), 
            config.get_throttle_config().get_timeout_millis(),
            config.get_throttle_config().get_clear_period_seconds()
        );

        let throttle_state = Arc::new(Mutex::new(requests));

        let app = Arc::new(Mutex::new(AppState::new()));

        let pages = get_pages(Some(&config.get_path()));

        let mut router: Router<(), axum::body::Body> = Router::new();

        for page in pages
        {
            crate::debug(format!("Adding page {:?}", page), None);

            let path = config.get_path()+"/";

            let uri = if page.get_uri().starts_with(&path)
            {
                page.get_uri().replace(&path, "/")
            }
            else if page.get_uri().starts_with("/")
            {
                page.get_uri()
            }
            else
            {
                "/".to_string()+&page.get_uri()
            };

            crate::debug(format!("Serving: {}", uri), None);
            
            router = router.route
            (
                &uri, 
                get(|| async move {page.clone().into_response()})
            )
        }

        router = router.layer(middleware::from_fn_with_state(throttle_state.clone(), handle_throttle));

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