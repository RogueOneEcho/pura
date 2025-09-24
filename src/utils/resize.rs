use crate::prelude::*;
use fast_image_resize::images::Image;
use fast_image_resize::{ImageBufferError, IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageFormat, ImageReader};
use lofty::picture::{MimeType, Picture, PictureType};
use std::fs::write;

const RESIZE_ALGORITHM: ResizeAlg = ResizeAlg::Nearest;

pub struct Resize {
    format: ImageFormat,
    image: DynamicImage,
    color_type: ExtendedColorType,
}

impl Resize {
    pub(crate) fn new(path: &PathBuf) -> Result<Resize, ImageError> {
        let reader = ImageReader::open(path)
            .map_err(ImageError::IO)?
            .with_guessed_format()
            .map_err(ImageError::IO)?;
        let format = reader.format().ok_or(ImageError::UnknownFormat)?;
        let image = reader.decode().map_err(ImageError::Image)?;
        let color_type = image.color().into();
        Ok(Self {
            format,
            image,
            color_type,
        })
    }

    pub(crate) fn to_file(
        &self,
        path: &Path,
        width: u32,
        height: u32,
    ) -> Result<PathBuf, ImageError> {
        let bytes = self.to_bytes(width, height)?;
        let extension = self
            .format
            .extensions_str()
            .first()
            .expect("should be at least one image extension");
        let path = path.with_extension(extension);
        write(&path, bytes).map_err(ImageError::IO)?;
        Ok(path)
    }

    pub(crate) fn to_picture(&self, width: u32, height: u32) -> Result<Picture, ImageError> {
        let mime = match self.format {
            ImageFormat::Png => MimeType::Png,
            ImageFormat::Jpeg => MimeType::Jpeg,
            ImageFormat::Gif => MimeType::Gif,
            format => MimeType::from_str(format.to_mime_type()),
        };
        let bytes = self.to_bytes(width, height)?;
        Ok(Picture::new_unchecked(
            PictureType::CoverFront,
            Some(mime),
            None,
            bytes,
        ))
    }

    fn to_bytes(&self, width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
        let mut target = Image::new(
            width,
            height,
            self.image
                .pixel_type()
                .expect("source image should have a pixel type"),
        );
        let mut resizer = Resizer::new();
        let options = ResizeOptions::default()
            .resize_alg(RESIZE_ALGORITHM)
            .fit_into_destination(None);
        resizer
            .resize(&self.image, &mut target, &options)
            .map_err(ImageError::Resize)?;
        let mut buffer = Vec::new();
        let result = match self.format {
            ImageFormat::Png => PngEncoder::new(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                self.color_type,
            ),
            ImageFormat::Jpeg => JpegEncoder::new(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                self.color_type,
            ),
            ImageFormat::Gif => GifEncoder::new(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                self.color_type,
            ),
            ImageFormat::WebP => WebPEncoder::new_lossless(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                self.color_type,
            ),
            format => return Err(ImageError::UnsupportedFormat(format)),
        };
        result.map_err(ImageError::Image)?;
        Ok(buffer)
    }
}

#[derive(Debug)]
#[allow(clippy::absolute_paths)]
pub enum ImageError {
    IO(std::io::Error),
    UnknownFormat,
    Image(image::error::ImageError),
    ImageBuffer(ImageBufferError),
    Resize(fast_image_resize::ResizeError),
    UnsupportedFormat(ImageFormat),
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self {
            ImageError::IO(e) => {
                format!("An I/O error occurred: {e}")
            }
            ImageError::UnknownFormat => "Unable to determine image format".to_owned(),
            ImageError::Image(e) => {
                format!("An image error occurred: {e}")
            }
            ImageError::ImageBuffer(e) => {
                format!("An image buffer error occurred: {e}")
            }
            ImageError::Resize(e) => {
                format!("A resize error occurred: {e}")
            }
            ImageError::UnsupportedFormat(format) => {
                format!("Unable to encode image format: {format:?}")
            }
        };
        write!(f, "{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "uses httpbin.org"]
    pub async fn resize_jpeg() {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let formats = vec!["jpeg", "png", "webp"];
        for format in formats {
            eprintln!("format: {format}");
            let url = Url::parse(&format!("https://httpbin.org/image/{format}"))
                .expect("url should be valid");
            let path = http
                .get(&url, None)
                .await
                .expect("get image should not fail");

            // Act
            let result = Resize::new(&path).assert_ok().to_bytes(100, 100);

            // Assert
            let bytes = result.assert_ok();
            assert!(!bytes.is_empty());
        }
    }
}
