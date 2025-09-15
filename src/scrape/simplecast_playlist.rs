use crate::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Playlist {
    pub href: Url,
    #[allow(clippy::struct_field_names)]
    #[serde(rename = "type")]
    pub playlist_type: String,
    pub title: String,
    pub image_url: Url,
    pub feed_url: Url,
    pub episodes: Episodes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Episodes {
    pub pages: Pages,
    pub collection: Vec<Episode>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pages {
    pub total: usize,
    pub previous: Option<Link>,
    pub next: Option<Link>,
    pub limit: usize,
    pub current: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub href: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Episode {
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
