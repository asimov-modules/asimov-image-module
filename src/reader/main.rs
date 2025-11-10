use std::error::Error;
use clap::Parser;
use image::GenericImageView;
use know::traits::ToJsonLd;
use std::io::Read;
use std::path::PathBuf;
use asimov_module::SysexitsError::{self, *};
use clientele::StandardOptions;

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

    let (image_data, abs_path) = read_input_bytes(options.url)?;

    let mut img = image::load_from_memory(&image_data)?;
    let (src_width, src_height) = img.dimensions();

    // Resize if target dimensions specified and differ from source
    if let Some((target_width, target_height)) = options.size {
        if target_width != src_width || target_height != src_height {
            img = img.resize_exact(
                target_width,
                target_height,
                image::imageops::FilterType::Lanczos3,
            );
        }
    }

    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let raw_data = rgb_img.into_raw();

    let file_url = format!("file:{}", abs_path);
    let image = know::classes::Image {
        id: Some(file_url.clone()),
        width: Some(width as _),
        height: Some(height as _),
        data: raw_data,
        source: Some(file_url),
    };

    let jsonld = image.to_jsonld()?;
    println!("{}", jsonld.to_string());

    Ok(EX_OK)
}

/// Read input from a file path (optionally prefixed by file:/file://) or from stdin.
/// Returns (bytes, canonical_file_url).
fn read_input_bytes(url: Option<String>) -> Result<(Vec<u8>, String), Box<dyn Error>> {
    if let Some(url) = &url {
        let input_path = {
            let p = url;
            let p = p.strip_prefix("file://").unwrap_or(p);
            let p = p.strip_prefix("file:").unwrap_or(p);
            p
        };
        let canonical = PathBuf::from(input_path)
            .canonicalize()?
            .to_string_lossy()
            .to_string();
        let data = std::fs::read(input_path)?;
        Ok((data, canonical))
    } else {
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data)?;
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
        return Err(format!("Width {width} is out of reasonable range (160-7680)"));
    }
    if !(120..=4320).contains(&height) {
        return Err(format!("Height {height} is out of reasonable range (120-4320)"));
    }

    Ok((width, height))
}
