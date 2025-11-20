# ASIMOV Image Module

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Package on Crates.io](https://img.shields.io/crates/v/asimov-image-module)](https://crates.io/crates/asimov-image-module)
[![Documentation](https://docs.rs/asimov-image-module/badge.svg)](https://docs.rs/asimov-image-module)

[ASIMOV] module for image processing — provides tools to read, write, and view images with [JSON-LD] output for seamless knowledge integration.

## ✨ Features

- To be determined!

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

## 👉 Examples

Read a file and output a JSON-LD object:
```bash
asimov-image-reader ./photo.jpg
```

Resize the image before emitting:
```bash
asimov-image-reader ./photo.jpg --size 800x600
```

Read from stdin:
```bash
cat photo.jpg | asimov-image-reader
```

Pipe the reader into the viewer (Linux/macOS):
```bash
# Show a single image
asimov-image-reader ./photo.jpg | asimov-image-viewer

# Resize before viewing
asimov-image-reader ./photo.jpg --size 800x600 | asimov-image-viewer
```

Pipe on Windows (PowerShell):
```powershell
asimov-image-reader .\photo.jpg | asimov-image-viewer
```

Tee the stream (debug pipelines) while viewing:
```bash
asimov-image-reader ./photo.jpg | asimov-image-viewer --union | jq .
```

View a sequence (any producer that emits one Image JSON per line will work):
```bash
# Example: loop multiple files through the reader into the viewer
for f in imgs/*.jpg; do asimov-image-reader "$f"; done | asimov-image-viewer
```

> Notes
> - The viewer auto-updates on each incoming frame and resizes the framebuffer to match the image.
> - The viewer expects RGB byte data (R, G, B per pixel) packed in data and width/height set.

## ⚙ Configuration

This module requires no configuration.

## 📚 Reference

### Installed Binaries

- `asimov-image-reader` — reads and emits image metadata as JSON-LD
- `asimov-image-viewer` — displays image JSON frames in a window

### `asimov-image-reader`

```
Usage: asimov-image-reader [OPTIONS] [URL]

Options:
  -s, --size <WxH>  Resize to specific width and height (e.g., 1920x1080)
      --license     Show license information
  -v, --verbose     Enable verbose output
  -V, --version     Print version information
  -h, --help        Print help
```

### `asimov-image-viewer`

```
Usage: asimov-image-viewer [OPTIONS]

Options:
-U, --union Copy stdin to stdout (tee)
--license Show license information
-v, --verbose Enable verbose output
-V, --version Print version information
-h, --help Print help
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
