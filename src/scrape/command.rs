use super::client::*;
use super::options::*;
use crate::prelude::*;
use crate::scrape::simplecast_playlist::Playlist;

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
        let html = self.client.get_html(&self.options.url)?;
        let episode_guid = get_simplecast_episode_guid(&html)?;
        debug!(
            "{} Simplecast player with episode id: {episode_guid}",
            "Found".bold()
        );
        let episode_url = Url::parse(&format!(
            "https://api.simplecast.com/episodes/{episode_guid}"
        ))
        .expect("URL should be valid");
        let episode: Episode = self.client.get_json(&episode_url)?;
        debug!(
            "{} playlist for {}",
            "Fetching".bold(),
            episode.podcast.title
        );
        let mut playlist_url = Url::parse(&format!(
            "https://api.simplecast.com/podcasts/{}/playlist",
            episode.podcast.id
        ))
        .expect("URL should be valid");
        let mut episodes = Vec::new();
        loop {
            let mut playlist: Playlist = self.client.get_json(&playlist_url)?;
            let next = playlist.episodes.pages.next.clone();
            episodes.append(&mut playlist.episodes.collection);
            let Some(link) = next else {
                break;
            };
            playlist_url = link.href;
        }
        info!(
            "{} {} episodes of {}",
            "Found".bold(),
            episodes.len(),
            episode.podcast.title
        );
        Ok(())
    }
}

fn get_simplecast_episode_guid(html: &Html) -> Result<String, ScrapeError> {
    get_element_attr(html, "div.audio-player iframe", "src")
        .into_iter()
        .find_map(|url| {
            let host = url.host_str()?;
            if host != "player.simplecast.com" {
                return None;
            }
            let guid = url.path_segments()?.next()?.to_owned();
            Some(guid)
        })
        .ok_or_else(|| ScrapeError::SimplecastNotFound)
}

fn get_element_attr(html: &Html, selector: &str, attr: &str) -> Vec<Url> {
    html.select(&Selector::parse(selector).expect("Selector should be valid"))
        .filter_map(|element| element.attr(attr).and_then(|url| parse_url(url).ok()))
        .collect()
}

fn parse_url(src: &str) -> Result<Url, ScrapeError> {
    Url::parse(src).map_err(|e| {
        error!("{e}");
        ScrapeError::InvalidUrl(src.to_owned())
    })
}

#[derive(Debug)]
pub enum ScrapeError {
    Config(ConfigError),
    Validation(Vec<ValidationError>),
    Client(ClientError),
    ElementNotFound(String),
    AttributeNotFound(String, String),
    InvalidUrl(String),
    SimplecastNotFound,
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
            ScrapeError::ElementNotFound(element) => format!("Unable to find element: {element}"),
            ScrapeError::AttributeNotFound(element, attr) => {
                format!("Unable to find attribute `{attr}` on element: {element}")
            }
            ScrapeError::InvalidUrl(url) => format!("Invalid Url: {url}"),
            ScrapeError::SimplecastNotFound => "Simplecast not found".to_owned(),
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
