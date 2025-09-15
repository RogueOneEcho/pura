use pura::prelude::*;

fn main() {
    let _ = init_logging();
    let result = ScrapeCommand::new().and_then(ScrapeCommand::execute);
    match result {
        Ok(()) => {
            info!("Success");
        }
        Err(e) => {
            error!("{e}");
        }
    }
}
