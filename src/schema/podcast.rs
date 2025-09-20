use crate::prelude::*;
#[cfg(test)]
use chrono::Utc;
use rss::extension::itunes::ITunesChannelExtension;
use rss::Channel;
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
    pub created_at: NaiveDateTime,
    pub episodes: Vec<Episode>,
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
            created_at: Utc::now().naive_utc(),
            episodes: vec![Episode::example()],
        }
    }
}

impl From<&Podcast> for Channel {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rss::validation::Validate;

    #[test]
    fn to_rss() {
        // Arrange
        let podcast = &Podcast::example();

        // Act
        let channel: Channel = podcast.into();

        // Assert
        let result = channel.validate();
        result.assert_ok();
    }
}
