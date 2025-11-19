// src/viewer/main.rs
// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-image-viewer requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use know::classes::Image as KnowImage;
use minifb::{Key, Window, WindowOptions};
use serde_json::de::Deserializer;
use std::error::Error;
use std::io::{self, Read, Write};

/// asimov-image-viewer
///
/// Reads JSON(-LD) encoded `know::classes::Image` objects from stdin,
/// optionally passes the input through to stdout (`--union`), and
/// displays the images in a window. Press ESC or close the window to exit.
#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Copy stdin to stdout (pass-through / tee)
    #[clap(short = 'U', long = "union")]
    union: bool,
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
    asimov_module::init_tracing_subscriber(&options.flags)
        .expect("failed to initialize logging");

    // Read all of stdin into a buffer (works with pretty-printed JSON).
    let mut stdin_buf = Vec::<u8>::new();
    io::stdin().read_to_end(&mut stdin_buf)?;

    // Optional pass-through (union) – write the same bytes to stdout.
    if options.union {
        let mut stdout = io::stdout();
        stdout.write_all(&stdin_buf)?;
        stdout.flush()?;
    }

    // If there's no input at all, just exit quietly.
    if stdin_buf.is_empty() {
        return Ok(EX_OK);
    }

    // Prepare a streaming JSON deserializer over the whole buffer.
    let stream = Deserializer::from_slice(&stdin_buf).into_iter::<KnowImage>();

    // Window + framebuffer will be created on the first valid image.
    let mut window: Option<Window> = None;
    let mut fb: Vec<u32> = Vec::new();
    let mut fb_width: usize = 0;
    let mut fb_height: usize = 0;
    let mut saw_any_image = false;

    for img_result in stream {
        let image = match img_result {
            Ok(img) => img,
            Err(err) => {
                // Not an image / malformed / different JSON value – skip and continue.
                eprintln!("asimov-image-viewer: failed to parse Image from JSON: {err}");
                continue;
            }
        };

        let width = match image.width {
            Some(w) if w > 0 => w as usize,
            _ => {
                eprintln!(
                    "asimov-image-viewer: image missing or invalid width, skipping frame"
                );
                continue;
            }
        };
        let height = match image.height {
            Some(h) if h > 0 => h as usize,
            _ => {
                eprintln!(
                    "asimov-image-viewer: image missing or invalid height, skipping frame"
                );
                continue;
            }
        };

        let data = &image.data;
        let expected_len = width
            .saturating_mul(height)
            .saturating_mul(3); // RGB (3 bytes per pixel)

        if data.len() != expected_len {
            eprintln!(
                "asimov-image-viewer: image data len={} does not match {}x{} RGB ({expected_len}), skipping frame",
                data.len(),
                width,
                height
            );
            continue;
        }

        // (Re)create window + framebuffer if needed or if dimensions changed.
        let recreate = match &window {
            None => true,
            Some(_) if width != fb_width || height != fb_height => true,
            Some(_) => false,
        };

        if recreate {
            let mut win = Window::new(
                "ASIMOV",
                width,
                height,
                WindowOptions::default(),
            )
                .map_err(|e| format!("failed to create window: {e}"))?;

            win.set_target_fps(60);

            window = Some(win);
            fb_width = width;
            fb_height = height;
            fb = vec![0; fb_width * fb_height];
        }

        // Convert packed RGB bytes -> 0x00RRGGBB pixels for minifb.
        if fb.len() != fb_width * fb_height {
            fb.resize(fb_width * fb_height, 0);
        }

        for (i, px) in data.chunks_exact(3).enumerate() {
            let r = px[0] as u32;
            let g = px[1] as u32;
            let b = px[2] as u32;
            fb[i] = (r << 16) | (g << 8) | b;
        }

        if let Some(win) = window.as_mut() {
            // Update title from id/source if present.
            if let Some(title_id) = image
                .id
                .as_deref()
                .or_else(|| image.source.as_deref())
            {
                let title = format!("{title_id} ({}x{})", fb_width, fb_height);
                win.set_title(&title);
            }

            win.update_with_buffer(&fb, fb_width, fb_height)
                .map_err(|e| format!("failed to update window buffer: {e}"))?;

            saw_any_image = true;

            // ESC or closing the window terminates early.
            if !win.is_open() || win.is_key_down(Key::Escape) {
                break;
            }
        } else {
            // Somehow lost the window; nothing to show.
            break;
        }
    }

    if saw_any_image {
        // Keep window open until user closes it or presses ESC.
        if let Some(mut win) = window {
            while win.is_open() && !win.is_key_down(Key::Escape) {
                // Just redraw the last frame at ~60 FPS:
                win.update_with_buffer(&fb, fb_width, fb_height)
                    .map_err(|e| format!("failed to update window buffer: {e}"))?;
            }
        }
    }

    Ok(EX_OK)
}
