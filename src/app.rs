use anyhow::Result;

use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::image_processor::{CompressionOptions, ImageProcessor, OutputFormat};
use crate::utils::{desktop_or_fallback, is_supported_image, output_path};

slint::include_modules!();

// ──────────────────────────────────────────────────────────────
// Internal image record (what we keep in RAM)
// ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImageRecord {
    pub id: i32,
    pub path: PathBuf,
    pub original_size: usize,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub quality: i32,
    pub target_format: OutputFormat,
    pub compressed_data: Option<Vec<u8>>,
    pub compressed_size: usize,
    pub target_size_percent: i32,
    pub max_dimension: i32,
}

// ──────────────────────────────────────────────────────────────
// App
// ──────────────────────────────────────────────────────────────
pub struct App {
    ui: AppWindow,
    state: Arc<Mutex<AppState>>,
}

struct AppState {
    records: Vec<ImageRecord>,
    next_id: i32,
}

impl App {
    pub fn new() -> Result<Self> {
        let ui = AppWindow::new()?;
        let state = Arc::new(Mutex::new(AppState {
            records: Vec::new(),
            next_id: 0,
        }));
        let app = Self { ui, state };
        app.wire_callbacks();
        Ok(app)
    }

    pub fn run(self) -> Result<()> {
        self.ui.run()?;
        Ok(())
    }

