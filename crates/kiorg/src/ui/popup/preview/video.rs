//! Video preview module for popup display

use crate::models::preview_content::VideoMeta;

/// Render video content optimized for popup view
///
/// This version focuses on displaying the video thumbnail at a large size
pub fn render_popup(
    ui: &mut egui::Ui,
    video_meta: &VideoMeta,
    available_width: f32,
    available_height: f32,
) {
    crate::ui::preview::image::render_interactive(
        ui,
        &video_meta.thumbnail_image,
        available_width,
        available_height,
    );
}
