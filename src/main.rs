use pura::prelude::*;
use std::process::exit;

#[tokio::main]
async fn main() {
    let _ = init_logging();
    let cli = Cli::parse();
    let services = match ServiceProvider::create().await {
        Ok(services) => services,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };
    match cli.command {
        Command::Scrape(options) => {
            let command = ScrapeCommand::new(services.http, services.podcasts);
            if let Err(e) = command.execute(options).await {
                error!("{e}");
                exit(1);
            }
        }
        Command::Download(options) => {
            let command = DownloadCommand::new(services.paths, services.http, services.podcasts);
            if let Err(e) = command.execute(options).await {
                error!("{e}");
                exit(1);
            }
        }
        Command::CreateFeeds(options) => {
            let command = FeedsCommand::new(services.podcasts, services.paths);
            if let Err(e) = command.execute(options).await {
                error!("{e}");
                exit(1);
            }
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Scrape a podcast from an RSS feed or website.
    Scrape(ScrapeOptions),
    /// Download episodes of a scraped podcast.
    Download(DownloadOptions),
    /// Create RSS feeds for a scraped podcast.
    CreateFeeds(FeedsOptions),
}
