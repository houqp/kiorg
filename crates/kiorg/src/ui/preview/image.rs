//! Image preview module

use crate::config::colors::AppColors;
use crate::models::dir_entry::DirEntryMeta;
use crate::models::preview_content::{CachedImageMeta, CachedPreviewContent, ImageMeta, metadata};
use crate::utils::preview_cache;
use egui::{Rect, RichText};
use image::{GenericImageView, ImageDecoder, ImageFormat};
use std::collections::HashMap;
use std::path::Path;

/// Render image content
pub fn render(
    ui: &mut egui::Ui,
    image_meta: &ImageMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Display image title
    ui.label(
        RichText::new(&image_meta.title)
            .color(colors.fg)
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    // Display image (centered)
    ui.vertical_centered(|ui| {
        ui.add(
            image_meta
                .image
                .clone()
                .max_size(egui::vec2(available_width, available_height * 0.6))
                .maintain_aspect_ratio(true),
        );
    });
    ui.add_space(15.0);

    // Create a table for regular metadata
    ui.label(
        RichText::new("Image Metadata")
            .color(colors.fg_folder)
            .strong()
            .size(14.0),
    );
    ui.add_space(5.0);
    egui::Grid::new("image_metadata_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            // Sort keys for consistent display
            let mut sorted_keys: Vec<&String> = image_meta.metadata.keys().collect();
            sorted_keys.sort();

            // Display each metadata field in a table row
            for key in sorted_keys {
                if let Some(value) = image_meta.metadata.get(key) {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.set_min_width(super::METADATA_TBL_KEY_COL_W);
                        ui.set_max_width(super::METADATA_TBL_KEY_COL_W);
                        ui.add(egui::Label::new(RichText::new(key).color(colors.fg)).wrap());
                    });
                    ui.add(egui::Label::new(RichText::new(value).color(colors.fg)).wrap());
                    ui.end_row();
                }
            }
        });

    // Display EXIF data in a separate table if available
    if let Some(exif_data) = &image_meta.exif_data {
        ui.add_space(15.0);
        ui.label(
            RichText::new("EXIF Data")
                .color(colors.fg_folder)
                .strong()
                .size(14.0),
        );
        ui.add_space(5.0);
        egui::Grid::new("exif_data_grid")
            .num_columns(2)
            .spacing([10.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                // Sort keys for consistent display
                let mut sorted_keys: Vec<&String> = exif_data.keys().collect::<Vec<&String>>();
                sorted_keys.sort();

                // Display each EXIF field in a table row
                for key in sorted_keys {
                    if let Some(value) = exif_data.get(key) {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.set_min_width(super::METADATA_TBL_KEY_COL_W);
                            ui.set_max_width(super::METADATA_TBL_KEY_COL_W);
                            ui.add(egui::Label::new(RichText::new(key).color(colors.fg)).wrap());
                        });
                        ui.add(egui::Label::new(RichText::new(value).color(colors.fg)).wrap());
                        ui.end_row();
                    }
                }
            });
    }
}

