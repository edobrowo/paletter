use clap::Parser;
use image;
use termcolor::{self, WriteColor};
use std::fmt;
use std::io::Write;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The number of colors that the image should be quantized into.
    #[clap(required = true, num_args = 1)]
    palette_size: usize,

    /// List of paths to image files. A palette will be generated for each image.
    #[arg(required = true, num_args = 1..)]
    path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Channel {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn max_channel(colors: &[Color]) -> (Channel, u8) {
        use std::cmp::Ordering;

        // let range = |selector: fn(&RGB32) -> u8| -> u8 {
        //     let (min, max) = colors
        //         .iter()
        //         .map(selector)
        //         .fold((u8::MAX, u8::MIN), |(min, max), val| {
        //             (u8::min(min, val), u8::max(max, val))
        //         });
        //     max - min
        // };
    
        // let red_range = range(|c| c.r);
        // let green_range = range(|c| c.g);
        // let blue_range = range(|c| c.b);

        let delta = {
            let low = Color::new(u8::MIN, u8::MIN, u8::MIN);
            let high = Color::new(u8::MAX, u8::MAX, u8::MAX);
            let (min, max) = colors.iter().fold((high, low), |(min, max), val| {
                (
                    Color::new(
                        u8::min(min.r, val.r),
                        u8::min(min.g, val.g),
                        u8::min(min.b, val.g),
                    ),
                    Color::new(
                        u8::max(max.r, val.r),
                        u8::max(max.g, val.g),
                        u8::max(max.b, val.g),
                    ),
                )
            });
            Color::new(max.r - min.r, max.g - min.g, max.b - min.b)
        };

        match delta.r.cmp(&delta.g).then(delta.g.cmp(&delta.b)) {
            Ordering::Greater => (Channel::Red, delta.r),
            Ordering::Less => (Channel::Blue, delta.b),
            Ordering::Equal => (Channel::Green, delta.g),
        }
    }

    pub fn average(colors: &[Color]) -> Color {
        let (r, g, b) = colors.iter().fold((0, 0, 0), |sum, val| {
            (sum.0 + val.r as u64, sum.1 + val.g as u64, sum.2 + val.b as u64)
        });
        let len = colors.len();
        Color::new(
            f32::round(r as f32 / len as f32) as u8,
            f32::round(g as f32 / len as f32) as u8,
            f32::round(b as f32 / len as f32) as u8,
        )
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}) #{:X}{:X}{:X}",
            self.r, self.g, self.b, self.r, self.g, self.b
        )
    }
}

fn median_cut(colors: Vec<Color>, palette_size: usize) -> Vec<Color> {
    if palette_size >= colors.len() {
        return colors;
    }

    let mut colors = colors;

    let mut buckets: Vec<usize> = Vec::new();
    buckets.push(0);
    buckets.push(colors.len());

    while buckets.len() < palette_size {
        let v: Vec<(Channel, u8)> = buckets
            .iter()
            .zip(buckets.iter().skip(1))
            .map(|(&a, &b)| Color::max_channel(&colors[a..b]))
            .collect();

        let max = v
            .iter()
            .enumerate()
            .max_by(|x, y| x.1 .1.cmp(&y.1 .1))
            .unwrap();

        let start = buckets[max.0];
        let end = buckets[max.0 + 1];
        let bucket = &mut colors[start..end];

        match max.1 .0 {
            Channel::Red => bucket.sort_by(|x, y| x.r.cmp(&y.r)),
            Channel::Green => bucket.sort_by(|x, y| x.g.cmp(&y.g)),
            Channel::Blue => bucket.sort_by(|x, y| x.b.cmp(&y.b)),
        };

        let mid = start + bucket.len() / 2;
        buckets.insert(max.0 + 1, mid);
    }

    buckets
        .iter()
        .zip(buckets.iter().skip(1))
        .map(|(&a, &b)| Color::average(&colors[a..b]))
        .collect()
}

fn img_to_colors<P: AsRef<Path>>(path: P) -> Result<Vec<Color>, image::ImageError> {
    let img = image::open(path)?;
    let img = img.to_rgb8();
    let colors = img
        .chunks_exact(3)
        .map(|ch| Color::new(ch[0], ch[1], ch[2]))
        .collect();
    Ok(colors)
}

fn main() {
    let args = Args::parse();

    let paths = args.path.into_iter().collect::<Vec<_>>();

    for path in paths {
        let colors = match img_to_colors(&path) {
            Ok(colors) => colors,
            Err(_) => {
                eprintln!("Invalid path or file: {}", path);
                return;
            }
        };

        let palette = median_cut(colors, args.palette_size);

        let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Always);

        writeln!(stdout, "{}", path).unwrap();
        for color in palette.iter() {
            let color_spec = termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Rgb(color.r, color.g, color.b))).clone();
            stdout.set_color(&color_spec).unwrap();
            writeln!(stdout, "{}", color).unwrap();
            stdout.reset().unwrap();
        }
    }
}
