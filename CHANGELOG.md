# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.5] - 2026-05-06

### Added
- Limiters for Maximum Dimension and Target File Size (percentage of original) to ImageCard.
- Target File Size limiter performs a 2-pass encoding search to achieve the desired file size.
- "Apply to All" button added to each image card to quickly clone settings across the queue.
- Global transparent blocking overlay and loading indicator during image processing to prevent unintended interactions.

### Fixed
- Fixed an issue on Windows where a console window would appear when launching the binary.

## [1.0.4] - 2026-05-06

### Added
- Format selector to individual image cards to choose output encoding (JPEG/PNG/WebP).
- Official PixelCrush logo (`icon.ico`, `icon.icns`, `icon.png`) integrated as the application and window icon.
- Cross-platform icon build steps via `winres` in `build.rs` for Windows executables.
- "About & Credits" section within the main application window with Slint licensing information.
- Documentation for running unsigned binaries on macOS and Windows in `README.md`.

### Changed
- Improved application UI identity and updated copyright year to 2026 under torocruzand branding.
- Enhanced responsive image grid system using a dynamic layout algorithm in Slint that prevents spacing issues with hidden elements.
- Centered header toolbar buttons vertically for a more polished aesthetic.
- Increased default compression quality from 80% to 90% for higher fidelity output.
- Adjusted image card height from 430px to 450px to accommodate the new format selectors.

### Fixed
- Fixed bug causing UI flashing when hovering over elements in the `ImageCard`.
- Fixed integer division issue that incorrectly displaced the quality slider thumb.
- Fixed window resize layout issues by replacing static rows with a dynamic absolute positioning grid inside a `ScrollView`.
- Fixed vertical alignment and invalid padding/coordinate references in the toolbar header.

## [1.0.1] - 2026-05-05
### Added
- Initial release of the PixelCrush Rust backend with Slint UI.
- Background thread processing for image compression.
- Drag-and-drop file support.
