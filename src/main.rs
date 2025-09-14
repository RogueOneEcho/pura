use log::{error, info};
use pura::*;

fn main() {
    let _ = init_logging();
    let command = ScrapeCommand;

    // Act
    match command.execute() {
        Ok(()) => {
            info!("Success");
        }
        Err(e) => {
            error!("{e}");
        }
    }
}
