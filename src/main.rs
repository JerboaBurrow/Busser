use std::time::Duration;

use busser::config::{read_config, Config, CONFIG_PATH};
use busser::content::sitemap::SiteMap;
use busser::integrations::discord::post::try_post;
use busser::integrations::git::clean_and_clone;
use busser::server::http::ServerHttp;
use busser::server::https::Server;
use busser::util::formatted_differences;
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
    
    let http_server = ServerHttp::new(0,0,0,0);
    let _http_redirect = spawn(http_server.serve());

    match read_config(CONFIG_PATH)
    {
        Some(c) => 
        {
            if c.git.is_some()
            {
                match clean_and_clone(&c.content.path, c.git.unwrap())
                {
                    Ok(_) => (),
                    Err(e) =>
                    {
                        busser::debug(format!("{}", e), None);
                        std::process::exit(1);
                    }
                }
            }
        },
        None =>
        {
            println!("No config found at {}", CONFIG_PATH);
            std::process::exit(1);
        }
    };

    if args.iter().any(|x| x == "--static-sitemap")
    {
        busser::debug(format!("Serving with static sitemap"), None);
        serve(insert_tag).await;
    }
    else
    {
        busser::debug(format!("Serving with dynamic sitemap"), None);
        serve_observed(insert_tag).await;
    }
}

/// Serve by observing the site content found at the path [busser::config::ContentConfig]
///  every [busser::config::ContentConfig::server_cache_period_seconds] the sitemap
///  hash (see [busser::content::sitemap::SiteMap::get_hash]) is checked, if it is 
///  different the server is re-served. 
/// 
///  On a re-serve if [busser::config::ContentConfig::message_on_sitemap_reload] is true
///   A status message with (uri) additions and removals will be posted to Discord.
async fn serve_observed(insert_tag: bool)
{
    let mut sitemap = SiteMap::build(&Config::load_or_default(CONFIG_PATH), insert_tag, false);
    let mut hash = sitemap.get_hash();

    let server = Server::new(0,0,0,0,sitemap.clone());
    let mut server_handle = server.get_handle();
    let mut thread_handle = spawn(async move {server.serve()}.await);
    
    loop
    {
        let config = Config::load_or_default(CONFIG_PATH);
        busser::debug(format!("Next sitemap check: {}s", config.content.server_cache_period_seconds), None);
        tokio::time::sleep(Duration::from_secs(config.content.server_cache_period_seconds.into())).await;
        
        let new_sitemap = SiteMap::build(&config, insert_tag, false);
        let sitemap_hash = new_sitemap.get_hash();

        if sitemap_hash != hash
        {
            busser::debug(format!("Sitemap changed, shutting down"), None);
            server_handle.shutdown();
            thread_handle.abort();

            let diffs = formatted_differences(new_sitemap.collect_uris(), sitemap.collect_uris());
            sitemap = new_sitemap.clone();

            let server = Server::new(0,0,0,0,new_sitemap);
            server_handle = server.get_handle();
            thread_handle = spawn(async move {server.serve()}.await);
            hash = sitemap_hash;
            busser::debug(format!("Re-served\n Diffs:\n{}", diffs), None);
            if config.content.message_on_sitemap_reload.is_some_and(|x|x)
            {
                try_post(config.notification_endpoint, &format!("The sitemap was refreshed with diffs:\n```{}```", diffs)).await;
            }
        }
    }
}

/// Serve without checking for sitemap changes
async fn serve(insert_tag: bool)
{
    let sitemap = SiteMap::build(&Config::load_or_default(CONFIG_PATH), insert_tag, false);
    let server = Server::new(0,0,0,0,sitemap);
    server.serve().await;
}