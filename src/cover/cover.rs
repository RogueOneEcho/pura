use crate::prelude::*;

const BANNER_WIDTH: u32 = 960;
const BANNER_HEIGHT: u32 = 540;
const COVER_SIZE: u32 = 720;

pub struct CoverCommand {
    podcasts: PodcastProvider,
    http: HttpClient,
    paths: PathProvider,
}

impl CoverCommand {
    #[must_use]
    pub fn new(podcasts: PodcastProvider, http: HttpClient, paths: PathProvider) -> Self {
        Self {
            podcasts,
            http,
            paths,
        }
    }

    pub async fn execute(&self, options: CoverOptions) -> Result<(), CoverError> {
        let podcast = self
            .podcasts
            .get(&options.podcast_id)
            .map_err(CoverError::GetPodcast)?;
        let url = podcast.image_url.ok_or(CoverError::NoImage)?;
        let src = self
            .http
            .get(&url, None)
            .await
            .map_err(CoverError::GetImage)?;
        let banner = self.paths.get_output_path_for_banner(&options.podcast_id);
        let cover = self.paths.get_output_path_for_cover(&options.podcast_id);
        let banner = Resize::new(&src)
            .map_err(CoverError::Image)?
            .to_file(&banner, BANNER_WIDTH, BANNER_HEIGHT)
            .map_err(CoverError::Image)?;
        let cover = Resize::new(&src)
            .map_err(CoverError::Image)?
            .to_file(&cover, COVER_SIZE, COVER_SIZE)
            .map_err(CoverError::Image)?;
        info!("{} cover and banner images", "Created".bold());
        trace!("{}", banner.display());
        trace!("{}", cover.display());
        Ok(())
    }
}

#[derive(Debug)]
pub enum CoverError {
    GetPodcast(DatabaseError),
    NoImage,
    GetImage(HttpError),
    Image(ImageError),
}

impl Display for CoverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let reason = match self {
            CoverError::GetPodcast(e) => format!("Unable to get podcast\n{e}"),
            CoverError::NoImage => "Podcast does not have an image".to_owned(),
            CoverError::GetImage(e) => format!("Unable to get image:\n{e}"),
            CoverError::Image(e) => format!("Unable to create image:\n{e}"),
        };
        write!(
            f,
            "{} to create cover and banner images\n{reason}",
            "Failed".bold()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn cover_command() {
        // Arrange
        let _ = init_logging();
        let services = ServiceProvider::create()
            .await
            .expect("ServiceProvider should not fail");
        let command = CoverCommand::new(services.podcasts, services.http, services.paths);
        let options = CoverOptions {
            podcast_id: "irl".to_owned(),
        };

        // Act
        let result = command.execute(options).await;

        // Assert
        result.assert_ok();
    }
}
