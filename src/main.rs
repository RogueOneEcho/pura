use pura::prelude::*;


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
