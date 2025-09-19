use pura::prelude::*;
use std::env::args;
use std::process::exit;

#[tokio::main]
async fn main() {
    let _ = init_logging();
    let services = match ServiceProvider::create().await {
        Ok(services) => services,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };
    let args: Vec<_> = args().collect();
    let command = args.get(1).cloned().unwrap_or_default();
    match command.as_str() {
        "scrape" => {
            scrape(services, args).await;
        }
        "download" => {
            download(services, args).await;
        }
        arg => {
            error!("Unknown command: `{arg}`");
            exit(1);
        }
    }
}

async fn download(services: ServiceProvider, args: Vec<String>) {
    let command = DownloadCommand::new(services.paths, services.http, services.podcasts);
    let Some(id) = args.get(2) else {
        error!("Missing id");
        exit(1);
    };
    if let Err(e) = command.execute(id).await {
        error!("{e}");
        exit(1);
    }
}

async fn scrape(services: ServiceProvider, args: Vec<String>) {
    let command = ScrapeCommand::new(services.http, services.podcasts);
    let Some(id) = args.get(2) else {
        error!("Missing id");
        exit(1);
    };
    let Some(url) = args.get(3) else {
        error!("Missing url");
        exit(1);
    };
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(e) => {
            error!("Invalid URL: {url}\n{e}");
            exit(1);
        }
    };
    if let Err(e) = command.execute(id, &url).await {
        error!("{e}");
        exit(1);
    }
}
