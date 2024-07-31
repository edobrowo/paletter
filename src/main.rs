use clap::Parser;
use std::io::{self, Write};
use termcolor::{self, WriteColor};
use paletter::{self, quantize};

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
        writeln!(&mut stdout, ": {}", path)?;

        let colors = match paletter::img_to_colors(path) {
            Ok(colors) => colors,
            Err(err) => {
                eprintln!("Invalid path or file: {}", path);
                return Err(io::Error::new(io::ErrorKind::Other, err));
            }
        };

        let palette = quantize::median_cut(colors, args.palette_size);

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
            writeln!(stdout)?;

            stdout.reset()?;
        }

        writeln!(stdout)?;
    }

    Ok(())
}
