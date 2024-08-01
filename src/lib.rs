pub mod color;
pub mod quantize;
use std::path::Path;

type ResColors = Result<Vec<color::Color>, image::ImageError>;

/// Reads an image file to a color buffer.
pub fn img_to_colors<P: AsRef<Path>>(path: P, alpha_min: u8) -> ResColors {
    let img = image::open(path)?;
    let img = img.to_rgba8();
    let colors = img
        .chunks_exact(4)
        .filter(|c| c[3] >= alpha_min)
        .map(|ch| color::Color::new(ch[0], ch[1], ch[2]))
        .collect();
    Ok(colors)
}
