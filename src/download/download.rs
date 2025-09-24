use crate::prelude::*;
use lofty::picture::Picture;
use tokio::task::{spawn_blocking, JoinError};

const CONCURRENCY: usize = 8;
const IMAGE_SIZE: u32 = 720;

pub struct DownloadCommand {
    paths: PathProvider,
    http: HttpClient,
    podcasts: PodcastProvider,
}

impl DownloadCommand {
    #[must_use]
    pub fn new(paths: PathProvider, http: HttpClient, podcasts: PodcastProvider) -> Self {
        Self {
            paths,
            http,
            podcasts,
        }
    }

    pub async fn execute(&self, options: DownloadOptions) -> Result<(), DownloadError> {
        let podcast = self
            .podcasts
            .get(&options.podcast_id)
            .map_err(DownloadError::GetPodcast)?;
        let results = self.process_episodes(podcast.clone(), options.year).await;
        let mut episodes = Vec::new();
        let mut errors = Vec::new();
        for result in results {
            match result {
                Ok(episode) => episodes.push(episode),
                Err(e) => errors.push(e),
            }
        }
        info!(
            "{} audio files for {} episodes",
            "Downloaded".bold(),
            episodes.len()
        );
        if !errors.is_empty() {
            warn!(
                "{} {} episodes due to failures",
                "Skipped".bold(),
                errors.len()
            );
        }
        Ok(())
    }

    async fn process_episodes(
        &self,
        mut podcast: Podcast,
        year: Option<i32>,
    ) -> Vec<Result<Episode, ProcessError>> {
        let episodes: Vec<_> = take(&mut podcast.episodes)
            .into_iter()
            .filter(|episode| {
                if let Some(year) = year {
                    if episode.published_at.year() != year {
                        return false;
                    }
                }
                let exists = self
                    .paths
                    .get_output_path_for_audio(&podcast.id, episode)
                    .exists();
                if exists {
                    trace!("{} existing episode: {episode}", "Skipping".bold());
                }
                !exists
            })
            .collect();
        debug!(
            "{} audio files for {} episodes",
            "Downloading".bold(),
            episodes.len()
        );
        stream::iter(episodes.into_iter().map(|episode| {
            let this = self;
            let podcast = podcast.clone();
            async move {
                let result = this.process_episode(&podcast, episode).await;
                if let Err(e) = &result {
                    warn!("{e}");
                }
                result
            }
        }))
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<_>>()
        .await
    }

    async fn process_episode(
        &self,
        podcast: &Podcast,
        episode: Episode,
    ) -> Result<Episode, ProcessError> {
        let path = self.download_episode(&episode).await?;
        let audio_path = self.copy_episode(&podcast.id, &episode, &path).await?;
        let cover = self.download_image(&episode).await?;
        trace!("{} tags for {episode}", "Setting".bold());
        Tag::execute(podcast, &episode, cover, &audio_path)
            .map_err(|e| ProcessError::Tag(episode.get_file_stem(), audio_path.clone(), e))?;
        Ok(episode)
    }

    async fn download_episode(&self, episode: &Episode) -> Result<PathBuf, ProcessError> {
        self.http
            .get(&episode.audio_url, Some(MP3_EXTENSION))
            .await
            .map_err(|e| ProcessError::DownloadAudio(episode.get_file_stem(), e))
    }

    async fn copy_episode(
        &self,
        podcast_id: &str,
        episode: &Episode,
        source_path: &PathBuf,
    ) -> Result<PathBuf, ProcessError> {
        let destination_path = self.paths.get_output_path_for_audio(podcast_id, episode);
        create_parent_dir_if_not_exist(&destination_path)
            .await
            .map_err(|e| {
                ProcessError::IO(
                    episode.get_file_stem(),
                    destination_path
                        .parent()
                        .expect("path should have a parent")
                        .into(),
                    e,
                )
            })?;
        trace!(
            "{} {episode}\nSource: {}\nTarget: {}",
            "Copying".bold(),
            source_path.display(),
            destination_path.display()
        );
        copy(&source_path, &destination_path)
            .await
            .map_err(|e| ProcessError::IO(episode.get_file_stem(), source_path.into(), e))?;
        Ok(destination_path)
    }

