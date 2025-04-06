use egui::{RichText, Ui};
use std::path::PathBuf;

use crate::config::colors::AppColors;
use crate::models::tab::Tab;
use crate::ui::file_list::{self, ROW_HEIGHT};
use crate::ui::style::{HEADER_ROW_HEIGHT, HEADER_FONT_SIZE};

pub struct LeftPanel {
    width: f32,
    height: f32,
}

impl LeftPanel {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn draw(
        &self,
        ui: &mut Ui,
        tab: &Tab,
        bookmarks: &[PathBuf],
        colors: &AppColors,
    ) -> Option<PathBuf> {
        let parent_entries = tab.parent_entries.clone();
        let parent_selected_index = tab.parent_selected_index;

        let mut path_to_navigate = None;

        ui.vertical(|ui| {
            ui.set_min_width(self.width);
            ui.set_max_width(self.width);
            ui.set_min_height(self.height);
            ui.label(RichText::new("Parent Directory").color(colors.gray).font(egui::FontId::proportional(HEADER_FONT_SIZE)));
            ui.separator();

            // Calculate available height for scroll area
            let available_height = self.height - HEADER_ROW_HEIGHT;

            egui::ScrollArea::vertical()
                .id_salt("parent_list_scroll")
                .auto_shrink([false; 2])
                .max_height(available_height)
                .show(ui, |ui| {
                    // Set the width of the content area
                    let scrollbar_width = 6.0;
                    ui.set_min_width(self.width - scrollbar_width);
                    ui.set_max_width(self.width - scrollbar_width);

                    // Draw all rows
                    for (i, entry) in parent_entries.iter().enumerate() {
                        let is_bookmarked = bookmarks.contains(&entry.path);
                        let clicked = file_list::draw_parent_entry_row(
                            ui,
                            entry,
                            i == parent_selected_index,
                            colors,
                            is_bookmarked,
                        );
                        if clicked {
                            path_to_navigate = Some(entry.path.clone());
                            break;
                        }
                    }

                    // Ensure current directory is visible in parent list
                    if !parent_entries.is_empty() {
                        let selected_pos = parent_selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(self.width, ROW_HEIGHT),
                            ),
                            Some(egui::Align::Center),
                        );
                    }
                });
        });

        path_to_navigate
    }
} 