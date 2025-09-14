use crate::scrape::*;
use colored::Colorize;
use rogue_config::ConfigError;
use std::fmt::{Display, Formatter};

pub struct ScrapeCommand;

impl ScrapeCommand {
    pub fn execute(self) -> Result<(), ScrapeError> {
        let options = ScrapeOptions::get()?;
        options.validate()?;
        let html_provider = HtmlProvider {
            cache_dir: options.cache_dir,
        };
        let _html = html_provider.get(&options.url)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    Config(ConfigError),
    Validation(Vec<ValidationError>),
    HtmlProvider(HtmlProviderError),
}

impl From<ConfigError> for ScrapeError {
    fn from(err: ConfigError) -> Self {
        ScrapeError::Config(err)
    }
}

impl From<HtmlProviderError> for ScrapeError {
    fn from(err: HtmlProviderError) -> Self {
        ScrapeError::HtmlProvider(err)
    }
}

impl From<Vec<ValidationError>> for ScrapeError {
    fn from(errors: Vec<ValidationError>) -> Self {
        ScrapeError::Validation(errors)
    }
}

impl Display for ScrapeError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            ScrapeError::Config(err) => format!("Unable to read config: {err}"),
            ScrapeError::HtmlProvider(err) => format!("Unable to fetch HTML: {err:?}"),
            ScrapeError::Validation(errors) => errors.iter().fold(String::new(), |mut acc, err| {
                acc.push('\n');
                acc.push_str(&err.to_string());
                acc
            }),
        };
        write!(f, "{} to scrape\n{reason}", "Failed".bold())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logging;

    #[test]
    pub fn scrape_command() -> Result<(), ScrapeError> {
        // Arrange
        let _ = init_logging();
        let command = ScrapeCommand;

        // Act
        command.execute()?;

        // Assert
        Ok(())
    }
}
