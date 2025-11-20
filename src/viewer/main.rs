// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-image-viewer requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use know::classes::Image as KnowImage;
use std::error::Error;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Copy stdin to stdout (pass-through / tee)
    #[arg(short = 'U', long = "union")]
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
    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

    let (tx, rx) = mpsc::channel::<KnowImage>();

    let union = options.union;
    let debug = options.flags.debug;
    let verbose = options.flags.verbose != 0;

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut stderr = io::stderr();

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
                            if debug || verbose {
                                let _ =
                                    writeln!(stderr, "WARN: failed to parse Image JSON-LD: {e}");
                            }
                        },
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
    });

    run_ui(rx, debug, verbose)?;
    Ok(EX_OK)
}

fn run_ui(rx: Receiver<KnowImage>, debug: bool, verbose: bool) -> Result<(), Box<dyn Error>> {
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
    )?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut latest: Option<KnowImage> = None;
        while let Ok(img) = rx.try_recv() {
            latest = Some(img);
        }

        if let Some(img) = latest {
            if let Err(e) = show_image(&mut window, &mut buffer, &mut width, &mut height, img) {
                if debug || verbose {
                    eprintln!("WARN: failed to display image: {e}");
                }
            }
        } else {
            window.update_with_buffer(&buffer, width, height)?;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    if debug {
        eprintln!("INFO: viewer exiting");
    }
    Ok(())
}

fn show_image(
    window: &mut minifb::Window,
    buffer: &mut Vec<u32>,
    width: &mut usize,
    height: &mut usize,
    img: KnowImage,
) -> Result<(), Box<dyn Error>> {
    let w = img.width.ok_or_else(|| err_msg("missing image.width"))? as usize;
    let h = img.height.ok_or_else(|| err_msg("missing image.height"))? as usize;

    let data = img.data;
    let expected = w
        .checked_mul(h)
        .and_then(|px| px.checked_mul(3))
        .ok_or_else(|| err_msg("width*height*3 overflow"))?;
    if data.len() != expected {
        return Err(err_msg(format!(
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
    window.update_with_buffer(buffer, *width, *height)?;
    Ok(())
}

fn err_msg<M: Into<String>>(m: M) -> Box<dyn Error> {
    m.into().into()
}
