# ASIMOV Image Module

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Package on Crates.io](https://img.shields.io/crates/v/asimov-image-module)](https://crates.io/crates/asimov-image-module)
[![Documentation](https://docs.rs/asimov-image-module/badge.svg)](https://docs.rs/asimov-image-module)

Image processing utilities for [ASIMOV] — read, view, and write images with [JSON-LD] output for seamless knowledge integration.

## ✨ Features

- Read images and emit JSON-LD (know::Image)
- View streamed JSON-LD image frames in a window
- Save JSON-LD images into multiple formats (PNG, JPEG, BMP, etc.)
- Proper error handling (sysexits) and tracing
- Full support for pipelines and CLI workflows

## 🛠️ Prerequisites

- [Rust] 1.85+ (2024 edition) if building from source code

## ⬇️ Installation

### Installation with the [ASIMOV CLI]

```bash
asimov module install image -v
```

### Installation from Source Code

```bash
cargo install asimov-image-module
```

## 🚀 Examples

### 📥 Reading Images (JSON-LD emitter)

**Read a file**
```bash
asimov-image-reader ./photo.jpg
```

**Resize before emitting**
```bash
asimov-image-reader ./photo.jpg --size 800x600
```

**Read from stdin**
```bash
cat photo.jpg | asimov-image-reader
```

**Verbose error output**
```bash
asimov-image-reader -v /no/such/file.jpg
asimov-image-reader -vv /no/such/file.jpg
```
> Notes
> - Reader emits one JSON object per line.
> - Format is inferred from bytes, not extension.
> - Errors use POSIX sysexits for safe pipelines.
> - Use -v, -vv, -vvv, or --debug for more detail.

### 🖼️ Viewing Images

**View a single image**
```bash
asimov-image-reader ./photo.jpg | asimov-image-viewer
```

**Resize before viewing**
```bash
asimov-image-reader ./photo.jpg --size 800x600 | asimov-image-viewer
```

**PowerShell (Windows)**
```powershell
asimov-image-reader .\photo.jpg | asimov-image-viewer
```

**View a stream of images**
```bash
for f in imgs/*.jpg; do
  asimov-image-reader "$f"
done | asimov-image-viewer
```

**Tee JSON-LD while viewing (debugging pipelines)**
```bash
asimov-image-reader ./photo.jpg \
  | asimov-image-viewer --union \
  | jq .
```

> Notes
> - The viewer auto-resizes to each incoming frame.
> - Input must match know::Image shape (width, height, data).
> - Closes with ESC.

### 💾 Writing Images

**Save a single image**
```bash
asimov-image-reader ./photo.jpg | asimov-image-writer out/photo.png
```

**Resize then save**
```bash
asimov-image-reader ./photo.jpg --size 800x600 \
  | asimov-image-writer out/photo-800x600.jpg
```

**Save to multiple files**
```bash
asimov-image-reader ./photo.jpg \
  | asimov-image-writer out/img.png out/img.jpg out/img.bmp
```

**Tee JSON-LD while saving**
```bash
asimov-image-reader ./photo.jpg \
  | asimov-image-writer --union out/photo.png \
  | jq .
```

From stdin
```bash
cat photo.jpg | asimov-image-reader | asimov-image-writer out/photo.png
```

Windows (PowerShell)
```powershell
asimov-image-reader .\photo.jpg | asimov-image-writer .\out\photo.png
asimov-image-reader -s 640x360 .\photo.jpg | asimov-image-writer .\out\photo-640x360.jpg
```

> Notes
> - File format is inferred from extension.
> - Parent directories are created automatically.
> - Invalid image data produces structured errors.

## ⚙ Configuration

This module requires no configuration.

## 📚 Reference

### Installed Binaries

- `asimov-image-reader` — decodes images → emits JSON-LD
- `asimov-image-viewer` — displays streamed JSON-LD frames
- `asimov-image-writer` — saves JSON-LD frames to file(s)

### `asimov-image-viewer`
```
Usage: asimov-image-viewer [OPTIONS]

Options:
    -U, --union       Copy stdin to stdout (tee)
    -v, --verbose     Increase logging (repeatable)
        --debug       Enable debug output
        --license     Show license
    -V, --version     Show version
    -h, --help        Show help
```

### `asimov-image-reader`
```
Usage: asimov-image-reader [OPTIONS] [URL]

Options:
    -s, --size <WxH>  Resize image before emitting (e.g. 1920x1080)
    -v, --verbose     Increase logging
        --debug       Enable debug output
        --license     Show license
    -V, --version     Show version
    -h, --help        Show help
```

### `asimov-image-writer`
```
Usage: asimov-image-writer [OPTIONS] [FILES]...

Arguments:
  [FILES]...    Output files. Each image is written to all paths. Format is
                inferred from the extension (.png, .jpg, .bmp)

Options:
    -U, --union       Copy stdin to stdout (tee)
    -v, --verbose...  Increase logging (repeatable)
        --debug       Enable debug output
        --license     Show license
    -V, --version     Show version
    -h, --help        Show help
```

## 👨‍💻 Development

```bash
git clone https://github.com/asimov-modules/asimov-image-module.git
```

---

[![Share on X](https://img.shields.io/badge/share%20on-x-03A9F4?logo=x)](https://x.com/intent/post?url=https://github.com/asimov-modules/asimov-image-module&text=asimov-image-module)
[![Share on Reddit](https://img.shields.io/badge/share%20on-reddit-red?logo=reddit)](https://reddit.com/submit?url=https://github.com/asimov-modules/asimov-image-module&title=asimov-image-module)
[![Share on Hacker News](https://img.shields.io/badge/share%20on-hn-orange?logo=ycombinator)](https://news.ycombinator.com/submitlink?u=https://github.com/asimov-modules/asimov-image-module&t=asimov-image-module)
[![Share on Facebook](https://img.shields.io/badge/share%20on-fb-1976D2?logo=facebook)](https://www.facebook.com/sharer/sharer.php?u=https://github.com/asimov-modules/asimov-image-module)
[![Share on LinkedIn](https://img.shields.io/badge/share%20on-linkedin-3949AB?logo=linkedin)](https://www.linkedin.com/sharing/share-offsite/?url=https://github.com/asimov-modules/asimov-image-module)

[ASIMOV]: https://asimov.sh
[ASIMOV CLI]: https://cli.asimov.sh
[JSON-LD]: https://json-ld.org
[Rust]: https://rust-lang.org
