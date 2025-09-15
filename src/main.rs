use pura::prelude::*;

fn main() {
    let _ = init_logging();
    let result = ScrapeCommand::new().and_then(ScrapeCommand::execute);
    match result {
        Ok(()) => {}
        Err(e) => {
            error!("{e}");
        }
    }
}
