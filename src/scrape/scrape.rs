use crate::prelude::*;
use rss::Channel;

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
        let content_type = self.http.head(url).await.map_err(ScrapeError::Head)?;
        let podcast = match content_type.as_str() {
            "application/xml" => self
                .execute_rss(podcast_id, url)
                .await
                .map_err(ScrapeError::Rss)?,
            _ => self
                .execute_simplecast(podcast_id, url)
                .await
                .map_err(ScrapeError::Simplecast)?,
        };
        info!("{} {} episodes", "Fetched".bold(), podcast.episodes.len());
        self.podcasts.put(&podcast).map_err(ScrapeError::Save)?;
        Ok(podcast)
    }

    pub(super) async fn execute_rss(
        &self,
        podcast_id: &str,
        url: &Url,
    ) -> Result<Podcast, ScrapeRssError> {
        let path = self
            .http
            .get(url, Some(XML_EXTENSION))
            .await
            .map_err(ScrapeRssError::Xml)?;
        let file = File::open(&path)
            .map_err(|e| ScrapeRssError::IO(podcast_id.to_owned(), path.clone(), e))?;
        let reader = BufReader::new(file);
        let channel = Channel::read_from(reader).map_err(ScrapeRssError::Parse)?;
        let mut podcast: Podcast = channel.try_into().map_err(ScrapeRssError::Convert)?;
        podcast.id = podcast_id.to_owned();
        Ok(podcast)
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    Head(HttpError),
    Simplecast(ScrapeSimplecastError),
    Rss(ScrapeRssError),
    Save(DatabaseError),
}

#[derive(Debug)]
#[allow(clippy::absolute_paths)]
pub enum ScrapeRssError {
    Xml(HttpError),
    IO(String, PathBuf, std::io::Error),
    Parse(rss::Error),
    Convert(PodcastConvertError),
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            ScrapeError::Head(e) => format!("Unable to get content type:\n{e}"),
            ScrapeError::Simplecast(e) => format!("{e}"),
            ScrapeError::Rss(e) => {
                format!("{e}",)
            }
            ScrapeError::Save(e) => format!("Unable to save: {e}"),
        };
        write!(f, "{} to scrape\n{reason}", "Failed".bold())
    }
}

impl Display for ScrapeRssError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            ScrapeRssError::Xml(e) => format!("Unable to get feed:\n{e}"),
            ScrapeRssError::IO(id, path, e) => {
                format!(
                    "An I/O error occurred while processing episode: {id}\nPath: {}\n{e}",
                    path.display()
                )
            }
            ScrapeRssError::Parse(e) => {
                format!("Unable to parse RSS\n{e}",)
            }
            ScrapeRssError::Convert(e) => {
                format!("Unable to convert RSS\n{e}",)
            }
        };
        write!(f, "{} to scrape\n{reason}", "Failed".bold())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn scrape_command_simplecast() {
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

    #[tokio::test]
    pub async fn scrape_command_rss() {
        // Arrange
        let _ = init_logging();
        let services = ServiceProvider::create()
            .await
            .expect("ServiceProvider should not fail");
        let command = ScrapeCommand::new(services.http, services.podcasts);
        let url = Url::parse("https://feeds.simplecast.com/lP7owBq8").expect("URL should parse");

        // Act
        let result = command.execute("irl-rss", &url).await;

        // Assert
        let podcast = result.assert_ok();
        assert!(podcast.episodes.len() > 30);
    }
}
