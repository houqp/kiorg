use egui::{RichText, Ui};

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;
use crate::ui::preview;
use crate::ui::style::{HEADER_ROW_HEIGHT, section_title_text};

const PANEL_SPACING: f32 = 10.0;

/// Draws the right panel (preview).
pub fn draw(app: &mut Kiorg, ctx: &egui::Context, ui: &mut Ui, width: f32, height: f32) {
    let colors = &app.colors;

    // Handle the loading case separately to avoid borrowing app in the closure
    let mut loading_path = None;
    let mut loading_receiver = None;

    // Clone the preview content to avoid borrow issues
    let (is_loading, preview_content) = match &app.preview_content {
        Some(PreviewContent::Loading(path, receiver, _cancel_sender)) => {
            loading_path = Some(path.clone());
            loading_receiver = Some(receiver.clone());

            (true, None)
        }
        // TODO: avoid the full clone here
        other => (false, other.clone()),
    };

    // Process loading state outside the UI closure
    if is_loading && let (Some(_path), Some(receiver)) = (&loading_path, &loading_receiver) {
        // Check if we have a receiver to poll for results
        let receiver_opt = Some(receiver.clone());
        let receiver = if let Some(receiver) = receiver_opt {
            receiver
        } else {
            // We can't process the loading state, so render empty
            preview::text::render_empty(ui, colors);
            return;
        };

        // Try to get a lock on the receiver
        let receiver_lock = if let Ok(lock) = receiver.lock() {
            lock
        } else {
            // We can't process the loading state, so render empty
            preview::text::render_empty(ui, colors);
            return;
        };

        // Try to receive the result without blocking
        if let Ok(result) = receiver_lock.try_recv() {
            // Request a repaint to update the UI with the result
            ctx.request_repaint();
            // Update the preview content with the result
            match result {
                Ok(content) => {
                    // Set the preview content directly with the received content
                    app.preview_content = Some(content);
                }
                Err(e) => {
                    app.preview_content =
                        Some(PreviewContent::text(format!("Error loading file: {e}")));
                }
            }
            // refresh with next frame
            return;
        }
    }

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.set_max_height(height);
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

                let available_width = PANEL_SPACING.mul_add(-2.0, width);
                let available_height = PANEL_SPACING.mul_add(-2.0, available_height);

                // Draw preview content based on the enum variant
                if is_loading {
                    // Display loading indicator
                    if let Some(path) = &loading_path {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(format!(
                                    "Loading preview contents for {}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ))
                                .color(colors.fg),
                            );
                        });
                    }
                } else {
                    match preview_content {
                        Some(PreviewContent::Text(ref text)) => {
                            preview::text::render(ui, text, colors);
                        }
                        Some(PreviewContent::HighlightedCode {
                            ref content,
                            language,
                        }) => {
                            preview::text::render_highlighted(ui, content, language);
                        }
                        Some(PreviewContent::Image(ref image_meta)) => {
                            preview::image::render(
                                ui,
                                image_meta,
                                colors,
                                available_width,
                                available_height,
                            );
                        }
                        Some(PreviewContent::Video(ref video_meta)) => {
                            preview::video::render(
                                ui,
                                video_meta,
                                colors,
                                available_width,
                                available_height,
                            );
                        }
                        Some(PreviewContent::Pdf(ref pdf_meta)) => {
                            preview::doc::render(
                                ui,
                                pdf_meta,
                                colors,
                                available_width,
                                available_height,
                            );
                        }
                        Some(PreviewContent::Epub(ref epub_meta)) => {
                            preview::doc::render_epub(
                                ui,
                                epub_meta,
                                colors,
                                available_width,
                                available_height,
                            );
                        }
                        Some(PreviewContent::Zip(ref entries)) => {
                            preview::zip::render(ui, entries, colors);
                        }
                        Some(PreviewContent::Tar(ref entries)) => {
                            preview::tar::render(ui, entries, colors);
                        }
                        Some(PreviewContent::Directory(ref entries)) => {
                            preview::directory::render(ui, entries, colors);
                        }
                        None => {
                            // No file selected or preview not loaded yet
                            preview::text::render_empty(ui, colors);
                        }
                        _ => {} // Other cases already handled
                    }
                }
            });

        // Draw help text in its own row at the bottom
        ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
            ui.add_space(2.0);
            ui.label(egui::RichText::new("? for help").color(colors.fg_light));
        });
    });
}
