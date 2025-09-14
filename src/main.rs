use pura::*;

#[tokio::main]
async fn main() {
    let _ = init_logging();
    let command = ScrapeCommand;

    // Act
    match command.execute().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{e}");
        }
    }
}
