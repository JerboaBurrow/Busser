use busser::server::http::ServerHttp;
use busser::server::https::Server;
use busser::program_version;

use tokio::task::spawn;

#[tokio::main]
async fn main() {

    let args: Vec<String> = std::env::args().collect();
 
    if args.iter().any(|x| x == "-v")
    {
        println!("Version: {}", program_version());
        std::process::exit(0);
    }

    let insert_tag = if args.iter().any(|x| x == "--no-tagging")
    {
        false
    }
    else
    {
        true
    };
    
    let server = Server::new(0,0,0,0, insert_tag);

    let http_server = ServerHttp::new(0,0,0,0);

    let _http_redirect = spawn(http_server.serve());

    server.serve().await;

}