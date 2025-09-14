use crate::prelude::*;
use super::client::*;
use super::options::*;

pub struct ScrapeCommand;

impl ScrapeCommand {
    pub fn execute(self) -> Result<(), ScrapeError> {
        let options = ScrapeOptions::get()?;
        options.validate()?;
        let client = Client {
            cache_dir: options.cache_dir,
        };
        let _html = client.get(&options.url)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    Config(ConfigError),
    Validation(Vec<ValidationError>),
    HtmlProvider(ClientError),
}

impl From<ConfigError> for ScrapeError {
    fn from(err: ConfigError) -> Self {
        ScrapeError::Config(err)
    }
}

impl From<ClientError> for ScrapeError {
    fn from(err: ClientError) -> Self {
        ScrapeError::HtmlProvider(err)
    }
}

impl From<Vec<ValidationError>> for ScrapeError {
    fn from(errors: Vec<ValidationError>) -> Self {
        ScrapeError::Validation(errors)
    }
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
