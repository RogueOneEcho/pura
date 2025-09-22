use crate::prelude::*;

pub struct FeedsCommand {
    podcasts: PodcastProvider,
    paths: PathProvider,
}

impl FeedsCommand {
    #[must_use]
    pub fn new(podcasts: PodcastProvider, paths: PathProvider) -> Self {
        Self { podcasts, paths }
    }

    pub async fn execute(&self, options: FeedsOptions) -> Result<(), FeedsError> {
        let podcast = self
            .podcasts
            .get(&options.podcast_id)
            .map_err(FeedsError::GetPodcast)?;
        let feeds = self.save_feeds(&podcast).await?;
        info!("{} {} rss feeds", "Created".bold(), feeds.len());
        Ok(())
    }

    async fn save_feeds(&self, podcast: &Podcast) -> Result<Vec<PathBuf>, FeedsError> {
        let mut paths = Vec::new();
        paths.push(self.save_feed(podcast, None, None).await?);
        let mut podcast = podcast.clone();
        let groups = group_by_season(take(&mut podcast.episodes));
        for (season, episodes) in groups {
            let mut p = podcast.clone();
            p.episodes = episodes;
            paths.push(self.save_feed(&p, season, None).await?);
            let year_groups = group_by_year(take(&mut p.episodes));
            for (year, episodes) in year_groups {
                p.episodes = episodes;
                paths.push(self.save_feed(&p, season, Some(year)).await?);
            }
        }
        Ok(paths)
    }

    async fn save_feed(
        &self,
        podcast: &Podcast,
        season: Option<usize>,
        year: Option<i32>,
    ) -> Result<PathBuf, FeedsError> {
        let mut channel: RssChannel = podcast.into();
        for item in &mut channel.items {
            self.replace_enclosure(podcast, item);
        }
        let xml = channel.to_string();
        let path = self
            .paths
            .get_output_path_for_rss(&podcast.id, season, year);
        let mut file = AsyncFile::create(&path)
            .await
            .map_err(|e| FeedsError::Xml(path.clone(), e))?;
        file.write_all(xml.as_bytes())
            .await
            .map_err(|e| FeedsError::Xml(path.clone(), e))?;
        file.flush()
            .await
            .map_err(|e| FeedsError::Xml(path.clone(), e))?;
        Ok(path)
    }

    fn replace_enclosure(&self, podcast: &Podcast, item: &mut RssItem) -> Option<()> {
        let guid = item.guid.clone()?;
        let episode = podcast
            .episodes
            .iter()
            .find(|episode| episode.id == guid.value)?;
        let enclosure = item.enclosure.as_mut()?;
        enclosure.url = self
            .paths
            .get_url_for_audio(&podcast.id, episode)?
            .to_string();
        Some(())
    }
}

fn group_by_season(episodes: Vec<Episode>) -> HashMap<Option<usize>, Vec<Episode>> {
    let mut groups: HashMap<Option<usize>, Vec<Episode>> = HashMap::new();
    for episode in episodes {
        let group = groups.entry(episode.season).or_default();
        group.push(episode);
    }
    groups
}

fn group_by_year(episodes: Vec<Episode>) -> HashMap<i32, Vec<Episode>> {
    let mut groups: HashMap<i32, Vec<Episode>> = HashMap::new();
    for episode in episodes {
        let year = episode.published_at.year();
        let group = groups.entry(year).or_default();
        group.push(episode);
    }
    groups
}

#[allow(clippy::absolute_paths)]
#[derive(Debug)]
pub enum FeedsError {
    GetPodcast(DatabaseError),
    Xml(PathBuf, std::io::Error),
}

impl Display for FeedsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            FeedsError::GetPodcast(e) => format!("Unable to get podcast\n{e}"),
            FeedsError::Xml(path, e) => {
                format!("Unable to write RSS\nPath: {}\n{e}", path.display())
            }
        };
        write!(f, "{} to create RSS feeds\n{reason}", "Failed".bold())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn feeds_command() {
        // Arrange
        let _ = init_logging();
        let services = ServiceProvider::create()
            .await
            .expect("ServiceProvider should not fail");
        let command = FeedsCommand::new(services.podcasts, services.paths);
        let options = FeedsOptions {
            podcast_id: "irl".to_owned(),
        };

        // Act
        let result = command.execute(options).await;

        // Assert
        result.assert_ok();
    }
}
