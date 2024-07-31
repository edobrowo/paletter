use clap::Parser;
use image;
use std::io::Write;
use std::path::Path;
use std::{fmt, io};
use termcolor::{self, WriteColor};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            (
                sum.0 + val.r as u64,
                sum.1 + val.g as u64,
                sum.2 + val.b as u64,
            )
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
        write!(f, "({},{},{})", self.r, self.g, self.b)
    }
}

#[derive(Clone, Debug)]
struct Bucket {
    pub index: usize,
    pub chan: Channel,
    pub delta: u8,
}

impl Bucket {
    pub fn new(index: usize, chan: Channel, delta: u8) -> Self {
        Self { index, chan, delta }
    }
}

fn median_cut(colors: Vec<Color>, palette_size: usize) -> Vec<Color> {
    if palette_size >= colors.len() {
        return colors;
    }

    let mut colors = colors;
    let mut buckets: Vec<Bucket> = Vec::new();

    let (chan, delta) = Color::max_channel(&colors);
    buckets.push(Bucket::new(0, chan, delta));
    buckets.push(Bucket::new(colors.len(), chan, 0));

    while buckets.len() < palette_size {
        let (i, max_bucket) = buckets
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.delta.cmp(&y.delta))
            .unwrap();

        let start = buckets[i].index;
        let end = buckets[i + 1].index;
        let mid = start + (end - start) / 2;

        let bucket_colors = &mut colors[start..end];

        match max_bucket.chan {
            Channel::Red => bucket_colors.sort_by(|x, y| x.r.cmp(&y.r)),
            Channel::Green => bucket_colors.sort_by(|x, y| x.g.cmp(&y.g)),
            Channel::Blue => bucket_colors.sort_by(|x, y| x.b.cmp(&y.b)),
        };

        let (chan0, delta0) = Color::max_channel(&colors[start..mid]);
        let (chan1, delta1) = Color::max_channel(&colors[mid..end]);

        buckets[i] = Bucket::new(start, chan0, delta0);
        buckets.insert(i + 1, Bucket::new(mid, chan1, delta1));
    }

    buckets
        .iter()
        .zip(buckets.iter().skip(1))
        .map(|(a, b)| Color::average(&colors[a.index..b.index]))
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
            let color_spec = termcolor::ColorSpec::new()
                .set_fg(Some(termcolor::Color::Rgb(color.r, color.g, color.b)))
                .clone();
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
            writeln!(stdout, "")?;

            stdout.reset()?;
        }

        writeln!(stdout, "")?;
    }

    Ok(())
}
