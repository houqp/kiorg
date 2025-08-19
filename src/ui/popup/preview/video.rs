//! Video preview module for popup display

use crate::config::colors::AppColors;
use crate::models::preview_content::VideoMeta;
use egui::{Image, RichText};

/// Render video content optimized for popup view
///
/// This version focuses on displaying the video thumbnail at a large size
pub fn render_popup(
    ui: &mut egui::Ui,
    video_meta: &VideoMeta,
    colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Use a layout that maximizes thumbnail space
    ui.vertical_centered(|ui| {
        ui.add_space(5.0);

        // Use most available space for the thumbnail
        let max_height = available_height * 0.90;
        let max_width = available_width * 0.90;

        // Add the video thumbnail with maximum possible size
        ui.add(
            Image::new(video_meta.thumbnail.clone())
                .max_size(egui::vec2(max_width, max_height))
                .maintain_aspect_ratio(true),
        );

        ui.add_space(10.0);

        // Show duration if available
        if let Some(duration) = video_meta.metadata.get("Duration") {
            ui.label(
                RichText::new(format!("Duration: {duration}"))
                    .color(colors.fg)
                    .size(14.0),
            );
        }

        // Show dimensions if available
        if let Some(dimensions) = video_meta.metadata.get("Dimensions") {
            ui.label(
                RichText::new(format!("Resolution: {dimensions}"))
                    .color(colors.fg_light)
                    .size(12.0),
            );
        }

        ui.add_space(5.0);
    });
}
