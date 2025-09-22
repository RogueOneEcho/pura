use crate::prelude::*;
#[cfg(test)]
use chrono::Utc;
use rss::extension::itunes::ITunesChannelExtension;
use std::error::Error;
use std::num::ParseIntError;
use strum_macros::AsRefStr;

/// Podcast or Channel
///
/// <https://help.apple.com/itc/podcasts_connect/#/itcb54353390>
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Podcast {
    /// Local file system ID
    pub id: String,
    /// GUID
    pub guid: String,
    /// Title
    pub title: String,
    /// HTML formatted description
    pub description: String,
    /// URL of JPEG or PNG artwork
    /// - Min: 1400 x 1400 px
    /// - Max: 3000 x 3000 px
    pub image_url: Option<Url>,
    /// ISO 639-2 code for language
    ///
    /// <https://www.loc.gov/standards/iso639-2/php/code_list.php>
    pub language: String,
    /// Category
    ///
    /// <https://podcasters.apple.com/support/1691-apple-podcasts-categories>
    pub category: Option<String>,
    /// Sub-category
    ///
    /// <https://podcasters.apple.com/support/1691-apple-podcasts-categories>
    pub sub_category: Option<String>,
    /// Parental advisory information
    pub explicit: bool,
    /// Group responsible for creating the show
    pub author: Option<String>,
    /// Website
    pub link: Url,
    /// Episodic or Serial
    pub podcast_type: PodcastType,
    /// Copyright details
    pub copyright: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub episodes: Vec<Episode>,
}

impl Podcast {
    pub(crate) fn validate_id(id: &str) -> Result<String, String> {
        if id.is_empty() {
            return Err("Value must not be empty".to_owned());
        }
        if id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            Ok(id.to_owned())
        } else {
            Err("Podcast ID must contain only lowercase letters and hyphens".to_owned())
        }
    }
}

/// Episodic or Serial
#[derive(AsRefStr, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum PodcastType {
    /// Specify episodic when episodes are intended to be consumed without any specific order.
    /// Apple Podcasts will present newest episodes first and display the publish date (required)
    /// of each episode. If organized into seasons, the newest season will be presented first -
    /// otherwise, episodes will be grouped by year published, newest first.
    #[default]
    Episodic,
    /// Specify serial when episodes are intended to be consumed in sequential order. Apple
    /// Podcasts will present the oldest episodes first and display the episode numbers (required)
    /// of each episode. If organized into seasons, the newest season will be presented first and
    /// <itunes:episode> numbers must be given for each episode.
    Serial,
}

impl Podcast {
    #[cfg(test)]
    pub(crate) fn example() -> Self {
        Self {
            id: "test".to_owned(),
            guid: "29e09be7-ee09-4671-9130-0da5b958e9a2".to_owned(),
            title: "Podcast Title".to_owned(),
            description: "Sed ac volutpat tortor. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Suspendisse placerat leo augue, id elementum orci venenatis eu.".to_owned(),
            image_url: None,
            language: "en-us".to_owned(),
            category: None,
            sub_category: None,
            explicit: false,
            author: None,
            link: Url::parse("https://example.com/").expect("URL should be valid"),
            podcast_type: PodcastType::default(),
            copyright: None,
            created_at: Some(Utc::now().naive_utc()),
            episodes: vec![Episode::example()],
        }
    }
}

impl From<&Podcast> for RssChannel {
    fn from(podcast: &Podcast) -> Self {
        Self {
            title: podcast.title.clone(),
            link: podcast.link.to_string(),
            description: podcast.description.clone(),
            language: Some(podcast.language.clone()),
            copyright: podcast.copyright.clone(),
            itunes_ext: Some(podcast.into()),
            items: podcast.episodes.iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<&Podcast> for ITunesChannelExtension {
    fn from(podcast: &Podcast) -> Self {
        Self {
            author: podcast.author.clone(),
            block: None,
            categories: Vec::new(),
            image: podcast.image_url.as_ref().map(ToString::to_string),
            explicit: Some(podcast.explicit.to_string()),
            complete: None,
            new_feed_url: None,
            owner: None,
            subtitle: None,
            summary: Some(podcast.description.clone()),
            keywords: None,
            r#type: Some(podcast.podcast_type.as_ref().to_lowercase()),
        }
    }
}

impl From<String> for PodcastType {
    fn from(value: String) -> Self {
        if value == "serial" {
            PodcastType::Serial
        } else {
            PodcastType::Episodic
        }
    }
}

impl TryFrom<RssChannel> for Podcast {
    type Error = PodcastConvertError;
    fn try_from(channel: RssChannel) -> Result<Self, Self::Error> {
        let itunes = channel
            .itunes_ext
            .ok_or(PodcastConvertError::Required("itunes".to_owned()))?;
        Ok(Podcast {
            id: String::new(),
            guid: String::new(),
            title: channel.title,
            description: channel.description,
            image_url: itunes.image.and_then(|u| Url::parse(&u).ok()),
            language: channel.language.unwrap_or_default(),
            category: itunes.categories.first().map(|c| c.text.clone()),
            sub_category: itunes.categories.get(1).map(|c| c.text.clone()),
            explicit: itunes.explicit.unwrap_or_default() == "true",
            author: itunes.author,
            link: Url::parse(&channel.link)
                .map_err(|e| PodcastConvertError::Url("link".to_owned(), e))?,
            podcast_type: itunes.r#type.unwrap_or_default().into(),
            copyright: channel.copyright,
            created_at: None,
            episodes: channel
                .items
                .into_iter()
                .map(Episode::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug)]
pub enum PodcastConvertError {
    Required(String),
    Url(String, url::ParseError),
    Date(String, chrono::ParseError),
    Integer(String, ParseIntError),
}

impl Display for PodcastConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{self:?}")
    }
}

impl Error for PodcastConvertError {}

#[cfg(test)]
mod tests {
    use super::*;
    use rss::validation::Validate;

    #[test]
    fn validate_id() {
        assert!(Podcast::validate_id("test").is_ok());
        assert!(Podcast::validate_id("te-st").is_ok());
        assert!(Podcast::validate_id("123").is_ok());
        assert!(Podcast::validate_id("test-123").is_ok());
        assert!(Podcast::validate_id("test_123").is_err());
        assert!(Podcast::validate_id("").is_err());
    }

    #[test]
    fn to_rss() {
        // Arrange
        let podcast = &Podcast::example();

        // Act
        let channel: RssChannel = podcast.into();

        // Assert
        let result = channel.validate();
        result.assert_ok();
    }
}
