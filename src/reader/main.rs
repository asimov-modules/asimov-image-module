// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-image-reader requires the 'std' feature");

use asimov_image_module::core::{Error, Result as CoreResult, handle_error};
use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use image::GenericImageView;
use know::traits::ToJsonLd;
use std::error::Error as StdError;
use std::io::Read;
use std::path::PathBuf;

/// asimov-image-reader
#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Input image file path.
    /// If not specified, reads from stdin
    url: Option<String>,

    /// Desired output dimensions in WxH format (e.g., 1920x1080)
    /// If not specified, uses the input file's native dimensions
    #[arg(short = 's', long = "size", value_parser = parse_dimensions)]
    size: Option<(u32, u32)>,
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

    let exit_code = match run_reader(&options) {
        Ok(()) => EX_OK,
        Err(err) => handle_error(&err, &options.flags),
    };

    Ok(exit_code)
}

fn run_reader(opts: &Options) -> CoreResult<()> {
    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_image_module::reader",
        url = ?opts.url,
        size = ?opts.size,
        "starting reader"
    );

    let (image_data, abs_path) = read_input_bytes(&opts.url)?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::debug!(
        target: "asimov_image_module::reader",
        path = %abs_path,
        bytes = image_data.len(),
        "read input image bytes"
    );

    let mut img = image::load_from_memory(&image_data)?;
    let (src_w, src_h) = img.dimensions();

    #[cfg(feature = "tracing")]
    asimov_module::tracing::debug!(
        target: "asimov_image_module::reader",
        width = src_w,
        height = src_h,
        "decoded image"
    );

    if let Some((target_w, target_h)) = opts.size {
        if target_w != src_w || target_h != src_h {
            #[cfg(feature = "tracing")]
            asimov_module::tracing::debug!(
                target: "asimov_image_module::reader",
                target_width = target_w,
                target_height = target_h,
                "resizing image"
            );

            img = img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3);
        }
    }

    let rgb_img = img.to_rgb8();
    let (w, h) = rgb_img.dimensions();
    let raw_data = rgb_img.into_raw();

    let file_url = format!("file:{abs_path}");
    let image = know::classes::Image {
        id: Some(file_url.clone()),
        width: Some(w as _),
        height: Some(h as _),
        data: raw_data,
        source: Some(file_url),
    };

    let jsonld = image
        .to_jsonld()
        .map_err(|e| Error::JsonLd(e.to_string()))?;

    println!("{jsonld}");

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_image_module::reader",
        width = w,
        height = h,
        "finished reader"
    );

    Ok(())
}

/// Read input from a file path (optionally prefixed by file:/file://) or from stdin.
/// Returns (bytes, canonical_file_url).
fn read_input_bytes(url: &Option<String>) -> CoreResult<(Vec<u8>, String)> {
    if let Some(url) = url {
        let input_path = {
            let p = url;
            let p = p.strip_prefix("file://").unwrap_or(p);
            let p = p.strip_prefix("file:").unwrap_or(p);
            p
        };

        let canonical = PathBuf::from(input_path)
            .canonicalize()
            .map_err(|e| Error::Io {
                context: "resolving input path",
                source: e,
            })?;

        let data = std::fs::read(input_path).map_err(|e| Error::Io {
            context: "reading input file",
            source: e,
        })?;

        Ok((data, canonical.to_string_lossy().to_string()))
    } else {
        let mut data = Vec::new();
        std::io::stdin()
            .read_to_end(&mut data)
            .map_err(|e| Error::Io {
                context: "reading from stdin",
                source: e,
            })?;
        Ok((data, "[stdin]".to_string()))
    }
}

/// Accepts "1920x1080", "1920×1080", with optional spaces. Validates reasonable ranges.
fn parse_dimensions(s: &str) -> Result<(u32, u32), String> {
    let s = s.trim().replace('×', "x");
    let parts: Vec<&str> = s.split('x').map(|t| t.trim()).collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(format!("Invalid format '{s}'. Use WxH (e.g., 1920x1080)"));
    }

    let width: u32 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid width: {}", parts[0]))?;
    let height: u32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid height: {}", parts[1]))?;

    if !(160..=7680).contains(&width) {
        return Err(format!(
            "Width {width} is out of reasonable range (160-7680)"
        ));
    }
    if !(120..=4320).contains(&height) {
        return Err(format!(
            "Height {height} is out of reasonable range (120-4320)"
        ));
    }

    Ok((width, height))
}
