use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat, imageops::FilterType};
use std::io::Cursor;
use std::path::Path;

// ──────────────────────────────────────────────────────────────
// Output format
// ──────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Jpeg,
    Png,
    WebP,
    Gif,
    Bmp,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "jpg" | "jpeg" => Self::Jpeg,
            "png" => Self::Png,
            "webp" => Self::WebP,
            "gif" => Self::Gif,
            "bmp" => Self::Bmp,
            _ => Self::Jpeg,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Jpeg => "JPEG",
            Self::Png => "PNG",
            Self::WebP => "WebP",
            Self::Gif => "GIF",
            Self::Bmp => "BMP",
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Compression options
// ──────────────────────────────────────────────────────────────
pub struct CompressionOptions {
    /// 1–100
    pub quality: u8,
    pub target_format: OutputFormat,
    /// Resize if any dimension exceeds this value (None = no resize)
    pub max_dimension: Option<u32>,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            quality: 80,
            target_format: OutputFormat::Jpeg,
            max_dimension: None,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Result
// ──────────────────────────────────────────────────────────────
#[allow(dead_code)]
pub struct ProcessedImage {
    pub data: Vec<u8>,
    pub format: OutputFormat,
    pub width: u32,
    pub height: u32,
}

impl ProcessedImage {
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

// ──────────────────────────────────────────────────────────────
// Processor
// ──────────────────────────────────────────────────────────────
pub struct ImageProcessor;

impl ImageProcessor {
    /// Compress/convert a file on disk.
    pub fn process(input_path: &Path, options: &CompressionOptions) -> Result<ProcessedImage> {
        let img = image::open(input_path)
            .with_context(|| format!("Cannot open image: {}", input_path.display()))?;

        Self::process_image(img, options)
    }

    /// Compress/convert an already-loaded `DynamicImage`.
    pub fn process_image(
        img: DynamicImage,
        options: &CompressionOptions,
    ) -> Result<ProcessedImage> {
        // Optional resize
        let img = match options.max_dimension {
            Some(max) if img.width() > max || img.height() > max => {
                img.resize(max, max, FilterType::Lanczos3)
            }
            _ => img,
        };

        let (w, h) = (img.width(), img.height());

        let data = match options.target_format {
            OutputFormat::Jpeg => Self::encode_jpeg(&img, options.quality)?,
            OutputFormat::Png => Self::encode_png(&img, options.quality)?,
            OutputFormat::WebP => Self::encode_webp(&img, options.quality)?,
            OutputFormat::Gif => Self::encode_fallback(&img, ImageFormat::Gif)?,
            OutputFormat::Bmp => Self::encode_fallback(&img, ImageFormat::Bmp)?,
        };

        Ok(ProcessedImage {
            data,
            format: options.target_format,
            width: w,
            height: h,
        })
    }

    // ── JPEG via mozjpeg ─────────────────────────────────────
    fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let rgb = img.to_rgb8();
        let (w, h) = rgb.dimensions();

        let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        comp.set_size(w as usize, h as usize);
        comp.set_quality(quality as f32);

        let raw: Vec<u8> = rgb.into_raw();
        let row_len = w as usize * 3;

        let mut started = comp
            .start_compress(Vec::new())
            .map_err(|e| anyhow::anyhow!("mozjpeg start: {e}"))?;

        for row_idx in 0..h as usize {
            let start = row_idx * row_len;
            started
                .write_scanlines(&raw[start..start + row_len])
                .map_err(|e| anyhow::anyhow!("mozjpeg scanline: {e}"))?;
        }

        started
            .finish()
            .map_err(|e| anyhow::anyhow!("mozjpeg finish: {e}"))
    }

    // ── PNG via oxipng ───────────────────────────────────────
    fn encode_png(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let raw = oxipng::RawImage::new(
            w,
            h,
            oxipng::ColorType::RGBA,
            oxipng::BitDepth::Eight,
            rgba.into_raw(),
        )
        .map_err(|e| anyhow::anyhow!("oxipng: {e}"))?;

        // Map quality 1–100 → preset 6–1
        let preset = ((100 - quality.min(100)) * 6 / 99).min(6) as u8;
        let opts = oxipng::Options::from_preset(preset);

        raw.create_optimized_png(&opts)
            .map_err(|e| anyhow::anyhow!("oxipng encode: {e}"))
    }

    // ── WebP via webp crate ──────────────────────────────────
    fn encode_webp(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let raw = rgba.into_raw();

        let encoder = webp::Encoder::new(&raw, webp::PixelLayout::Rgba, w, h);
        let memory = encoder.encode(quality as f32);
        Ok(memory.to_vec())
    }

    // ── Fallback via image crate ─────────────────────────────
    fn encode_fallback(img: &DynamicImage, fmt: ImageFormat) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), fmt)?;
        Ok(buf)
    }

    // ── Thumbnail ────────────────────────────────────────────
    pub fn generate_thumbnail(path: &Path, size: u32) -> Result<DynamicImage> {
        let img = image::open(path)?;
        Ok(img.thumbnail(size, size))
    }

    /// Returns (width, height) without full decode where possible.
    pub fn dimensions(path: &Path) -> Result<(u32, u32)> {
        let reader = image::ImageReader::open(path)?.with_guessed_format()?;
        Ok(reader.into_dimensions()?)
    }
}
