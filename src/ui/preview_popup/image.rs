//! Image preview module for popup display

use crate::config::colors::AppColors;
use crate::models::preview_content::ImageMeta;
use egui::{Image, RichText};

/// Render image content optimized for popup view
///
/// This version focuses on displaying the image at a large size without metadata tables
pub fn render_popup(
    ui: &mut egui::Ui,
    image_meta: &ImageMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Use a layout that maximizes image space
    ui.vertical_centered(|ui| {
        ui.add_space(5.0);

        // Calculate available space (use virtually all remaining space)
        let max_height = available_height * 0.90;
        let max_width = available_width * 0.90;

        // Add the image with maximum possible size
        ui.add(
            Image::new(&image_meta.texture)
                .max_size(egui::vec2(max_width, max_height))
                .maintain_aspect_ratio(true),
        );

        // Optional: Very minimal dimensions display if needed
        if let Some(dimensions) = image_meta.metadata.get("Dimensions") {
            ui.add_space(5.0);
            ui.label(RichText::new(dimensions).color(colors.fg_light).size(12.0));
        }
        ui.add_space(5.0);
    });
}
