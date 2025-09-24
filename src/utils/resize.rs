use crate::prelude::*;
use fast_image_resize::images::Image;
use fast_image_resize::{ImageBufferError, IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ImageEncoder, ImageFormat, ImageReader};

const RESIZE_ALGORITHM: ResizeAlg = ResizeAlg::Nearest;

pub struct Resize;

impl Resize {
    pub(crate) fn execute(path: &PathBuf, width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
        let reader = ImageReader::open(path)
            .map_err(ImageError::IO)?
            .with_guessed_format()
            .map_err(ImageError::IO)?;
        let format = reader.format().ok_or(ImageError::UnknownFormat)?;
        let src = reader.decode().map_err(ImageError::Image)?;
        let mut target = Image::new(
            width,
            height,
            src.pixel_type()
                .expect("source image should have a pixel type"),
        );
        let mut resizer = Resizer::new();
        let options = ResizeOptions::default().resize_alg(RESIZE_ALGORITHM);
        resizer
            .resize(&src, &mut target, &options)
            .map_err(ImageError::Resize)?;
        let mut buffer = Vec::new();
        resizer
            .resize(&src, &mut target, &options)
            .map_err(ImageError::Resize)?;
        let result = match format {
            ImageFormat::Png => PngEncoder::new(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                src.color().into(),
            ),
            ImageFormat::Jpeg => JpegEncoder::new(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                src.color().into(),
            ),
            ImageFormat::Gif => GifEncoder::new(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                src.color().into(),
            ),
            ImageFormat::WebP => WebPEncoder::new_lossless(&mut buffer).write_image(
                target.buffer(),
                width,
                height,
                src.color().into(),
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
            let result = Resize::execute(&path, 100, 100);

            // Assert
            let bytes = result.assert_ok();
            assert!(!bytes.is_empty());
        }
    }
}
