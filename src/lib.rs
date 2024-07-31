pub mod color;
pub mod quantize;
use std::path::Path;

/// Reads an image file to a color buffer.
pub fn img_to_colors<P: AsRef<Path>>(path: P) -> Result<Vec<color::Color>, image::ImageError> {
    let img = image::open(path)?;
    let img = img.to_rgb8();
    let colors = img
        .chunks_exact(3)
        .map(|ch| color::Color::new(ch[0], ch[1], ch[2]))
        .collect();
    Ok(colors)
}
