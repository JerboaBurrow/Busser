use std::time::Duration;

use busser::config::{read_config, Config, CONFIG_PATH};
use busser::content::filter::ContentFilter;
use busser::content::sitemap::{self, SiteMap};
use busser::content::Content;
use busser::server::http::ServerHttp;
use busser::server::https::Server;
use busser::task::{next_job_time, schedule_from_option, TaskPool};
use busser::{openssl_version, program_version};

use tokio::task::spawn;

#[tokio::main]
async fn main() {

    let args: Vec<String> = std::env::args().collect();
 
    if args.iter().any(|x| x == "-v")
    {
        println!("Version: {}\n{}", program_version(), openssl_version());
        std::process::exit(0);
    }

    if args.iter().any(|x| x == "-d")
    {
        unsafe { busser::OPTIONS.debug = true; }
    }

    if args.iter().any(|x| x == "-t")
    {
        unsafe { busser::OPTIONS.debug_timestamp = true; }
    }

    let insert_tag = if args.iter().any(|x| x == "--no-tagging")
    {
        false
    }
    else
    {
        true
    };
    
    let config = match read_config(CONFIG_PATH)
    {
        Some(c) => c,
        None =>
        {
            std::process::exit(1)
        }
    };
    
    let http_server = ServerHttp::new(0,0,0,0);
    let _http_redirect = spawn(http_server.serve());

    serve(insert_tag).await;

}

async fn serve(insert_tag: bool)
{
    let mut hash: Vec<u8> = vec![];
    let mut server = Server::new(0,0,0,0,sitemap);
    loop
    {
        let config = Config::load_or_default(CONFIG_PATH);
        let sitemap = SiteMap::from_config(&config, insert_tag);
        let sitemap_hash = sitemap.get_hash();

        if sitemap_hash != hash
        {
            spawn(server.serve());
        }

        let schedule = schedule_from_option(config.stats.save_schedule.clone());
        let next_run = match schedule
        {
            Some(s) => next_job_time(s.clone()),
            None => None
        };
        let last_run = chrono::offset::Utc::now();
    }
}