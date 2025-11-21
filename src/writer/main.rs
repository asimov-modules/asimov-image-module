// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-image-viewer requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use know::classes::Image as KnowImage;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
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

pub fn main() -> Result<SysexitsError, Box<dyn Error>> {
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

    run_writer(options)?;
    Ok(EX_OK)
}

fn run_writer(opts: Options) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let union = opts.union;
    let debug = opts.flags.debug;
    let verbose = opts.flags.verbose != 0;

    if opts.files.is_empty() && (debug || verbose) {
        let _ = writeln!(
            stderr,
            "INFO: no output FILES provided; images will not be saved"
        );
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
                        if debug || verbose {
                            let _ = writeln!(stderr, "WARN: failed to parse Image JSON-LD: {e}");
                        }
                        continue;
                    },
                };

                if let Err(e) = save_image_to_all(&parsed, &opts.files) {
                    let _ = writeln!(stderr, "WARN: failed to save image: {e}");
                }
            },
            Err(e) => {
                if debug || verbose {
                    let _ = writeln!(stderr, "ERROR: stdin read error: {e}");
                }
                break;
            },
        }
    }

    if debug {
        let _ = writeln!(stderr, "INFO: writer exiting");
    }
    Ok(())
}

fn save_image_to_all(img: &KnowImage, outputs: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let w = img.width.ok_or_else(|| err_msg("missing image.width"))? as usize;
    let h = img.height.ok_or_else(|| err_msg("missing image.height"))? as usize;

    let expected = w
        .checked_mul(h)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| err_msg("width*height*3 overflow"))?;

    if img.data.len() != expected {
        return Err(err_msg(format!(
            "byte length {} does not match width*height*3 ({expected})",
            img.data.len()
        )));
    }

    let rgb_img = image::RgbImage::from_raw(w as u32, h as u32, img.data.clone())
        .ok_or_else(|| err_msg("failed to construct RgbImage from raw data"))?;

    let dyn_img = image::DynamicImage::ImageRgb8(rgb_img);

    for path in outputs {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        dyn_img
            .save(path)
            .map_err(|e| err_msg(format!("saving to '{}' failed: {e}", display_path(path))))?;
    }

    Ok(())
}

fn display_path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn err_msg<M: Into<String>>(m: M) -> Box<dyn Error> {
    m.into().into()
}
