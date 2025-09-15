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
        let html = self.client.get_html(&self.options.url)?;
        let iframe_src = parse_iframe_src(&html)?;
        let player_html = self.client.get_html(&iframe_src)?;
        let audio_src = parse_audio_src(&player_html)?;
        let file = self.client.get(&audio_src)?;
        let podcast = Podcast {
            title: String::new(),
            description: String::new(),
            date: String::new(),
            episode_type: String::new(),
            author: String::new(),
            subtitle: String::new(),
            summary: String::new(),
            duration: 0,
            guid: String::new(),
            file,
        };
        info!("{podcast}");
        Ok(())
    }
}

fn parse_iframe_src(html: &Html) -> Result<Url, ScrapeError> {
    let src = get_element_attr(html, "div.audio-player iframe", "src")?;
    parse_url(&src)
}

fn parse_audio_src(html: &Html) -> Result<Url, ScrapeError> {
    let src = get_element_attr(html, "audio", "src")?;
    parse_url(&src)
}

fn get_element_attr(html: &Html, selector: &str, attr: &str) -> Result<String, ScrapeError> {
    let s = Selector::parse(selector).expect("Selector should be valid");
    let element = html
        .select(&s)
        .nth(0)
        .ok_or_else(|| ScrapeError::ElementNotFound(selector.to_owned()))?;
    element
        .attr(attr)
        .map(str::to_owned)
        .ok_or_else(|| ScrapeError::AttributeNotFound(selector.to_owned(), attr.to_owned()))
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
