//! Image preview module for popup display

use crate::config::colors::AppColors;
use crate::models::preview_content::ImageMeta;
use egui::{Image, Rect};

/// Render image content optimized for popup view
///
/// This version focuses on displaying the image at a large size without metadata tables
pub fn render_popup(
    ui: &mut egui::Ui,
    image_meta: &ImageMeta,
    _colors: &AppColors,
    available_width: f32,
    available_height: f32,
) {
    // Use a layout that maximizes image space and supports pan/zoom
    ui.vertical_centered(|ui| {
        // Calculate available space
        let default_init_height = available_height * 0.90;
        let default_init_width = available_width * 0.90;

        let image = Image::new(image_meta.image_source.clone());
        let (raw_img_w, raw_img_h) = if let Some(size) = image.size() {
            (size[0], size[1])
        } else {
            // fallback: use default init size
            (default_init_width, default_init_height)
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
