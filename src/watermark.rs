use std::path::Path;

use image::ImageError;

/// Apply a watermark to an image.
/// The watermark is applied to the bottom left corner of the image.
/// You can adjust the size of the watermark with the `watermark_size` parameter.
/// A value around `0.4` is recommended.
pub fn watermark_image(
    source: &Path,
    destination_dir: &Path,
    watermark_path: &Path,
    watermark_scale: f64,
) -> Result<(), ImageError> {
    let image = image::open(source)?;
    let watermark = image::open(watermark_path)?;
    let width = image.width();

    let watermark_width = (width as f64 * watermark_scale) as u32;
    let watermark_height =
        (watermark_width as f64 * watermark.height() as f64 / watermark.width() as f64) as u32;

    let watermark = watermark.resize_exact(
        watermark_width,
        watermark_height,
        image::imageops::FilterType::Nearest,
    );

    let mut image = image.to_rgba8();
    let y = image.height() as i64 - watermark.height() as i64 - 1;
    image::imageops::overlay(&mut image, &watermark, 1, y);
    image.save(destination_dir.join(source.file_name().unwrap()))?;

    Ok(())
}
