use crate::download::ResizeError;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageReader};
use lofty::picture::MimeType;
use std::path::PathBuf;

pub(crate) fn resize_image(
    path: &PathBuf,
    mime_type: &MimeType,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ResizeError> {
    let src = ImageReader::open(path)
        .map_err(ResizeError::IO)?
        .decode()
        .map_err(ResizeError::Image)?;
    let mut target = Image::new(
        width,
        height,
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
            width,
            height,
            src.color().into(),
        ),
        MimeType::Jpeg => JpegEncoder::new(&mut buffer).write_image(
            target.buffer(),
            width,
            height,
            src.color().into(),
        ),
        _ => {
            return Err(ResizeError::Mime(mime_type.clone()));
        }
    };
    result.map_err(ResizeError::Image)?;
    Ok(buffer)
}
