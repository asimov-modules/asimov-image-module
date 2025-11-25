// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-image-writer requires the 'std' feature");

use asimov_image_module::core::{
    Error, Result as CoreResult, handle_error, info_user, warn_user_with_error,
};
use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use know::classes::Image as KnowImage;
use std::error::Error as StdError;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

/// asimov-image-writer
#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Copy stdin to stdout (pass-through / tee)
    #[arg(short = 'U', long)]
    union: bool,

    /// Output file(s). Each incoming image is saved to all of these paths.
    /// Format is inferred from the file extension (e.g., .png, .jpg, .bmp).
    #[arg(value_name = "FILES")]
    files: Vec<PathBuf>,
}

pub fn main() -> Result<SysexitsError, Box<dyn StdError>> {
    // Load environment variables from `.env`:
    asimov_module::dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = asimov_module::args_os()?;

    // Parse command-line options:
    let options = Options::parse_from(args);

    // Handle the `--version` flag:
    if options.flags.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(EX_OK);
    }

    // Handle the `--license` flag:
    if options.flags.license {
        print!("{}", include_str!("../../UNLICENSE"));
        return Ok(EX_OK);
    }

    // Configure logging & tracing:
    #[cfg(feature = "tracing")]
    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

    let exit_code = match run_writer(&options) {
        Ok(()) => EX_OK,
        Err(err) => handle_error(&err, &options.flags),
    };

    Ok(exit_code)
}

fn run_writer(opts: &Options) -> CoreResult<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let flags = &opts.flags;
    let union = opts.union;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_image_module::writer",
        union = union,
        outputs = ?opts.files,
        "starting writer"
    );

    if opts.files.is_empty() {
        info_user(flags, "no output FILES provided; images will not be saved");
    }

    for line_res in stdin.lock().lines() {
        match line_res {
            Ok(line) => {
                if union {
                    let _ = writeln!(stdout, "{line}");
                    let _ = stdout.flush();
                }

                let parsed: KnowImage = match serde_json::from_str(&line) {
                    Ok(img) => img,
                    Err(e) => {
                        warn_user_with_error(flags, "failed to parse Image JSON-LD", &e);
                        continue;
                    },
                };

                if let Err(e) = save_image_to_all(&parsed, &opts.files) {
                    warn_user_with_error(flags, "failed to save image", &e);
                }
            },
            Err(e) => {
                warn_user_with_error(flags, "stdin read error", &e);
                break;
            },
        }
    }

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_image_module::writer",
        "writer exiting"
    );

    Ok(())
}

fn save_image_to_all(img: &KnowImage, outputs: &[PathBuf]) -> CoreResult<()> {
    let w = img
        .width
        .ok_or_else(|| Error::InvalidDimensions("missing image.width".into()))?
        as usize;
    let h = img
        .height
        .ok_or_else(|| Error::InvalidDimensions("missing image.height".into()))?
        as usize;

    let expected = w
        .checked_mul(h)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| Error::InvalidBuffer("width*height*3 overflow".into()))?;

    if img.data.len() != expected {
        return Err(Error::InvalidBuffer(format!(
            "byte length {} does not match width*height*3 ({expected})",
            img.data.len()
        )));
    }

    let rgb_img = image::RgbImage::from_raw(w as u32, h as u32, img.data.clone())
        .ok_or_else(|| Error::InvalidBuffer("failed to construct RgbImage from raw data".into()))?;

    let dyn_img = image::DynamicImage::ImageRgb8(rgb_img);

    for path in outputs {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| Error::Io {
                    context: "creating parent directory",
                    source: e,
                })?;
            }
        }

        dyn_img
            .save(path)
            .map_err(|e| Error::Other(format!("saving to '{}' failed: {e}", path.display())))?;
    }

    Ok(())
}
