use crate::
{
    web::throttle::{IpThrottler, handle_throttle},
    util::read_file_utf8
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

use super::model::{AppState, CONFIG_PATH, Config};

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
            10.0, 
            5000
        );

        let throttle_state = Arc::new(Mutex::new(requests));

        let app = Arc::new(Mutex::new(AppState::new()));
        
        let self_uri = format!("https://{}.{}.{}.{}",a,b,c,d);

        ServerHttp
        {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a,b,c,d)), config.get_port_http()),
            router: Router::new()
            .route("/", get(|| async move 
            {
                    crate::debug(format!("http redirect"), None);
                    Redirect::permanent(&self_uri)
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