//! Image preview module for popup display

use crate::models::preview_content::ImageMeta;

/// Render image content optimized for popup view
///
/// This version focuses on displaying the image at a large size without metadata tables
pub fn render_popup(
    ui: &mut egui::Ui,
    image_meta: &ImageMeta,
    available_width: f32,
    available_height: f32,
) {
    crate::ui::preview::image::render_interactive(
        ui,
        &image_meta.image,
        available_width,
        available_height,
    );
}