    fn wire_callbacks(&self) {
        let ui = self.ui.as_weak();
        let state = Arc::clone(&self.state);

        // ── add-images ───────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_add_images(move || {
                let picked = rfd::FileDialog::new()
                    .add_filter(
                        "Images",
                        &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff", "tif"],
                    )
                    .set_directory(dirs::home_dir().unwrap_or_default())
                    .pick_files();

                if let Some(paths) = picked {
                    let paths: Vec<PathBuf> = paths.into_iter().collect();
                    let state3 = Arc::clone(&state2);
                    ui2.upgrade_in_event_loop(move |ui| {
                        add_paths_to_model(&ui, &state3, paths);
                    })
                    .ok();
                }
            });
        }

        // ── drop-files ───────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_drop_files(move |paths: ModelRc<SharedString>| {
                let collected: Vec<PathBuf> = paths
                    .iter()
                    .map(|s| PathBuf::from(s.as_str()))
                    .filter(|p| is_supported_image(p))
                    .collect();
                let state3 = Arc::clone(&state2);
                ui2.upgrade_in_event_loop(move |ui| {
                    add_paths_to_model(&ui, &state3, collected);
                })
                .ok();
            });
        }

        // ── remove-image ─────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_remove_image(move |index: i32| {
                {
                    let mut st = state2.lock().unwrap();
                    if (index as usize) < st.records.len() {
                        st.records.remove(index as usize);
                    }
                }
                ui2.upgrade_in_event_loop(move |ui| {
                    let model: ModelRc<ImageItem> = ui.get_images();
                    let mut items: Vec<ImageItem> = model.iter().collect();
                    if (index as usize) < items.len() {
                        items.remove(index as usize);
                    }
                    ui.set_images(Rc::new(VecModel::from(items)).into());
                })
                .ok();
            });
        }

        // ── set-quality ──────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_set_quality(move |index: i32, quality: i32| {
                {
                    let mut st = state2.lock().unwrap();
                    if let Some(rec) = st.records.get_mut(index as usize) {
                        rec.quality = quality;
                    }
                }
                ui2.upgrade_in_event_loop(move |ui| {
                    update_item(&ui, index as usize, |item| {
                        item.quality = quality;
                    });
                })
                .ok();
            });
        }

        // ── set-format ───────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_set_format(move |index: i32, fmt: SharedString| {
                let target = OutputFormat::from_str(fmt.as_str());
                {
                    let mut st = state2.lock().unwrap();
                    if let Some(rec) = st.records.get_mut(index as usize) {
                        rec.target_format = target;
                    }
                }
                let fmt2 = fmt.clone();
                ui2.upgrade_in_event_loop(move |ui| {
                    update_item(&ui, index as usize, |item| {
                        item.target_format = fmt2.clone();
                    });
                })
                .ok();
            });
        }

        // ── compress-single ──────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_compress_single(move |index: i32, quality: i32| {
                ui2.upgrade_in_event_loop(move |ui| {
                    update_item(&ui, index as usize, |item| {
                        item.status = "processing".into();
                    });
                })
                .ok();

                let state3 = Arc::clone(&state2);
                let ui3 = ui2.clone();
                std::thread::spawn(move || {
                    compress_one(index as usize, quality as u8, &state3, &ui3);
                });
            });
        }

        // ── compress-all ─────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_compress_all(move || {
                let state3 = Arc::clone(&state2);
                let ui3 = ui2.clone();

                ui2.upgrade_in_event_loop(|ui| {
                    ui.set_is_processing(true);
                    ui.set_status_text("Compressing all images…".into());
                    // Mark all pending
                    let model = ui.get_images();
                    for i in 0..model.row_count() {
                        let mut item = model.row_data(i).unwrap();
                        if item.status != "processing" {
                            item.status = "processing".into();
                            model.set_row_data(i, item);
                        }
                    }
                })
                .ok();

                std::thread::spawn(move || {
                    let count = state3.lock().unwrap().records.len();
                    for i in 0..count {
                        let q = state3.lock().unwrap().records[i].quality as u8;
                        compress_one(i, q, &state3, &ui3);
                    }
                    ui3.upgrade_in_event_loop(move |ui| {
                        ui.set_is_processing(false);
                        let done = ui
                            .get_images()
                            .iter()
                            .filter(|it| it.status.as_str() == "done")
                            .count();
                        let total = ui.get_images().row_count();
                        ui.set_status_text(format!("{done}/{total} done").into());
                    })
                    .ok();
                });
            });
        }

        // ── export-image ─────────────────────────────────────
        {
            let state2 = Arc::clone(&state);
            self.ui.on_export_image(move |index: i32, _fmt: SharedString| {
                let st = state2.lock().unwrap();
                if let Some(rec) = st.records.get(index as usize) {
                    if let Some(data) = &rec.compressed_data {
                        let suggested = output_path(
                            &rec.path,
                            "_compressed",
                            rec.target_format.extension(),
                        );
                        let data = data.clone();
                        if let Some(save_path) = rfd::FileDialog::new()
                            .set_file_name(
                                suggested
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .as_ref(),
                            )
                            .set_directory(desktop_or_fallback())
                            .save_file()
                        {
                            std::fs::write(&save_path, &data).ok();
                        }
                    }
                }
            });
        }

        // ── save-all ─────────────────────────────────────────
        {
            let state2 = Arc::clone(&state);
            self.ui.on_save_all(move || {
                if let Some(dir) = rfd::FileDialog::new()
                    .set_directory(desktop_or_fallback())
                    .pick_folder()
                {
                    let st = state2.lock().unwrap();
                    for rec in &st.records {
                        if let Some(data) = &rec.compressed_data {
                            let out = output_path(
                                &rec.path,
                                "_compressed",
                                rec.target_format.extension(),
                            );
                            let dest = dir.join(out.file_name().unwrap_or_default());
                            std::fs::write(dest, data).ok();
                        }
                    }
                }
            });
        }

        // ── close-comparison ─────────────────────────────────
        {
            let ui2 = ui.clone();
            self.ui.on_close_comparison(move || {
                ui2.upgrade_in_event_loop(|ui| {
                    ui.set_comparison_index(-1);
                })
                .ok();
            });
        }

        // ── set-target-size-percent ──────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_set_target_size_percent(move |index: i32, pct: i32| {
                {
                    let mut st = state2.lock().unwrap();
                    if let Some(rec) = st.records.get_mut(index as usize) {
                        rec.target_size_percent = pct;
                    }
                }
                ui2.upgrade_in_event_loop(move |ui| {
                    update_item(&ui, index as usize, |item| { item.target_size_percent = pct; });
                }).ok();
            });
        }

        // ── set-max-dimension ────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_set_max_dimension(move |index: i32, dim: i32| {
                {
                    let mut st = state2.lock().unwrap();
                    if let Some(rec) = st.records.get_mut(index as usize) {
                        rec.max_dimension = dim;
                    }
                }
                ui2.upgrade_in_event_loop(move |ui| {
                    update_item(&ui, index as usize, |item| { item.max_dimension = dim; });
                }).ok();
            });
        }

        // ── apply-to-all ─────────────────────────────────────
        {
            let ui2 = ui.clone();
            let state2 = Arc::clone(&state);
            self.ui.on_apply_to_all(move |src_index: i32| {
                let (fmt, q, pct, max_dim) = {
                    let st = state2.lock().unwrap();
                    if let Some(rec) = st.records.get(src_index as usize) {
                        (rec.target_format, rec.quality, rec.target_size_percent, rec.max_dimension)
                    } else {
                        return;
                    }
                };

                {
                    let mut st = state2.lock().unwrap();
                    for rec in &mut st.records {
                        rec.target_format = fmt;
                        rec.quality = q;
                        rec.target_size_percent = pct;
                        rec.max_dimension = max_dim;
                    }
                }

                ui2.upgrade_in_event_loop(move |ui| {
                    let model = ui.get_images();
                    for i in 0..model.row_count() {
                        let mut item = model.row_data(i).unwrap();
                        item.target_format = fmt.display_name().into();
                        item.quality = q;
                        item.target_size_percent = pct;
                        item.max_dimension = max_dim;
                        model.set_row_data(i, item);
                    }
                }).ok();
            });
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Helper: add file paths to state and UI model
// ──────────────────────────────────────────────────────────────
fn add_paths_to_model(ui: &AppWindow, state: &Arc<Mutex<AppState>>, paths: Vec<PathBuf>) {
    let mut new_items: Vec<ImageItem> = Vec::new();
    {
        let mut st = state.lock().unwrap();
        for path in &paths {
            if !is_supported_image(path) {
                continue;
            }
            let meta = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let original_size = meta.len() as usize;
            let (w, h) = ImageProcessor::dimensions(path).unwrap_or((0, 0));
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("?")
                .to_uppercase();
            let target_fmt = OutputFormat::from_str(&ext);
            let id = st.next_id;
            st.next_id += 1;

            st.records.push(ImageRecord {
                id,
                path: path.clone(),
                original_size,
                width: w,
                height: h,
                format: ext.clone(),
                quality: 90,
                target_format: target_fmt,
                compressed_data: None,
                compressed_size: 0,
                target_size_percent: 0,
                max_dimension: 0,
            });

            let file_name: SharedString = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .into();
            let file_path: SharedString = path.to_string_lossy().to_string().into();

            new_items.push(ImageItem {
                id,
                file_name,
                file_path,
                original_size: original_size as i32,
                compressed_size: 0,
                original_width: w as i32,
                original_height: h as i32,
                format: ext.into(),
                target_format: target_fmt.display_name().into(),
                quality: 90,
                status: "pending".into(),
                thumbnail: Image::default(), // placeholder while loading
                compressed_thumbnail: Image::default(),
                compression_ratio: 0.0,
                target_size_percent: 0,
                max_dimension: 0,
            });
        }
    }

    if new_items.is_empty() {
        return;
    }

    let mut existing: Vec<ImageItem> = ui.get_images().iter().collect();
    let start_index = existing.len();
    existing.extend(new_items);
    let count = existing.len();
    ui.set_images(Rc::new(VecModel::from(existing)).into());
    ui.set_status_text(format!("{count} image(s) loaded — generating previews…").into());

    // Generate thumbnails in background, then push them back
    let ui_weak = ui.as_weak();
    let valid_paths: Vec<PathBuf> = paths
        .into_iter()
        .filter(|p| is_supported_image(p) && std::fs::metadata(p).is_ok())
        .collect();

    std::thread::spawn(move || {
        for (offset, path) in valid_paths.iter().enumerate() {
            let index = start_index + offset;
            let raw_opt: Option<(Vec<u8>, u32, u32)> = ImageProcessor::generate_thumbnail(path, 400)
                .ok()
                .map(|img| img.to_rgba8())
                .map(|rgba| {
                    let (w, h) = rgba.dimensions();
                    (rgba.into_raw(), w, h)
                });

            ui_weak
                .upgrade_in_event_loop(move |ui| {
                    if let Some((raw_pixels, w, h)) = raw_opt {
                        let buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                            bytemuck::cast_slice(&raw_pixels),
                            w,
                            h,
                        );
                        update_item(&ui, index, move |item| {
                            item.thumbnail = Image::from_rgba8(buf);
                        });
                    }
                })
                .ok();
        }
        ui_weak
            .upgrade_in_event_loop(move |ui| {
                let count = ui.get_images().row_count();
                ui.set_status_text(format!("{count} image(s) ready").into());
            })
            .ok();
    });
}

