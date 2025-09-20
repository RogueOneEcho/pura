use crate::prelude::*;
use crate::scrape::simplecast::*;

const CONCURRENCY: usize = 8;

impl ScrapeCommand {
    pub(super) async fn execute_simplecast(
        &self,
        podcast_id: &str,
        url: &Url,
    ) -> Result<Podcast, ScrapeSimplecastError> {
        let player_id = self.get_player_id(url).await?;
        let episode = self.get_episode(&player_id).await?;
        let podcast = self.get_podcast(&episode).await?;
        let playlist = self.get_playlist(&episode).await?;
        info!(
            "{} {} episodes of {}",
            "Found".bold(),
            playlist.len(),
            episode.podcast.title
        );
        let episodes = self.get_episodes(&playlist).await;
        let diff = playlist.len() - episodes.len();
        if diff > 0 {
            warn!("{} {} episodes due to failures", "Skipped".bold(), diff);
        }
        Ok(convert(podcast_id, podcast, episodes))
    }

    async fn get_player_id(&self, url: &Url) -> Result<String, ScrapeSimplecastError> {
        let html = self
            .http
            .get_html(url)
            .await
            .map_err(ScrapeSimplecastError::GetPage)?;
        let episode_guid = get_simplecast_episode_guid(&html)
            .ok_or_else(|| ScrapeSimplecastError::PlayerNotFound(url.clone()))?;
        trace!(
            "{} Simplecast player with episode id: {episode_guid}",
            "Found".bold()
        );
        Ok(episode_guid)
    }

    async fn get_episode(&self, id: &str) -> Result<SimplecastEpisode, ScrapeSimplecastError> {
        let episode_url = Url::parse(&format!("https://api.simplecast.com/episodes/{id}"))
            .expect("URL should be valid");
        let episode: SimplecastEpisode = self
            .http
            .get_json(&episode_url)
            .await
            .map_err(|e| ScrapeSimplecastError::GetEpisode(id.to_owned(), e))?;
        Ok(episode)
    }

    async fn get_podcast(
        &self,
        episode: &SimplecastEpisode,
    ) -> Result<SimplecastPodcast, ScrapeSimplecastError> {
        debug!(
            "{} podcast for {}",
            "Fetching".bold(),
            episode.podcast.title
        );
        let url = Url::parse(&format!(
            "https://api.simplecast.com/podcasts/{}",
            episode.podcast.id
        ))
        .expect("URL should be valid");
        self.http
            .get_json(&url)
            .await
            .map_err(|e| ScrapeSimplecastError::GetPlaylist(episode.podcast.id.clone(), e))
    }

    async fn get_playlist(
        &self,
        episode: &SimplecastEpisode,
    ) -> Result<Vec<SimplecastPlaylistEpisode>, ScrapeSimplecastError> {
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
            let mut playlist: SimplecastPlaylist =
                self.http.get_json(&playlist_url).await.map_err(|e| {
                    ScrapeSimplecastError::GetPlaylist(episode.podcast.id.clone(), e)
                })?;
            let next = playlist.episodes.pages.next.clone();
            episodes.append(&mut playlist.episodes.collection);
            let Some(link) = next else {
                break;
            };
            playlist_url = link.href;
        }
        Ok(episodes)
    }

    async fn get_episodes(&self, playlist: &[SimplecastPlaylistEpisode]) -> Vec<SimplecastEpisode> {
        debug!(
            "{} metadata for {} episodes",
            "Fetching".bold(),
            playlist.len()
        );
        stream::iter(playlist.iter().map(|episode| {
            let this = self;
            async move {
                match this.get_episode(&episode.id).await {
                    Ok(ep) => Some(ep),
                    Err(e) => {
                        warn!("{} to get episode {}", "Failed".bold(), episode.id);
                        debug!("{e}");
                        None
                    }
                }
            }
        }))
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
    }
}

fn get_simplecast_episode_guid(html: &Html) -> Option<String> {
    let mut src = get_element_attr(html, "iframe", "src");
    src.append(&mut get_element_attr(html, "iframe", "data-src"));
    src.into_iter().find_map(|url| {
        if url.is_empty() {
            return None;
        }
        let url = match Url::parse(&url) {
            Ok(url) => url,
            Err(e) => {
                warn!("Unable to parse URL: {url}\n{e}");
                return None;
            }
        };
        let host = url.host_str()?;
        if host != "player.simplecast.com" && host != "embed.simplecast.com" {
            return None;
        }
        let guid = url.path_segments()?.next()?.to_owned();
        Some(guid)
    })
}

fn get_element_attr(html: &Html, selector: &str, attr: &str) -> Vec<String> {
    html.select(&Selector::parse(selector).expect("Selector should be valid"))
        .filter_map(|element| element.attr(attr).map(str::to_owned))
        .collect()
}

fn convert(
    podcast_id: &str,
    podcast: SimplecastPodcast,
    episodes: Vec<SimplecastEpisode>,
) -> Podcast {
    let mut podcast: Podcast = podcast.into();
    podcast.id = podcast_id.to_owned();
    podcast.episodes = episodes.into_iter().map(Into::into).collect();
    podcast
}

#[derive(Debug)]
pub enum ScrapeSimplecastError {
    GetPage(HttpError),
    PlayerNotFound(Url),
    GetEpisode(String, HttpError),
    GetPlaylist(String, HttpError),
}

impl Display for ScrapeSimplecastError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            ScrapeSimplecastError::GetPage(e) => format!("Unable to get page\n{e}"),
            ScrapeSimplecastError::PlayerNotFound(url) => {
                format!("Page does not contain a Simplecast Player\nURL: {url}")
            }
            ScrapeSimplecastError::GetEpisode(id, e) => format!("Unable to get episode: {id}\n{e}"),
            ScrapeSimplecastError::GetPlaylist(id, e) => {
                format!("Unable to get playlist: {id}\n{e}")
            }
        };
        write!(f, "{} to scrape\n{reason}", "Failed".bold())
    }
}
