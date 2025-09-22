use crate::prelude::*;
use chrono::Datelike;
use fast_image_resize::images::Image;
use fast_image_resize::{ImageBufferError, IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageReader};
use lofty::config::WriteOptions;
use lofty::error::LoftyError;
use lofty::id3::v2::Id3v2Tag;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::{Accessor, TagExt};
use lofty::tag::TagType;
use std::mem::take;
use tokio::fs::copy;
use tokio::fs::create_dir_all;
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

    pub async fn execute(&self, podcast_id: &str, year: Option<i32>) -> Result<(), DownloadError> {
        let podcast = self
            .podcasts
            .get(podcast_id)
            .map_err(DownloadError::GetPodcast)?;
        let results = self.process_episodes(podcast.clone(), year).await;
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
        set_episode_tags(podcast, &episode, &audio_path, cover)
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
        let destination_dir = destination_path
            .parent()
            .expect("output path should have parent directory");
        if !destination_dir.exists() {
            trace!("Creating directory: {}", destination_dir.display());
            create_dir_all(&destination_dir).await.map_err(|e| {
                ProcessError::IO(episode.get_file_stem(), destination_dir.into(), e)
            })?;
        }
        trace!(
            "{} {episode}\nSource: {}\nTarget: {}",
            "Copying".bold(),
            source_path.display(),
            destination_path.display()
        );
        copy(&source_path, &destination_path)
            .await
            .map_err(|e| ProcessError::IO(episode.get_file_stem(), destination_dir.into(), e))?;
        Ok(destination_path)
    }

    async fn download_image(&self, episode: &Episode) -> Result<Option<Picture>, ProcessError> {
        let Some(url) = &episode.image_url else {
            return Ok(None);
        };
        trace!("{} image for episode: {episode}", "Downloading".bold());
        let extension = url.get_extension();
        let mime_type = match extension.clone().map(|e| e.to_lowercase()).as_deref() {
            Some(JPG_EXTENSION | JPEG_EXTENSION) => MimeType::Jpeg,
            Some(PNG_EXTENSION) => MimeType::Png,
            _ => {
                warn!(
                    "Unable to determine mimetype of image for episode: {episode} \nURL: {:?}",
                    episode.image_url
                );
                MimeType::Jpeg
            }
        };
        let path = self
            .http
            .get(url, extension.as_deref())
            .await
            .map_err(|e| ProcessError::DownloadImage(episode.get_file_stem(), e))?;
        trace!("{} image for episode: {episode}", "Resizing".bold());
        let m = mime_type.clone();
        let img_bytes =
            spawn_blocking(move || -> Result<Vec<u8>, ResizeError> { resize_image(&path, &m) })
                .await
                .map_err(|e| ProcessError::Task(episode.get_file_stem(), e))?
                .map_err(|e| ProcessError::ResizeImage(episode.get_file_stem(), e))?;
        trace!("{} image for episode: {episode}", "Resized".bold());
        let cover =
            Picture::new_unchecked(PictureType::CoverFront, Some(mime_type), None, img_bytes);
        Ok(Some(cover))
    }
}

fn resize_image(path: &PathBuf, mime_type: &MimeType) -> Result<Vec<u8>, ResizeError> {
    let src = ImageReader::open(path)
        .map_err(ResizeError::IO)?
        .decode()
        .map_err(ResizeError::Image)?;
    let mut target = Image::new(
        IMAGE_SIZE,
        IMAGE_SIZE,
        src.pixel_type()
            .expect("source image should have a pixel type"),
    );
    let mut resizer = Resizer::new();
    let options = ResizeOptions::default().resize_alg(ResizeAlg::Nearest);
    resizer
        .resize(&src, &mut target, &options)
        .map_err(ResizeError::Resize)?;
    let mut buffer = Vec::new();
    let result = match mime_type {
        MimeType::Png => PngEncoder::new(&mut buffer).write_image(
            target.buffer(),
            IMAGE_SIZE,
            IMAGE_SIZE,
            src.color().into(),
        ),
        MimeType::Jpeg => JpegEncoder::new(&mut buffer).write_image(
            target.buffer(),
            IMAGE_SIZE,
            IMAGE_SIZE,
            src.color().into(),
        ),
        _ => {
            return Err(ResizeError::Mime(mime_type.clone()));
        }
    };
    result.map_err(ResizeError::Image)?;
    Ok(buffer)
}

fn set_episode_tags(
    podcast: &Podcast,
    episode: &Episode,
    path: &PathBuf,
    cover: Option<Picture>,
) -> Result<(), LoftyError> {
    TagType::Ape.remove_from_path(path)?;
    TagType::Id3v1.remove_from_path(path)?;
    TagType::Id3v2.remove_from_path(path)?;
    let tag = create_tags(podcast, episode, cover);
    trace!("{} tags for {episode}", "Setting".bold());
    tag.save_to_path(path, WriteOptions::default())?;
    Ok(())
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn create_tags(podcast: &Podcast, episode: &Episode, cover: Option<Picture>) -> Id3v2Tag {
    let mut tag = Id3v2Tag::default();
    tag.set_title(episode.title.trim().to_owned());
    tag.set_artist(podcast.title.clone());
    if let Some(season) = episode.season {
        tag.set_album(format!("Season {season}"));
    }
    tag.set_disk(episode.season.unwrap_or_default() as u32);
    let year = episode.published_at.year() as u32;
    tag.set_year(year);
    if let Some(number) = episode.number {
        tag.set_track(number as u32);
    }
    tag.set_comment(episode.description.clone());
    if let Some(cover) = cover {
        tag.insert_picture(cover);
    }
    tag
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
    Tag(String, PathBuf, LoftyError),
    DownloadImage(String, HttpError),
    Task(String, JoinError),
    ResizeImage(String, ResizeError),
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

#[derive(Debug)]
#[allow(clippy::absolute_paths)]
pub enum ResizeError {
    IO(std::io::Error),
    Image(image::error::ImageError),
    ImageBuffer(ImageBufferError),
    Resize(fast_image_resize::ResizeError),
    Mime(MimeType),
}

impl Display for ResizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self {
            ResizeError::IO(e) => {
                format!("An I/O error occurred: {e}")
            }
            ResizeError::Image(e) => {
                format!("An image error occurred: {e}")
            }
            ResizeError::ImageBuffer(e) => {
                format!("An image buffer error occurred: {e}")
            }
            ResizeError::Resize(e) => {
                format!("A resize error occurred: {e}")
            }
            ResizeError::Mime(mime_type) => {
                format!("Unable to encode image type: {mime_type}")
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

        // Act
        let result = command.execute("irl", Some(2019)).await;

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
