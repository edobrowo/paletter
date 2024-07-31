use clap::Parser;
use image;
use termcolor::{self, WriteColor};
use std::{fmt, io};
use std::io::Write;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The number of colors that the image should be quantized into.
    #[clap(required = true, num_args = 1)]
    palette_size: usize,

    /// List of image file paths. A palette will be generated for each image.
    #[arg(required = true, num_args = 1..)]
    path: Vec<String>,

    /// Display the colors in hexadecimal.
    #[clap(long)]
    hex: bool,

    /// Display the colors in decimal.
    #[clap(long, short)]
    decimal: bool,
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

    pub fn max_channel(colors: &[Self]) -> (Channel, u8) {
        use std::cmp::Ordering;

        // let range = |selector: fn(&Self) -> u8| -> u8 {
        //     let (min, max) = colors
        //         .iter()
        //         .map(selector)
        //         .fold((u8::MAX, u8::MIN), |(min, max), val| {
        //             (u8::min(min, val), u8::max(max, val))
        //         });
        //     max - min
        // };
    
        // let r = range(|c| c.r);
        // let g = range(|c| c.g);
        // let b = range(|c| c.b);
        // match r.cmp(&g).then(g.cmp(&b)) {
        //     Ordering::Greater => (Channel::Red, r),
        //     Ordering::Less => (Channel::Blue, b),
        //     Ordering::Equal => (Channel::Green, g),
        // }

        let delta = {
            let low = Self::new(u8::MIN, u8::MIN, u8::MIN);
            let high = Self::new(u8::MAX, u8::MAX, u8::MAX);
            let (min, max) = colors.iter().fold((high, low), |(min, max), val| {
                (
                    Self::new(
                        u8::min(min.r, val.r),
                        u8::min(min.g, val.g),
                        u8::min(min.b, val.g),
                    ),
                    Self::new(
                        u8::max(max.r, val.r),
                        u8::max(max.g, val.g),
                        u8::max(max.b, val.g),
                    ),
                )
            });
            Self::new(max.r - min.r, max.g - min.g, max.b - min.b)
        };

        match delta.r.cmp(&delta.g).then(delta.g.cmp(&delta.b)) {
            Ordering::Greater => (Channel::Red, delta.r),
            Ordering::Less => (Channel::Blue, delta.b),
            Ordering::Equal => (Channel::Green, delta.g),
        }
    }

    pub fn average(colors: &[Self]) -> Self {
        let (r, g, b) = colors.iter().fold((0, 0, 0), |sum, val| {
            (sum.0 + val.r as u64, sum.1 + val.g as u64, sum.2 + val.b as u64)
        });
        let len = colors.len();
        Self::new(
            f32::round(r as f32 / len as f32) as u8,
            f32::round(g as f32 / len as f32) as u8,
            f32::round(b as f32 / len as f32) as u8,
        )
    }

    pub fn to_hex_string(&self) -> String {
        format!("#{:X}{:X}{:X}", self.r, self.g, self.b)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({},{},{})",
            self.r, self.g, self.b
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

fn main() -> io::Result<()> {
    let args = Args::parse();
    let paths = args.path.into_iter().collect::<Vec<_>>();

    let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Always);

    let mut bold_spec = termcolor::ColorSpec::new();
    bold_spec.set_bold(true);

    for (i, path) in paths.iter().enumerate() {

        stdout.set_color(&bold_spec)?;
        write!(stdout, "Image {}", i + 1)?;
        stdout.reset()?;
        write!(&mut stdout, ": {}\n", path)?;
    
        let colors = match img_to_colors(&path) {
            Ok(colors) => colors,
            Err(err) => {
                eprintln!("Invalid path or file: {}", path);
                return Err(io::Error::new(io::ErrorKind::Other, err));
            }
        };

        let palette = median_cut(colors, args.palette_size);

        for color in palette.iter() {
            let color_spec = termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Rgb(color.r, color.g, color.b))).clone();
            stdout.set_color(&color_spec)?;

            if args.decimal || !args.hex {
                write!(stdout, "{}", color)?;
                if args.hex {
                    write!(stdout, " ")?;
                }
            }
            if args.hex {
                write!(stdout, "{}", color.to_hex_string())?;
            }
            write!(stdout, "\n")?;

            stdout.reset()?;
        }
    }

    Ok(())
}
