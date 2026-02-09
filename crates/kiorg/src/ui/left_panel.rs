use egui::Ui;
use std::path::PathBuf;

use crate::app::Kiorg;
use crate::ui::file_list::{self, ROW_HEIGHT};
use crate::ui::style::HEADER_ROW_HEIGHT;

use super::style::section_title_text;

/// Draws the left panel (parent directory list).
/// Returns Some(PathBuf) if a directory was clicked for navigation.
pub fn draw(app: &mut Kiorg, ui: &mut Ui, width: f32, height: f32) -> Option<PathBuf> {
    let tab = app.tab_manager.current_tab_ref();
    let parent_entries = &tab.parent_entries;
    let parent_selected_index = tab.parent_selected_index;
    let colors = &app.colors;
    let bookmarks = &app.bookmarks;

    let mut path_to_navigate = None;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.label(section_title_text("Parent Directory", colors));
        ui.separator();

        // Calculate available height for scroll area
        let available_height = height - HEADER_ROW_HEIGHT;

        egui::ScrollArea::vertical()
            .id_salt("parent_list_scroll")
            .auto_shrink([false; 2])
            .max_height(available_height)
            // TODO: use show_row as an optimization
            .show(ui, |ui| {
                // Set the width of the content area
                let scrollbar_width = 6.0;
                ui.set_min_width(width - scrollbar_width);
                ui.set_max_width(width - scrollbar_width);

                // Draw all rows
                for (i, entry) in parent_entries.iter().enumerate() {
                    let is_bookmarked = bookmarks.contains(&entry.meta.path);
                    // Check if this entry is in the clipboard as a cut or copy operation
                    let (is_in_cut_clipboard, is_in_copy_clipboard) = match &app.clipboard {
                        Some(crate::app::Clipboard::Cut(paths)) => {
                            if paths.contains(&entry.meta.path) {
                                (true, false)
                            } else {
                                (false, false)
                            }
                        }
                        Some(crate::app::Clipboard::Copy(paths)) => {
                            if paths.contains(&entry.meta.path) {
                                (false, true)
                            } else {
                                (false, false)
                            }
                        }
                        None => (false, false),
                    };
                    let response = file_list::draw_parent_entry_row(
                        ui,
                        entry,
                        i == parent_selected_index,
                        colors,
                        is_bookmarked,
                        is_in_cut_clipboard,
                        is_in_copy_clipboard,
                    );
                    if response.clicked() {
                        path_to_navigate = Some(entry.meta.path.clone());
                    }

                    // Also navigate on double-click
                    if response.double_clicked() {
                        path_to_navigate = Some(entry.meta.path.clone());
                    }
                }

                // Ensure current directory is visible in parent list
                if app.scroll_left_panel && !parent_entries.is_empty() {
                    let ui_spacing = ui.spacing().item_spacing.y;
                    let spaced_row_height = ROW_HEIGHT + ui_spacing;
                    let selected_pos = parent_selected_index as f32 * spaced_row_height;
                    ui.scroll_to_rect(
                        egui::Rect::from_min_size(
                            egui::pos2(0.0, selected_pos),
                            egui::vec2(width, ROW_HEIGHT),
                        ),
                        Some(egui::Align::Center),
                    );
                    app.scroll_left_panel = false;
                }
            });
    });

    path_to_navigate
}
