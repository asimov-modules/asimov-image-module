// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-image-viewer requires the 'std' feature");

use asimov_image_module::core::{Error, Result as CoreResult, handle_error, warn_user_with_error};
use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use know::classes::Image as KnowImage;
use std::error::Error as StdError;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

/// asimov-image-viewer
#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Copy stdin to stdout (pass-through / tee)
    #[arg(short = 'U', long = "union")]
    union: bool,
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

    let exit_code = match run_viewer(&options) {
        Ok(()) => EX_OK,
        Err(err) => handle_error(&err, &options.flags),
    };

    Ok(exit_code)
}

fn run_viewer(opts: &Options) -> CoreResult<()> {
    let flags = &opts.flags;
    let union = opts.union;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_image_module::viewer",
        union = union,
        "starting viewer"
    );

    let (tx, rx) = mpsc::channel::<KnowImage>();

    // Reader thread: stdin -> JSON lines -> KnowImage -> channel
    let debug = flags.debug;
    let verbose = flags.verbose;

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line_res in stdin.lock().lines() {
            match line_res {
                Ok(line) => {
                    if union {
                        let _ = writeln!(stdout, "{line}");
                        let _ = stdout.flush();
                    }
                    match serde_json::from_str::<KnowImage>(&line) {
                        Ok(img) => {
                            if tx.send(img).is_err() {
                                break;
                            }
                        },
                        Err(e) => {
                            if debug || verbose >= 1 {
                                eprintln!("WARN: failed to parse Image JSON-LD");
                            }
                            #[cfg(feature = "tracing")]
                            asimov_module::tracing::warn!(
                                target: "asimov_image_module::viewer",
                                error = %e,
                                "failed to parse Image JSON-LD"
                            );
                        },
                    }
                },
                Err(e) => {
                    if debug || verbose >= 1 {
                        eprintln!("WARN: stdin read error: {e}");
                    }
                    #[cfg(feature = "tracing")]
                    asimov_module::tracing::warn!(
                        target: "asimov_image_module::viewer",
                        error = %e,
                        "stdin read error"
                    );
                    break;
                },
            }
        }
    });

    run_ui(rx, flags)?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_image_module::viewer",
        "viewer exiting"
    );

    Ok(())
}

fn run_ui(rx: Receiver<KnowImage>, flags: &StandardOptions) -> CoreResult<()> {
    use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

    let mut width: usize = 320;
    let mut height: usize = 240;
    let mut buffer: Vec<u32> = vec![0; width * height];

    let mut window = Window::new(
        "ASIMOV",
        width,
        height,
        WindowOptions {
            resize: true,
            scale: Scale::X1,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: false,
            borderless: false,
            transparency: false,
            ..WindowOptions::default()
        },
    )
    .map_err(|e| Error::Other(e.to_string()))?;

    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut latest: Option<KnowImage> = None;
        while let Ok(img) = rx.try_recv() {
            latest = Some(img);
        }

        if let Some(img) = latest {
            if let Err(e) = show_image(&mut window, &mut buffer, &mut width, &mut height, img) {
                warn_user_with_error(flags, "failed to display image", &e);
            }
        } else {
            window
                .update_with_buffer(&buffer, width, height)
                .map_err(|e| Error::Other(e.to_string()))?;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn show_image(
    window: &mut minifb::Window,
    buffer: &mut Vec<u32>,
    width: &mut usize,
    height: &mut usize,
    img: KnowImage,
) -> CoreResult<()> {
    let w = img
        .width
        .ok_or_else(|| Error::InvalidDimensions("missing image.width".into()))?;
    let h = img
        .height
        .ok_or_else(|| Error::InvalidDimensions("missing image.height".into()))?;

    let data = img.data;
    let expected = w
        .checked_mul(h)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| Error::InvalidBuffer("width*height*3 overflow".into()))?;
    if data.len() != expected {
        return Err(Error::InvalidBuffer(format!(
            "byte length {} does not match width*height*3 ({expected})",
            data.len()
        )));
    }

    if *width != w || *height != h || buffer.len() != w * h {
        *width = w;
        *height = h;
        *buffer = vec![0; w * h];
    }

    for (i, chunk) in data.chunks_exact(3).enumerate() {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        buffer[i] = (r << 16) | (g << 8) | b;
    }

    window.set_title(&format!(
        "{} ({}x{})",
        img.id.unwrap_or_else(|| "ASIMOV".to_string()),
        w,
        h
    ));
    window
        .update_with_buffer(buffer, *width, *height)
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok(())
}
