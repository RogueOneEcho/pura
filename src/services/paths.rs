use crate::prelude::*;
use std::env::current_dir;

const DEFAULT_CACHE_DIR: &str = "cache";
const DEFAULT_OUTPUT_DIR: &str = "output";
const HTTP_DIR: &str = "http";
const PODCASTS_DIR: &str = "podcasts";
pub(crate) const HEAD_EXTENSION: &str = "head";
pub(crate) const HTML_EXTENSION: &str = "html";
pub(crate) const JSON_EXTENSION: &str = "json";
pub(crate) const MP3_EXTENSION: &str = "mp3";
pub(crate) const XML_EXTENSION: &str = "xml";
const RSS_FILE_NAME: &str = "feed.xml";

#[derive(Default)]
pub struct PathProvider {
    options: AppOptions,
}

impl PathProvider {
    pub fn new(options: AppOptions) -> Self {
        Self { options }
    }

    pub(crate) fn get_cache_dir(&self) -> PathBuf {
        self.options
            .cache_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CACHE_DIR))
    }

    pub(crate) fn get_http_dir(&self) -> PathBuf {
        self.get_cache_dir().join(HTTP_DIR)
    }

    pub(crate) fn get_podcast_dir(&self) -> PathBuf {
        self.get_cache_dir().join(PODCASTS_DIR)
    }

    pub(crate) fn get_output_dir(&self) -> PathBuf {
        self.options
            .output_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIR))
    }

    #[allow(clippy::unused_self)]
    fn get_sub_path_for_audio(&self, podcast_id: &str, episode: &Episode) -> PathBuf {
        let season = episode.get_formatted_season();
        let year = episode.published_at.year().to_string();
        let file_stem = episode.get_file_stem();
        let extension = episode
            .audio_url
            .get_extension()
            .unwrap_or(MP3_EXTENSION.to_owned());
        PathBuf::new()
            .join(podcast_id)
            .join(season)
            .join(year)
            .join(file_stem)
            .with_extension(extension)
    }

    pub(crate) fn get_output_path_for_audio(&self, podcast_id: &str, episode: &Episode) -> PathBuf {
        self.get_output_dir()
            .join(self.get_sub_path_for_audio(podcast_id, episode))
    }

    pub(crate) fn get_url_for_audio(&self, podcast_id: &str, episode: &Episode) -> Option<Url> {
        if let Some(base) = &self.options.server_base {
            let path = self.get_sub_path_for_audio(podcast_id, episode);
            base.join(path.to_string_lossy().as_ref()).ok()
        } else {
            let path = current_dir()
                .ok()?
                .join(self.get_output_path_for_audio(podcast_id, episode));
            Url::from_file_path(path).ok()
        }
    }

    pub(crate) fn get_output_path_for_rss(
        &self,
        podcast_id: &str,
        season: Option<usize>,
        year: Option<i32>,
    ) -> PathBuf {
        assert!(!podcast_id.is_empty(), "podcast id should not be empty");
        let path = self.get_output_dir().join(podcast_id);
        if season.is_none() && year.is_none() {
            return path.join(RSS_FILE_NAME);
        }
        let season = Episode::format_season(season);
        let year = year.map(|s| s.to_string()).unwrap_or_default();
        path.join(season).join(year).join(RSS_FILE_NAME)
    }

    pub(crate) fn get_output_path_for_cover(&self, podcast_id: &str) -> PathBuf {
        self.get_output_dir().join(podcast_id).join("cover.jpg")
    }

    pub(crate) fn get_output_path_for_banner(&self, podcast_id: &str) -> PathBuf {
        self.get_output_dir().join(podcast_id).join("banner.jpg")
    }

    pub(crate) fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let dirs = vec![
            ("Cache directory", self.get_cache_dir()),
            ("HTTP cache directory", self.get_http_dir()),
            ("Podcasts cache directory", self.get_podcast_dir()),
            ("Output directory", self.get_output_dir()),
        ];
        for (name, dir) in dirs {
            if let Err(e) = Validate::directory(dir) {
                errors.push(ValidationError::Path(name.to_owned(), e));
            }
        }
        errors.to_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate() {
        // Arrange
        let paths = PathProvider::default();

        // Act
        let result = paths.validate();

        // Assert
        result.assert_ok_debug();
    }

    #[test]
    fn get_output_path_for_rss() {
        // Arrange
        let paths = PathProvider::default();

        // Act
        // Assert
        assert_eq!(
            paths.get_output_path_for_rss("abc", None, None),
            PathBuf::from("output/abc/feed.xml")
        );
        assert_eq!(
            paths.get_output_path_for_rss("abc", Some(1), None),
            PathBuf::from("output/abc/S01/feed.xml")
        );
        assert_eq!(
            paths.get_output_path_for_rss("abc", Some(1), Some(1234)),
            PathBuf::from("output/abc/S01/1234/feed.xml")
        );
        assert_eq!(
            paths.get_output_path_for_rss("abc", None, Some(1234)),
            PathBuf::from("output/abc/S00/1234/feed.xml")
        );
    }
}