/// Generate a URI for an image file path
#[inline]
fn image_path_to_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// Read image file, extract metadata, and create `PreviewContent`
pub fn read_image_with_metadata(
    entry: DirEntryMeta,
    ctx: &egui::Context,
    available_width: Option<f32>,
) -> Result<ImageMeta, String> {
    // Get the filename for the title
    let title = entry
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Create a HashMap to store metadata
    let mut metadata = HashMap::new();

    // Open the image file
    let reader =
        image::ImageReader::open(&entry.path).map_err(|e| format!("failed to open image: {e}"))?;
    let img_format = reader.format();
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| format!("failed to create decoder for image: {e}"))?;
    let exif_bytes = decoder
        .exif_metadata()
        .map_err(|e| format!("failed to extract exif metadata: {e}"))?;
    let orientation = decoder
        .orientation()
        .map_err(|e| format!("failed to get image orientation: {e}"))?;

    // Create a separate HashMap for EXIF data
    let mut exif_data = None;

    if let Some(v) = exif_bytes {
        let (fields, _) =
            exif::parse_exif(v.as_ref()).map_err(|e| format!("failed to parse EXIF data: {e}"))?;

        // Only create the HashMap if we have EXIF data
        if !fields.is_empty() {
            let mut exif_map = HashMap::new();
            for field in fields {
                exif_map.insert(
                    format!("{}", field.tag),
                    format!("{}", field.display_value()),
                );
            }
            exif_data = Some(exif_map);
        }
    }

    if let Ok(metadata_os) = std::fs::metadata(&entry.path) {
        let size = metadata_os.len();
        metadata.insert(
            metadata::IMG_FILE_SIZE.to_string(),
            crate::utils::format::format_size(size, false),
        );
    }

    // Try to get format-specific information
    let skip_cache = if let Some(format) = img_format {
        let name = match format {
            ImageFormat::Jpeg => "JPEG".to_string(),
            ImageFormat::Png => "PNG".to_string(),
            ImageFormat::Gif => "GIF".to_string(),
            ImageFormat::WebP => "WebP".to_string(),
            ImageFormat::Tiff => "TIFF".to_string(),
            ImageFormat::Bmp => "BMP".to_string(),
            ImageFormat::Ico => "ICO".to_string(),
            ImageFormat::Tga => "TGA".to_string(),
            ImageFormat::Dds => "DDS".to_string(),
            ImageFormat::Farbfeld => "Farbfeld".to_string(),
            ImageFormat::Avif => "AVIF".to_string(),
            ImageFormat::Qoi => "QOI".to_string(),
            _ => {
                // Handle additional formats from image_extras
                let format_str = format!("{format:?}");
                match format_str.as_str() {
                    "Ora" => "OpenRaster".to_string(),
                    "Otb" => "OTA Bitmap".to_string(),
                    "Pcx" => "PCX".to_string(),
                    "Sgi" => "SGI".to_string(),
                    "Wbmp" => "Wireless Bitmap".to_string(),
                    "Xbm" => "X BitMap".to_string(),
                    "Xpm" => "X PixMap".to_string(),
                    "Pnm" => "PNM".to_string(),
                    _ => format_str,
                }
            }
        };
        metadata.insert(metadata::IMG_FORMAT.to_string(), name);

        // Add format-specific metadata
        if format == ImageFormat::Gif {
            // For GIF files, use URI source to enable animation
            let uri = image_path_to_uri(&entry.path);
            // need to manually clear cache to reload the gif if there is a
            // change in content.
            //
            // ctx.forget_image(&uri) won't work because internal egui animation
            // code creates one uri per gif animation frame.
            for loader in ctx.loaders().image.lock().iter() {
                loader.forget_all();
            }
            let meta = crate::models::preview_content::ImageMeta::from_uri(
                title, metadata, uri, exif_data,
            );

            // Skip cache saving for GIF as requested
            return Ok(meta);
        }

        // Create cache bytes from ORIGINAL image ONLY if it's not already an
        // optimized format
        match format {
            image::ImageFormat::Png | image::ImageFormat::WebP => Some(true),
            _ => Some(false),
        }
    } else {
        None
    }
    .unwrap_or(false);

    let mut img = match image::DynamicImage::from_decoder(decoder) {
        Ok(img) => img,
        Err(e) => return Err(format!("Failed to decode image: {e}")),
    };

    img.apply_orientation(orientation);

    // Extract basic image information
    let dimensions = img.dimensions();
    metadata.insert(
        metadata::IMG_DIMENSIONS.to_string(),
        format!("{}x{} pixels", dimensions.0, dimensions.1),
    );

    let texture_id = format!("image_{}", entry.path.display());

    if !skip_cache {
        let title_clone = title.clone();
        let metadata_clone = metadata.clone();
        let exif_data_clone = exif_data.clone();
        let img_clone = img.clone();

        std::thread::spawn(move || {
            let mut png_bytes = Vec::new();
            if img_clone
                .write_to(
                    &mut std::io::Cursor::new(&mut png_bytes),
                    image::ImageFormat::Png,
                )
                .is_ok()
            {
                let cached_content = CachedPreviewContent::Image(CachedImageMeta {
                    title: title_clone,
                    metadata: metadata_clone,
                    exif_data: exif_data_clone,
                    cache_bytes: Some(png_bytes),
                    uri: None,
                });
                let cache_key = preview_cache::calculate_cache_key(&entry);
                if let Err(e) = preview_cache::save_preview(&cache_key, &cached_content) {
                    tracing::warn!("Failed to save image preview cache: {}", e);
                }
            }
        });
    }

    // Resize for UI texture to save GPU memory
    if let Some(w) = available_width {
        let (width, height) = img.dimensions();
        // Only resize if the image is larger than the available width
        // Cast w to u32 for comparison and resizing
        let w_u32 = w as u32;
        if width > w_u32 {
            let ratio = height as f64 / width as f64;
            let new_height = (w as f64 * ratio) as u32;
            img = img.resize(w_u32, new_height, image::imageops::FilterType::Triangle);
        }
    }

    // Get color type
    metadata.insert(
        metadata::IMG_COLOR_TYPE.to_string(),
        format!("{:?}", img.color()),
    );

    // Add color depth information
    match img.color() {
        image::ColorType::Rgb8 | image::ColorType::Rgba8 => {
            metadata.insert(
                metadata::IMG_BIT_DEPTH.to_string(),
                "8 bits per channel".to_string(),
            );
        }
        image::ColorType::Rgb16 | image::ColorType::Rgba16 => {
            metadata.insert(
                metadata::IMG_BIT_DEPTH.to_string(),
                "16 bits per channel".to_string(),
            );
        }
        image::ColorType::L8 | image::ColorType::La8 => {
            metadata.insert(
                metadata::IMG_BIT_DEPTH.to_string(),
                "8 bits (grayscale)".to_string(),
            );
        }
        image::ColorType::L16 | image::ColorType::La16 => {
            metadata.insert(
                metadata::IMG_BIT_DEPTH.to_string(),
                "16 bits (grayscale)".to_string(),
            );
        }
        _ => {}
    }

    // Convert the image to RGBA8 format for egui
    let rgba8_img = img.to_rgba8();
    let dimensions = rgba8_img.dimensions();
    let size = [dimensions.0 as _, dimensions.1 as _];
    let pixels = rgba8_img.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    let texture = ctx.load_texture(texture_id, color_image, egui::TextureOptions::default());

    let meta = crate::models::preview_content::ImageMeta::new(title, metadata, texture, exif_data);

    Ok(meta)
}