// ──────────────────────────────────────────────────────────────
// Helper: compress one record, send result back to UI thread
// ──────────────────────────────────────────────────────────────
fn compress_one(
    index: usize,
    _quality_unused: u8,
    state: &Arc<Mutex<AppState>>,
    ui: &slint::Weak<AppWindow>,
) {
    let (path, target_format, orig_size, quality, target_size_pct, max_dim) = {
        let st = state.lock().unwrap();
        match st.records.get(index) {
            Some(rec) => (
                rec.path.clone(),
                rec.target_format,
                rec.original_size,
                rec.quality,
                rec.target_size_percent,
                rec.max_dimension,
            ),
            None => return,
        }
    };

    let max_dim_opt = if max_dim > 0 { Some(max_dim as u32) } else { None };

    let target_size = if target_size_pct > 0 {
        Some((orig_size as f64 * (target_size_pct as f64 / 100.0)) as usize)
    } else {
        None
    };

    let opts = CompressionOptions {
        quality: quality as u8,
        target_format,
        max_dimension: max_dim_opt,
        target_size,
    };

    match ImageProcessor::process(&path, &opts) {
        Ok(processed) => {
            let compressed_size = processed.size();
            let ratio = if orig_size > 0 {
                (1.0 - compressed_size as f32 / orig_size as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };

            // Build thumbnail from encoded bytes (done in background thread)
            let comp_thumb_raw: Option<Vec<u8>> = image::load_from_memory(&processed.data)
                .ok()
                .map(|img| img.thumbnail(400, 400).to_rgba8().into_raw());
            let comp_thumb_dims: Option<(u32, u32)> = image::load_from_memory(&processed.data)
                .ok()
                .map(|img| {
                    let t = img.thumbnail(400, 400);
                    (t.width(), t.height())
                });

            // Store compressed data
            {
                let mut st = state.lock().unwrap();
                if let Some(rec) = st.records.get_mut(index) {
                    rec.compressed_data = Some(processed.data);
                    rec.compressed_size = compressed_size;
                    rec.quality = quality as i32;
                }
            }

            ui.upgrade_in_event_loop(move |ui| {
                // Build Slint Image on main thread from raw pixels
                let comp_thumb = comp_thumb_raw
                    .zip(comp_thumb_dims)
                    .and_then(|(raw, (w, h))| {
                        let buf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                            bytemuck::cast_slice(&raw),
                            w,
                            h,
                        );
                        Some(Image::from_rgba8(buf))
                    })
                    .unwrap_or_default();

                update_item(&ui, index, move |item| {
                    item.compressed_size = compressed_size as i32;
                    item.compression_ratio = ratio;
                    item.compressed_thumbnail = comp_thumb.clone();
                    item.status = "done".into();
                    item.quality = quality as i32;
                });

                let total = ui.get_images().row_count();
                let done = ui
                    .get_images()
                    .iter()
                    .filter(|it| it.status.as_str() == "done")
                    .count();
                ui.set_status_text(format!("{done}/{total} compressed").into());
            })
            .ok();
        }
        Err(e) => {
            eprintln!("[compress_one #{index}] {e}");
            ui.upgrade_in_event_loop(move |ui| {
                update_item(&ui, index, |item| {
                    item.status = "error".into();
                });
            })
            .ok();
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Helper: mutate a single item in the model in-place
// ──────────────────────────────────────────────────────────────
fn update_item<F: FnOnce(&mut ImageItem)>(ui: &AppWindow, index: usize, f: F) {
    let model: ModelRc<ImageItem> = ui.get_images();
    if index < model.row_count() {
        let mut item = model.row_data(index).unwrap();
        f(&mut item);
        model.set_row_data(index, item);
    }
}

