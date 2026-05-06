use std::path::{Path, PathBuf};

/// Format byte count as a human-readable string.
#[allow(dead_code)]
pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1_048_576 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    }
}

/// Return true if the path looks like a supported image file.
pub fn is_supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "tiff" | "tif")
    )
}

/// Suggest an output path next to the original, with suffix and new extension.
pub fn output_path(original: &Path, suffix: &str, ext: &str) -> PathBuf {
    let stem = original
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let parent = original.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}{suffix}.{ext}"))
}

/// Return the desktop directory or fall back to home/cwd.
pub fn desktop_or_fallback() -> PathBuf {
    dirs::desktop_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}
