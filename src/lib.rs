pub mod color;
pub mod octree;
pub mod quantize;
use std::path::Path;

type ResColors = Result<Vec<color::Rgb24>, image::ImageError>;

/// Reads an image file to an RGB24 buffer.
pub fn img_to_rgb24<P: AsRef<Path>>(path: P, alpha_min: u8) -> ResColors {
    let img = image::open(path)?;
    let img = img.to_rgba8();
    let colors = img
        .chunks_exact(4)
        .filter(|c| c[3] >= alpha_min)
        .map(|ch| color::Rgb24::new(ch[0], ch[1], ch[2]))
        .collect();
    Ok(colors)
}
