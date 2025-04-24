use egui::{Image, RichText, Ui};
use std::collections::HashSet;
use std::sync::OnceLock;

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;
use crate::ui::style::{section_title_text, HEADER_ROW_HEIGHT};

/// Global HashSet of supported image extensions for efficient lookups
static IMAGE_EXTENSIONS: OnceLock<HashSet<String>> = OnceLock::new();

/// Get the set of supported image extensions
fn get_image_extensions() -> &'static HashSet<String> {
    IMAGE_EXTENSIONS.get_or_init(|| {
        ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"]
            .iter()
            .map(|&s| s.to_string())
            .collect()
    })
}

const PANEL_SPACING: f32 = 10.0;

/// Draws the right panel (preview).
pub fn draw(app: &Kiorg, ui: &mut Ui, width: f32, height: f32) {
    // No longer need tab reference since we're using the preview_content enum
    // let tab = app.tab_manager.current_tab_ref();
    let colors = &app.colors;
    let preview_content = &app.preview_content;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.label(section_title_text("Preview", colors));
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

                // Draw preview content based on the enum variant
                match preview_content {
                    Some(PreviewContent::Text(text)) => {
                        ui.label(RichText::new(text).color(colors.fg));
                    }
                    Some(PreviewContent::Image(uri)) => {
                        ui.centered_and_justified(|ui| {
                            let available_width = width - PANEL_SPACING * 2.0;
                            let available_height = available_height - PANEL_SPACING * 2.0;
                            // Use the URI directly with the Image widget
                            ui.add(
                                Image::new(uri)
                                    .max_size(egui::vec2(available_width, available_height))
                                    .maintain_aspect_ratio(true),
                            );
                        });
                    }
                    None => {
                        ui.label(RichText::new("No file selected").color(colors.fg));
                    }
                }
            });

        // Draw help text in its own row at the bottom
        ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
            ui.label(RichText::new("? for help").color(colors.gray));
        });
    });
}

pub fn update_preview_cache(app: &mut Kiorg, _ctx: &egui::Context) {
    let tab = app.tab_manager.current_tab_ref();
    let selected_path = tab.entries.get(tab.selected_index).map(|e| e.path.clone());

    // Check if the selected file is the same as the cached one in app
    if selected_path.as_ref() == app.cached_preview_path.as_ref() {
        return; // Cache hit, no need to update
    }

    // Cache miss, update the preview content in app
    let maybe_entry = selected_path.as_ref().and_then(|p| {
        tab.entries.iter().find(|entry| &entry.path == p).cloned() // Clone the entry data if found
    });
    app.cached_preview_path = selected_path; // Update the cached path in app regardless

    if let Some(entry) = maybe_entry {
        if entry.is_dir {
            app.preview_content = Some(PreviewContent::text(format!(
                "Directory: {}",
                entry.path.display()
            )));
        } else {
            let ext = entry
                .path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            match ext {
                Some(ext) if get_image_extensions().contains(&ext) => {
                    app.preview_content = Some(PreviewContent::image(entry.path));
                }
                _ => {
                    match std::fs::read_to_string(&entry.path) {
                        Ok(content) => {
                            // Only show first 1000 characters for text files
                            let preview_text = content.chars().take(1000).collect::<String>();
                            app.preview_content = Some(PreviewContent::text(preview_text));
                        }
                        Err(_) => {
                            // For binary files or files that can't be read
                            app.preview_content = Some(PreviewContent::text(format!(
                                "Binary file: {} bytes",
                                entry.size
                            )));
                        }
                    }
                }
            }
        }
    } else {
        app.preview_content = None; // No content to display
        app.cached_preview_path = None; // Clear cache in app if no file is selected
    }
}
