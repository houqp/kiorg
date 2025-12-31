use egui::{RichText, Ui};

use crate::app::Kiorg;
use crate::models::preview_content::PreviewContent;
use crate::ui::preview;
use crate::ui::style::{HEADER_ROW_HEIGHT, section_title_text};

const PANEL_SPACING: f32 = 10.0;

/// Draws the right panel (preview).
pub fn draw(app: &mut Kiorg, _ctx: &egui::Context, ui: &mut Ui, width: f32, height: f32) {
    if matches!(app.show_popup, Some(crate::ui::popup::PopupType::Preview)) {
        // If preview is alwready shown in a popup, avoid unnecessary rendering in this panel
        return;
    }

    let colors = &app.colors;

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
                match &app.preview_content {
                    Some(PreviewContent::Loading(path, _, _)) => {
                        // Display loading indicator
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
                    Some(PreviewContent::Text(text)) => {
                        preview::text::render(ui, text, colors);
                    }
                    Some(PreviewContent::HighlightedCode { content, language }) => {
                        preview::text::render_highlighted(ui, content, language);
                    }
                    Some(PreviewContent::PluginPreview { components }) => {
                        preview::plugin::render(
                            ui,
                            components,
                            colors,
                            available_width,
                            available_height,
                        );
                    }
                    Some(PreviewContent::Image(image_meta)) => {
                        preview::image::render(
                            ui,
                            image_meta,
                            colors,
                            available_width,
                            available_height,
                        );
                    }
                    Some(PreviewContent::Video(video_meta)) => {
                        preview::video::render(
                            ui,
                            video_meta,
                            colors,
                            available_width,
                            available_height,
                        );
                    }
                    Some(PreviewContent::Pdf(pdf_meta)) => {
                        preview::doc::render(
                            ui,
                            pdf_meta,
                            colors,
                            available_width,
                            available_height,
                        );
                    }
                    Some(PreviewContent::Epub(epub_meta)) => {
                        preview::doc::render_epub(
                            ui,
                            epub_meta,
                            colors,
                            available_width,
                            available_height,
                        );
                    }
                    Some(PreviewContent::Zip(entries)) => {
                        preview::zip::render(ui, entries, colors);
                    }
                    Some(PreviewContent::Tar(entries)) => {
                        preview::tar::render(ui, entries, colors);
                    }
                    Some(PreviewContent::Directory(entries)) => {
                        preview::directory::render(ui, entries, colors);
                    }
                    None => {
                        // No file selected or preview not loaded yet
                        preview::text::render_empty(ui, colors);
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
