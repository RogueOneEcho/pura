use crate::scrape::*;
use rogue_config::ConfigError;

pub struct ScrapeCommand;

impl ScrapeCommand {
    pub async fn execute(self) -> Result<(), ConfigError> {
        let _options = ScrapeOptions::get()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logging;

    #[tokio::test]
    pub async fn scrape_command() -> Result<(), ConfigError> {
        // Arrange
        let _ = init_logging();
        let command = ScrapeCommand;

        // Act
        command.execute().await?;

        // Assert
        Ok(())
    }
}
