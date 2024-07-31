use clap::Parser;
use paletter::quantize;
use std::error::Error;
use std::io::Write;
use termcolor::{self, WriteColor};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Number of colors in the palette.
    #[clap(required = true, long, short)]
    palette_size: usize,

    /// List of image file paths. A palette will be generated for each image.
    #[arg(required = true, num_args = 1.., long, short)]
    files: Vec<String>,

    /// Display the colors in hexadecimal.
    #[clap(long)]
    hex: bool,

    /// Display the colors in RGB24.
    #[clap(long, short)]
    rgb: bool,

    /// Display colors without any color styling.
    #[clap(long, short)]
    uncolored: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let paths = args.files.into_iter().collect::<Vec<_>>();

    let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Always);

    let mut bold_spec = termcolor::ColorSpec::new();
    bold_spec.set_bold(true);

    let mut stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Always);

    let mut err_spec = termcolor::ColorSpec::new();
    err_spec.set_fg(Some(termcolor::Color::Red));

    for (i, path) in paths.iter().enumerate() {
        let colors = match paletter::img_to_colors(path) {
            Ok(colors) => colors,
            Err(_) => {
                stderr.set_color(&err_spec)?;
                writeln!(stderr, "Invalid path: {}", &path)?;
                continue;
            }
        };

        stdout.set_color(&bold_spec)?;
        write!(stdout, "Image {}", i + 1)?;

        stdout.reset()?;
        writeln!(&mut stdout, ": {}", path)?;

        let palette = quantize::median_cut(colors, args.palette_size);

        let rgb = args.rgb || !args.hex;
        let hex = args.hex;
        let colored = !args.uncolored;

        for color in palette {
            if colored {
                let mut color_spec = termcolor::ColorSpec::new();
                color_spec.set_fg(Some(termcolor::Color::Rgb(color.r, color.g, color.b)));

                stdout.set_color(&color_spec)?;
            }

            if rgb {
                write!(stdout, "{color}")?;
                hex.then(|| write!(stdout, " "));
            }
            if hex {
                write!(stdout, "{}", color.to_hex_string())?;
            }
            writeln!(stdout)?;

            stdout.reset()?;
        }

        writeln!(stdout)?;
    }

    Ok(())
}
