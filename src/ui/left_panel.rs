use egui::{RichText, Ui};
use std::path::PathBuf;

use crate::app::Kiorg;
use crate::ui::file_list::{self, ROW_HEIGHT};
use crate::ui::style::{HEADER_FONT_SIZE, HEADER_ROW_HEIGHT};

/// Draws the left panel (parent directory list).
/// Returns Some(PathBuf) if a directory was clicked for navigation.
pub fn draw(app: &Kiorg, ui: &mut Ui, width: f32, height: f32) -> Option<PathBuf> {
    let tab = app.tab_manager.current_tab_ref();
    let parent_entries = tab.parent_entries.clone();
    let parent_selected_index = tab.parent_selected_index;
    let colors = &app.colors;
    let bookmarks = &app.bookmarks;

    let mut path_to_navigate = None;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.label(
            RichText::new("Parent Directory")
                .color(colors.gray)
                .font(egui::FontId::proportional(HEADER_FONT_SIZE)),
        );
        ui.separator();

        // Calculate available height for scroll area
        let available_height = height - HEADER_ROW_HEIGHT;

        egui::ScrollArea::vertical()
            .id_salt("parent_list_scroll")
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show(ui, |ui| {
                // Set the width of the content area
                let scrollbar_width = 6.0;
                ui.set_min_width(width - scrollbar_width);
                ui.set_max_width(width - scrollbar_width);

                // Draw all rows
                for (i, entry) in parent_entries.iter().enumerate() {
                    let is_bookmarked = bookmarks.contains(&entry.path);
                    let response = file_list::draw_parent_entry_row(
                        ui,
                        entry,
                        i == parent_selected_index,
                        colors,
                        is_bookmarked,
                    );
                    if response.clicked() {
                        path_to_navigate = Some(entry.path.clone());
                    }

                    // Also navigate on double-click
                    if response.double_clicked() {
                        path_to_navigate = Some(entry.path.clone());
                    }
                }

                // Ensure current directory is visible in parent list
                if !parent_entries.is_empty() {
                    let selected_pos = parent_selected_index as f32 * ROW_HEIGHT;
                    ui.scroll_to_rect(
                        egui::Rect::from_min_size(
                            egui::pos2(0.0, selected_pos),
                            egui::vec2(width, ROW_HEIGHT),
                        ),
                        Some(egui::Align::Center),
                    );
                }
            });
    });

    path_to_navigate
}
