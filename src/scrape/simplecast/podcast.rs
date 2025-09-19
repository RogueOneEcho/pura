use super::*;
use crate::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SimplecastPodcast {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub podcast_type: String,
    pub site: SimplecastSite,
    pub language: String,
    pub authors: SimplecastAuthors,
    pub copyright: Option<String>,
    pub image_url: Option<Url>,
    pub published_at: DateTime<FixedOffset>,
    pub created_at: NaiveDateTime,
    pub is_explicit: bool,
}

impl From<SimplecastPodcast> for Podcast {
    fn from(podcast: SimplecastPodcast) -> Self {
        Podcast {
            id: podcast.id.clone(),
            guid: podcast.id,
            title: podcast.title,
            description: podcast.description,
            image_url: podcast.image_url,
            language: podcast.language,
            category: None,
            sub_category: None,
            explicit: podcast.is_explicit,
            author: podcast.authors.collection.first().map(|a| a.name.clone()),
            link: podcast.site.external_website,
            podcast_type: podcast.podcast_type.into(),
            copyright: podcast.copyright,
            created_at: podcast.created_at,
            episodes: vec![],
        }
    }
}
