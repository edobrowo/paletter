pub mod color;
pub mod median_cut;
pub mod octree;

use std::path::Path;

use color::Rgb24;
use median_cut::median_cut;
use octree::octree;

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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Method {
    MedianCut,
    Octree,
}

/// Quantize a palette with the specified method.
pub fn solve(method: Method, colors: Vec<Rgb24>, palette_size: usize) -> Vec<Rgb24> {
    if palette_size >= colors.len() {
        return colors.to_vec();
    }

    match method {
        Method::MedianCut => median_cut(colors, palette_size),
        Method::Octree => octree(&colors, palette_size),
    }
}
