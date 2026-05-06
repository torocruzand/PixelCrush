# PixelCrush — Image Optimizer

PixelCrush is a modern, blazingly fast desktop image optimization tool built with Rust and Slint. It allows you to quickly compress, convert, and compare images while maintaining a premium, responsive UI.

## Features

- **Blazing Fast**: Powered by Rust and native encoding libraries (`mozjpeg`, `oxipng`, `webp`).
- **Modern UI**: Declarative, responsive UI powered by Slint.
- **Drag & Drop**: Easily add images by dragging them into the app.
- **Before/After Comparison**: Interactive slider to compare the original and compressed image side-by-side.
- **Format Conversion**: Convert between JPEG, PNG, and WebP effortlessly.
- **Batch Processing**: Compress and save all your images at once with one click.
- **Quality Control**: Individual quality sliders for fine-tuned control.

## Installation

### Pre-compiled Binaries (Recommended)
You can download the latest pre-compiled binaries for Windows, macOS, and Linux from the [Releases page](https://github.com/torocruzand/image-converter/releases).

### Build from Source

#### Prerequisites
- Rust toolchain (`cargo`)
- Slint dependencies (see [Slint's setup guide](https://slint.dev/docs/rust/slint/))

#### Building
1. Clone the repository:
   ```bash
   git clone https://github.com/torocruzand/image-converter.git
   cd image-converter
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```
3. Run the application:
   ```bash
   cargo run --release
   ```

## Usage

1. Open **PixelCrush**.
2. Click "Select Images" or drag and drop images into the window.
3. Adjust the quality slider or target format for each image as needed.
4. Click "Compress" on an individual image, or "⚡ Compress All" at the top to batch process.
5. Once compressed, click "↔ Compare" to inspect the visual quality vs the original.
6. Click the download icon (⬇) to save an individual image, or "⬇ Save All" to pick an output directory and save them all.

## Tech Stack

- **[Rust](https://www.rust-lang.org/)**: Core logic, threading, and performance.
- **[Slint](https://slint.dev/)**: Next-generation GUI toolkit for the desktop UI.
- **[image](https://crates.io/crates/image)**: Image decoding and thumbnail generation.
- **[mozjpeg](https://crates.io/crates/mozjpeg)**: High-quality JPEG encoder.
- **[oxipng](https://crates.io/crates/oxipng)**: Lossless PNG compression.
- **[webp](https://crates.io/crates/webp)**: Google's WebP encoder.
- **[rfd](https://crates.io/crates/rfd)**: Native file dialogs for opening and saving.

## License
MIT License