    async fn download_image(&self, episode: &Episode) -> Result<Option<Picture>, ProcessError> {
        let Some(url) = &episode.image_url else {
            return Ok(None);
        };
        trace!("{} image for episode: {episode}", "Downloading".bold());
        let extension = url.get_extension();
        let path = self
            .http
            .get(url, extension.as_deref())
            .await
            .map_err(|e| ProcessError::DownloadImage(episode.get_file_stem(), e))?;
        trace!("{} image for episode: {episode}", "Resizing".bold());
        let picture = spawn_blocking(move || -> Result<Picture, ImageError> {
            Resize::new(&path)?.to_picture(IMAGE_SIZE, IMAGE_SIZE)
        })
        .await
        .map_err(|e| ProcessError::Task(episode.get_file_stem(), e))?
        .map_err(|e| ProcessError::ResizeImage(episode.get_file_stem(), e))?;
        trace!("{} image for episode: {episode}", "Resized".bold());
        Ok(Some(picture))
    }
}

#[allow(clippy::absolute_paths)]
#[derive(Debug)]
pub enum DownloadError {
    GetPodcast(DatabaseError),
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            DownloadError::GetPodcast(e) => format!("Unable to get podcast\n{e}"),
        };
        write!(f, "{} to download\n{reason}", "Failed".bold())
    }
}

#[derive(Debug)]
#[allow(clippy::absolute_paths)]
pub enum ProcessError {
    DownloadAudio(String, HttpError),
    IO(String, PathBuf, std::io::Error),
    Tag(String, PathBuf, lofty::error::LoftyError),
    DownloadImage(String, HttpError),
    Task(String, JoinError),
    ResizeImage(String, ImageError),
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self {
            ProcessError::DownloadAudio(id, e) => {
                format!("Unable to download audio for episode: {id}\n{e}")
            }
            ProcessError::IO(id, path, e) => {
                format!(
                    "An I/O error occurred while processing episode: {id}\nPath: {}\n{e}",
                    path.display()
                )
            }
            ProcessError::Tag(id, path, e) => {
                format!(
                    "A tag error occurred while processing episode: {id}\nPath: {}\n{e}",
                    path.display()
                )
            }
            ProcessError::DownloadImage(id, e) => {
                format!("Unable to download image for episode: {id}\n{e}")
            }
            ProcessError::Task(id, e) => {
                format!("Unable to resize image for episode: {id}\nA task error occurred:\n{e}")
            }
            ProcessError::ResizeImage(id, e) => {
                format!("Unable to resize image for episode: {id}\n{e}")
            }
        };
        write!(f, "{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn download_command() {
        // Arrange
        let _ = init_logging();
        let services = ServiceProvider::create()
            .await
            .expect("ServiceProvider should not fail");
        let command = DownloadCommand::new(services.paths, services.http, services.podcasts);
        let options = DownloadOptions {
            podcast_id: "irl".to_owned(),
            year: Some(2019),
        };

        // Act
        let result = command.execute(options).await;

        // Assert
        result.assert_ok();
    }

    #[tokio::test]
    pub async fn process_episode() {
        // Arrange
        let _ = init_logging();
        let services = ServiceProvider::create()
            .await
            .expect("ServiceProvider should not fail");
        let podcast = services.podcasts.get("irl").expect("podcast should exist");
        let command = DownloadCommand::new(services.paths, services.http, services.podcasts);
        let episode = podcast
            .episodes
            .get(1)
            .expect("should be at least one episode")
            .clone();

        // Act
        let result = command.process_episode(&podcast, episode).await;

        // Assert
        result.assert_ok();
    }
}
