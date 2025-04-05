use egui::{RichText, TextureHandle, Ui};
use image::io::Reader as ImageReader;
use std::io::Cursor;

use crate::config::colors::AppColors;
use crate::models::tab::Tab;
use crate::ui::file_list::ROW_HEIGHT;
use crate::ui::style::VERTICAL_PADDING;

const PANEL_SPACING: f32 = 10.0;

pub struct RightPanel {
    width: f32,
    height: f32,
}

impl RightPanel {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn draw(
        &self,
        ui: &mut Ui,
        tab: &Tab,
        colors: &AppColors,
        preview_content: &str,
        current_image: &Option<TextureHandle>,
    ) {
        ui.vertical(|ui| {
            ui.set_min_width(self.width);
            ui.set_max_width(self.width);
            ui.set_min_height(self.height);
            ui.add_space(VERTICAL_PADDING);
            ui.label(RichText::new("Preview").color(colors.gray));
            ui.separator();

            // Calculate available height for scroll area
            let available_height = self.height - ROW_HEIGHT - VERTICAL_PADDING * 4.0;

            egui::ScrollArea::vertical()
                .id_salt("preview_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0;
                    ui.set_min_width(self.width - scrollbar_width);
                    ui.set_max_width(self.width - scrollbar_width);

                    // Draw preview content
                    if let Some(entry) = tab.entries.get(tab.selected_index) {
                        if entry.is_dir {
                            ui.label(RichText::new(format!("Directory: {}", entry.path.display())).color(colors.fg));
                        } else {
                            // Show image preview if available
                            if let Some(texture) = current_image {
                                ui.centered_and_justified(|ui| {
                                    let available_width = self.width - PANEL_SPACING * 2.0;
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
            ui.add_space(VERTICAL_PADDING);
        });
    }
}

pub fn update_preview(
    tab: &Tab,
    ctx: &egui::Context,
    preview_content: &mut String,
    current_image: &mut Option<TextureHandle>,
) {
    if let Some(entry) = tab.entries.get(tab.selected_index) {
        if entry.is_dir {
            *preview_content = format!("Directory: {}", entry.path.display());
            *current_image = None;
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
                if let Ok(bytes) = std::fs::read(&entry.path) {
                    if let Ok(img) = ImageReader::new(Cursor::new(bytes))
                        .with_guessed_format()
                        .unwrap()
                        .decode()
                    {
                        let size = [img.width() as _, img.height() as _];
                        let image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            img.to_rgba8().as_raw(),
                        );
                        *current_image = Some(ctx.load_texture(
                            entry.path.to_string_lossy().to_string(),
                            image,
                            egui::TextureOptions::default(),
                        ));
                        *preview_content = format!("Image: {}x{}", img.width(), img.height());
                        return;
                    }
                }
            }

            // Clear image texture for non-image files
            *current_image = None;
            match std::fs::read_to_string(&entry.path) {
                Ok(content) => {
                    // Only show first 1000 characters for text files
                    *preview_content = content.chars().take(1000).collect();
                }
                Err(_) => {
                    // For binary files or files that can't be read
                    *preview_content = format!("Binary file: {} bytes", entry.size);
                }
            }
        }
    } else {
        preview_content.clear();
        *current_image = None;
    }
}
