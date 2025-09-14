use super::client::*;
use super::options::*;
use crate::prelude::*;

pub struct ScrapeCommand {
    options: ScrapeOptions,
    client: Client,
}

impl ScrapeCommand {
    pub fn new() -> Result<Self, ScrapeError> {
        let options = ScrapeOptions::get()?;
        options.validate()?;
        let client = Client {
            cache_dir: options.cache_dir.clone(),
        };
        Ok(Self { options, client })
    }

    pub fn execute(self) -> Result<(), ScrapeError> {
        let _html = self.client.get_html(&self.options.url)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    Config(ConfigError),
    Validation(Vec<ValidationError>),
    Client(ClientError),
}

impl From<ConfigError> for ScrapeError {
    fn from(err: ConfigError) -> Self {
        ScrapeError::Config(err)
    }
}

impl From<ClientError> for ScrapeError {
    fn from(err: ClientError) -> Self {
        ScrapeError::Client(err)
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
            ScrapeError::Config(e) => format!("Unable to read config: {e}"),
            ScrapeError::Client(e) => format!("Unable to fetch HTML: {e}"),
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
        let command = ScrapeCommand::new()?;

        // Act
        command.execute()?;

        // Assert
        Ok(())
    }
}