/// Render an interactive image with pan and zoom support
pub fn render_interactive(
    ui: &mut egui::Ui,
    image: &egui::Image<'static>,
    available_width: f32,
    available_height: f32,
) {
    ui.vertical_centered(|ui| {
        let default_init_height = available_height * 0.90;
        let default_init_width = available_width * 0.90;

        let (raw_img_w, raw_img_h) = if let Some(size) = image.size() {
            (size[0], size[1])
        } else {
            // early return if image hasn't been loaded
            ui.allocate_ui_with_layout(
                egui::vec2(default_init_width, default_init_height),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.spinner();
                },
            );
            return;
        };

        // Unique id for storing pan/zoom state per image
        let id = ui.id().with("image_pan_zoom");
        let mut pan = ui.ctx().data(|d| {
            d.get_temp::<egui::Vec2>(id.with("pan"))
                .unwrap_or(egui::Vec2::ZERO)
        });

        let init_zoom = || -> f32 {
            let scale_x = default_init_width / raw_img_w;
            let scale_y = default_init_height / raw_img_h;
            scale_x.min(scale_y).min(1.0)
        };
        let mut zoom = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(id.with("zoom")).unwrap_or_else(init_zoom));
        let mut reset_view = false;

        egui::ScrollArea::both()
            .id_salt("image_scroll_area")
            .wheel_scroll_multiplier(egui::Vec2 { x: zoom, y: zoom })
            .show(ui, |ui| {
                // The viewport is available_width x available_height
                let viewport_size = egui::vec2(available_width, available_height);
                let response =
                    ui.allocate_response(viewport_size, egui::Sense::DRAG | egui::Sense::CLICK);
                // Double click to reset zoom and pan
                if response.double_clicked() {
                    reset_view = true;
                    return;
                }
                // detect pan through click and drag
                if response.dragged() {
                    // drag_delta is absolute value relative to view port without zoom applied
                    pan += response.drag_delta() * zoom;
                }

                // detect pan and zoom through touch pad
                ui.input(|i| {
                    // Pinch zoom: zoom_delta is a relative multiplier, not an offset
                    let zoom_delta = i.zoom_delta();
                    zoom *= zoom_delta;
                    // scroll value is absolute vlaue relative to view port without zoom applied
                    let scroll = i.smooth_scroll_delta;
                    if scroll.x.abs() > 0.0 {
                        pan.x += scroll.x * zoom;
                    }
                    if scroll.y.abs() > 0.0 {
                        pan.y += scroll.y * zoom;
                    }
                });

                // Zoomed image can be larger than the viewport
                let scaled_img_size = egui::vec2(raw_img_w, raw_img_h) * zoom;
                if scaled_img_size.x <= viewport_size.x {
                    // disable panning when image is not zoomed in
                    pan.x = 0.0;
                } else {
                    // Clamp pan so image always shows up in the view area
                    let half_width = scaled_img_size.x / 2.0;
                    pan.x = pan.x.clamp(-half_width, half_width);
                }
                if scaled_img_size.y <= viewport_size.y {
                    pan.y = 0.0;
                } else {
                    let half_height = scaled_img_size.y / 2.0;
                    pan.y = pan.y.clamp(-half_height, half_height);
                }

                // Store updated state
                ui.ctx().data_mut(|d| d.insert_temp(id.with("pan"), pan));
                ui.ctx().data_mut(|d| d.insert_temp(id.with("zoom"), zoom));

                // use from_center_size to always center image when pan is 0
                let paint_rect =
                    Rect::from_center_size(response.rect.center() + pan, scaled_img_size);
                image.paint_at(ui, paint_rect);
            });

        if reset_view {
            zoom = init_zoom();
            pan = egui::Vec2::ZERO;
            ui.ctx().data_mut(|d| d.insert_temp(id.with("pan"), pan));
            ui.ctx().data_mut(|d| d.insert_temp(id.with("zoom"), zoom));
        }
    });
}
