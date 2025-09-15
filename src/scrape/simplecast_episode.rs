use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Episode {
    pub long_description: String,
    pub audio_status: String,
    pub image_url: Url,
    #[serde(rename = "type")]
    pub episode_type: String,
    pub token: String,
    pub description: String,
    pub slug: String,
    pub number: u32,
    pub audio_file: AudioFile,
    pub title: String,
    pub episode_url: String,
    pub audio_file_size: u64,
    pub published_at: DateTime<FixedOffset>,
    pub href: Url,
    pub audio_file_path: String,
    pub enclosure_url: Url,
    pub id: String,
    pub podcast: Podcast,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AudioFile {
    pub url: String,
    pub size: u64,
    pub path_tc: String,
    pub path: String,
    pub name: String,
    pub href: Url,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Podcast {
    pub href: Url,
    pub title: String,
    pub image_url: Url,
    pub id: String,
    pub episodes: Count,
    pub display_owner_email: bool,
    pub created_at: NaiveDateTime,
    pub account_id: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Count {
    pub count: u32,
}

impl Display for Episode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match serde_yaml::to_string(self) {
            Ok(yaml) => yaml,
            Err(_) => format!("{self:?}"),
        };
        write!(f, "{message}")
    }
}
