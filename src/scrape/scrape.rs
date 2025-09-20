use crate::prelude::*;

pub struct ScrapeCommand {
    pub(super) http: HttpClient,
    pub(super) podcasts: PodcastProvider,
}

impl ScrapeCommand {
    #[must_use]
    pub fn new(http: HttpClient, podcasts: PodcastProvider) -> Self {
        Self { http, podcasts }
    }

    pub async fn execute(&self, podcast_id: &str, url: &Url) -> Result<Podcast, ScrapeError> {
        let podcast = self
            .execute_simplecast(podcast_id, url)
            .await
            .map_err(ScrapeError::Simplecast)?;
        info!("{} {} episodes", "Fetched".bold(), podcast.episodes.len());
        self.podcasts.put(&podcast).map_err(ScrapeError::Save)?;
        Ok(podcast)
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    Simplecast(ScrapeSimplecastError),
    Save(DatabaseError),
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            ScrapeError::Simplecast(e) => format!("{e}"),
            ScrapeError::Save(e) => format!("Unable to save: {e}"),
        };
        write!(f, "{} to scrape\n{reason}", "Failed".bold())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn scrape_command() {
        // Arrange
        let _ = init_logging();
        let services = ServiceProvider::create()
            .await
            .expect("ServiceProvider should not fail");
        let command = ScrapeCommand::new(services.http, services.podcasts);
        let url = Url::parse("https://irlpodcast.org").expect("URL should parse");

        // Act
        let result = command.execute("irl", &url).await;

        // Assert
        let podcast = result.assert_ok();
        assert!(podcast.episodes.len() > 30);
    }
}
