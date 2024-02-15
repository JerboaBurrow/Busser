use crate::
{
    config::read_config, util::read_file_utf8, web::throttle::{handle_throttle, IpThrottler}
};

use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, path::Path};
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::
{
    routing::{post, get}, 
    Router, 
    response::Redirect,
    middleware
};

pub struct ServerHttp
{
    addr: SocketAddr,
    router: Router
}

impl ServerHttp
{
    pub fn new 
    (
        a: u8,
        b: u8,
        c: u8,
        d: u8
    ) 
    -> ServerHttp
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

        let mut domain = config.get_domain();

        domain = domain.replacen("http://", "https://", 1);

        if !domain.starts_with("https://")
        {
            domain = "https://".to_string()+&domain
        }
        
        ServerHttp
        {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a,b,c,d)), config.get_port_http()),
            router: Router::new()
            .route("/", get(|| async move 
            {
                    crate::debug(format!("http redirect to {}", domain), None);
                    Redirect::permanent(&domain)
            }))
            .layer(middleware::from_fn_with_state(throttle_state.clone(), handle_throttle))

        }
    }

    pub fn get_addr(self: ServerHttp) -> SocketAddr
    {
        self.addr
    }

    pub async fn serve(self: ServerHttp)
    {
        axum::Server::bind(&self.addr)
        .serve(self.router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    }

}