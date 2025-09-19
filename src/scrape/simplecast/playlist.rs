use crate::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastPlaylist {
    pub href: Url,
    #[allow(clippy::struct_field_names)]
    #[serde(rename = "type")]
    pub playlist_type: String,
    pub title: String,
    pub image_url: Url,
    pub feed_url: Url,
    pub episodes: SimplecastEpisodes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastEpisodes {
    pub pages: SimplecastPages,
    pub collection: Vec<SimplecastPlaylistEpisode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastPages {
    pub total: usize,
    pub previous: Option<SimplecastLink>,
    pub next: Option<SimplecastLink>,
    pub limit: usize,
    pub current: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastLink {
    pub href: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastPlaylistEpisode {
    #[allow(clippy::struct_field_names)]
    #[serde(rename = "type")]
    pub episode_type: String,
    pub title: String,
    pub season_number: Option<usize>,
    pub number: Option<usize>,
    pub image_url: Option<Url>,
    pub id: String,
    pub enclosure_url: Url,
    pub duration: usize,
}
