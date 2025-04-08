use egui::{RichText, Ui};
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::fs;

use crate::ui::style::{HEADER_ROW_HEIGHT, HEADER_FONT_SIZE};
use crate::app::Kiorg;

const PANEL_SPACING: f32 = 10.0;

/// Draws the right panel (preview).
pub fn draw(app: &Kiorg, ui: &mut Ui, width: f32, height: f32) {
    let tab = app.tab_manager.current_tab_ref();
    let colors = &app.colors;
    let preview_content = &app.preview_content;
    let current_image = &app.current_image;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.label(
            RichText::new("Preview")
                .color(colors.gray)
                .font(egui::FontId::proportional(HEADER_FONT_SIZE)),
        );
        ui.separator();

        // Calculate available height for scroll area
        let available_height = height - HEADER_ROW_HEIGHT;

        egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show(ui, |ui| {
                // Set the width of the content area
                let scrollbar_width = 6.0;
                ui.set_min_width(width - scrollbar_width);
                ui.set_max_width(width - scrollbar_width);

                // Draw preview content
                if let Some(entry) = tab.entries.get(tab.selected_index) {
                    if entry.is_dir {
                        ui.label(
                            RichText::new(format!("Directory: {}", entry.path.display()))
                                .color(colors.fg),
                        );
                    } else {
                        // Show image preview if available
                        if let Some(texture) = current_image {
                            ui.centered_and_justified(|ui| {
                                let available_width = width - PANEL_SPACING * 2.0;
                                let available_height = available_height - PANEL_SPACING * 2.0;
                                let image_size = texture.size_vec2();
                                let scale = (available_width / image_size.x)
                                    .min(available_height / image_size.y);
                                let scaled_size = image_size * scale;

                                ui.add(egui::Image::new((texture.id(), scaled_size)));
                            });
                        }

                        // Show text preview
                        if !preview_content.is_empty() {
                            ui.label(RichText::new(preview_content).color(colors.fg));
                        }
                    }
                } else {
                    ui.label(RichText::new("No file selected").color(colors.fg));
                }
            });

        // Draw help text in its own row at the bottom
        ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
            ui.label(RichText::new("? for help").color(colors.gray));
        });
    });
}

pub fn update_preview_cache(app: &mut Kiorg, ctx: &egui::Context) {
    let tab = app.tab_manager.current_tab_ref();
    let selected_path = tab
        .entries
        .get(tab.selected_index)
        .map(|e| e.path.clone());

    // Check if the selected file is the same as the cached one in app
    if selected_path.as_ref() == app.cached_preview_path.as_ref() {
        return; // Cache hit, no need to update
    }

    // Cache miss, update the preview content in app
    let maybe_entry = selected_path.as_ref().and_then(|p| {
        tab.entries
            .iter()
            .find(|entry| &entry.path == p)
            .cloned() // Clone the entry data if found
    });
    app.cached_preview_path = selected_path; // Update the cached path in app regardless

    if let Some(entry) = maybe_entry {
        if entry.is_dir {
            app.preview_content = format!("Directory: {}", entry.path.display());
            app.current_image = None;
        } else {
            // Check if it's an image file
            let is_image = entry
                .path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .is_some_and(|ext| {
                    ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str())
                });

            if is_image {
                match fs::read(&entry.path) {
                    Ok(bytes) => {
                        match ImageReader::new(Cursor::new(bytes))
                            .with_guessed_format()
                        {
                            Ok(reader) => match reader.decode() {
                                Ok(img) => {
                                    let size = [img.width() as _, img.height() as _];
                                    let image_buffer = img.to_rgba8();
                                    let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        image_buffer.as_raw(),
                                    );
                                    app.current_image = Some(ctx.load_texture(
                                        entry.path.to_string_lossy().to_string(),
                                        egui_image,
                                        egui::TextureOptions::default(),
                                    ));
                                    app.preview_content =
                                        format!("Image: {}x{}", img.width(), img.height());
                                }
                                Err(e) => {
                                    eprintln!("Failed to decode image {:?}: {}", entry.path, e);
                                    app.preview_content = format!("Failed to decode image: {}", e);
                                    app.current_image = None;
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed guess image format {:?}: {}", entry.path, e);
                                app.preview_content = format!("Failed guess image format: {}", e);
                                app.current_image = None;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read image file {:?}: {}", entry.path, e);
                        app.preview_content = format!("Failed to read file: {}", e);
                        app.current_image = None;
                    }
                }
            } else {
                // Clear image texture for non-image files
                app.current_image = None;
                match std::fs::read_to_string(&entry.path) {
                    Ok(content) => {
                        // Only show first 1000 characters for text files
                        app.preview_content = content.chars().take(1000).collect();
                    }
                    Err(_) => {
                        // For binary files or files that can't be read
                        app.preview_content = format!("Binary file: {} bytes", entry.size);
                    }
                }
            }
        }
    } else {
        app.preview_content.clear();
        app.current_image = None;
        app.cached_preview_path = None; // Clear cache in app if no file is selected
    }
}
