use egui::{RichText, TextureHandle, Ui};

use crate::config::colors::AppColors;
use crate::models::tab::Tab;
use crate::ui::file_list::ROW_HEIGHT;

const VERTICAL_PADDING: f32 = 8.0;
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
                            ui.label(RichText::new("Directory").color(colors.gray));
                        } else if let Some(image) = current_image {
                            ui.centered_and_justified(|ui| {
                                let available_width = self.width - PANEL_SPACING * 2.0;
                                let available_height = available_height - PANEL_SPACING * 2.0;
                                let image_size = image.size_vec2();
                                let scale = (available_width / image_size.x)
                                    .min(available_height / image_size.y);
                                let scaled_size = image_size * scale;

                                ui.add(egui::Image::new((image.id(), scaled_size)));
                            });
                        } else {
                            ui.add(
                                egui::TextEdit::multiline(&mut String::from(preview_content))
                                    .desired_width(self.width - PANEL_SPACING)
                                    .desired_rows(30)
                                    .interactive(false),
                            );
                        }
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
