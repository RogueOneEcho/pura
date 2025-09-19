use crate::prelude::*;
use rss::extension::itunes::ITunesItemExtension;
use rss::{Enclosure, Guid, Item};
use strum_macros::AsRefStr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct Episode {
    /// GUID
    pub id: String,
    /// Title
    pub title: String,
    /// HTML formatted description
    pub description: String,
    /// URL of media file including a file extension
    /// - Supported file formats include M4A, MP3, MOV, MP4, M4V, and PDF
    pub audio_url: Url,
    /// Size of audio file in bytes
    pub audio_file_size: u64,
    /// Mime type of audio file
    pub audio_content_type: String,
    /// Duration in seconds
    pub duration: Option<u64>,
    /// URL of JPEG or PNG artwork
    /// - Min: 1400 x 1400 px
    /// - Max: 3000 x 3000 px
    pub image_url: Option<Url>,
    /// Parental advisory information
    pub explicit: bool,
    /// Episode type
    pub episode_type: EpisodeType,
    /// Season number
    pub season: Option<usize>,
    /// Episode number
    pub number: Option<usize>,
    /// Date and time episode was released
    pub published_at: DateTime<FixedOffset>,
}

/// Episode type
#[derive(AsRefStr, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum EpisodeType {
    /// Complete content
    #[default]
    Full,
    /// Short promotional piece
    /// - Show trailer has no season or episode number
    /// - Season trailer has a season number and no episode number
    /// - Episode trailer has an episode number and optionally a season number
    Trailer,
    /// Extra content
    /// - Show bonus has no season or episode number
    /// - Season bonus has a season number and no episode number
    /// - Episode specific bonus has an episode number and optionally a season number
    Bonus,
}

impl Episode {
    pub(crate) fn get_file_stem(&self) -> String {
        let date = self.get_formatted_date();
        let number = self.get_padded_number();
        let title = self.get_sanitized_title();
        format!("{date} {number} {title}")
    }

    fn get_padded_number(&self) -> String {
        if let Some(number) = self.number {
            format!("{number:04}")
        } else {
            "____".to_owned()
        }
    }

    pub(crate) fn get_formatted_season(&self) -> String {
        let number = self.season.unwrap_or(0);
        format!("S{number:02}")
    }

    fn get_formatted_date(&self) -> String {
        self.published_at.format("%Y-%m-%d").to_string()
    }

    fn get_sanitized_title(&self) -> String {
        Sanitizer::execute(&self.title).trim().to_owned()
    }

    #[cfg(test)]
    pub(crate) fn example() -> Self {
        Self {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Lorem ipsum dolor sit amet".to_string(),
            description: "Aenean sit amet sem quis velit viverra vestibulum. Vivamus aliquam mattis ipsum, a dignissim elit pulvinar vitae. Aliquam neque risus, tincidunt sit amet elit quis, malesuada ultrices urna.".to_string(),
            image_url: Some(Url::parse("https://example.com/image.jpg").expect("URL should be valid")),
            audio_url: Url::parse("https://example.com/season-1/episode-1.mp3").expect("URL should be valid"),
            episode_type: Default::default(),
            season: Some(2),
            number: Some(3),
            audio_file_size: 1024,
            audio_content_type: "audio/mpeg".to_owned(),
            published_at: Default::default(),
            duration: None,
            explicit: false,
        }
    }
}

impl Display for Episode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.get_file_stem())
    }
}

impl From<&Episode> for Item {
    fn from(episode: &Episode) -> Self {
        Item {
            title: Some(episode.title.clone()),
            link: Some(episode.audio_url.to_string()),
            guid: Some(Guid {
                value: episode.id.clone(),
                permalink: false,
            }),
            description: Some(episode.description.clone()),
            pub_date: Some(episode.published_at.to_rfc2822()),
            enclosure: Some(episode.into()),
            itunes_ext: Some(episode.into()),
            ..Default::default()
        }
    }
}

impl From<&Episode> for Enclosure {
    fn from(episode: &Episode) -> Self {
        Enclosure {
            url: episode.audio_url.to_string(),
            length: episode.audio_file_size.to_string(),
            mime_type: episode.audio_content_type.clone(),
        }
    }
}

impl From<&Episode> for ITunesItemExtension {
    fn from(episode: &Episode) -> Self {
        ITunesItemExtension {
            duration: episode.duration.map(|d| d.to_string()),
            explicit: Some(episode.explicit.to_string()),
            image: episode.image_url.as_ref().map(ToString::to_string),
            episode: episode.number.map(|n| n.to_string()),
            season: episode.season.map(|s| s.to_string()),
            episode_type: Some(episode.episode_type.as_ref().to_lowercase()),
            summary: Some(episode.description.clone()),
            ..Default::default()
        }
    }
}

impl From<String> for EpisodeType {
    fn from(value: String) -> Self {
        if value == "full" {
            EpisodeType::Full
        } else if value == "trailer" {
            EpisodeType::Trailer
        } else {
            EpisodeType::Bonus
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_file_stem() {
        // Arrange
        let episode = Episode::example();

        // Act
        let result = episode.get_file_stem();

        // Assert
        assert_eq!(result, "1970-01-01 0003 Lorem ipsum dolor sit amet");
    }
}
