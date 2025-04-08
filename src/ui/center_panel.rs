use egui::Ui;
use std::path::PathBuf;

use crate::config;
use crate::config::SortPreference;
use crate::ui::file_list::{self, TableHeaderParams, ROW_HEIGHT};
use crate::app::Kiorg;

/// Handles clipboard paste operations (copy/cut)
/// Returns true if any operation was performed
pub fn handle_clipboard_operations(
    clipboard: &mut Option<(Vec<PathBuf>, bool)>,
    current_path: &std::path::Path,
) -> bool {
    if let Some((paths, is_cut)) = clipboard.take() {
        for path in paths {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let mut new_path = current_path.join(name);

            // Handle duplicate names
            let mut counter = 1;
            while new_path.exists() {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| format!(".{}", e))
                    .unwrap_or_default();
                new_path = current_path.join(format!("{}_{}{}", stem, counter, ext));
                counter += 1;
            }

            if is_cut {
                if let Err(e) = std::fs::rename(&path, &new_path) {
                    eprintln!("Failed to move: {e}");
                }
            } else if let Err(e) = std::fs::copy(&path, &new_path) {
                eprintln!("Failed to copy: {e}");
            }
        }
        true
    } else {
        false
    }
}

/// Draws the center panel content.
pub fn draw(app: &mut Kiorg, ui: &mut Ui, width: f32, height: f32) {
    let tab = app.tab_manager.current_tab();

    // Sort entries before cloning them
    tab.sort_entries();

    // Clone necessary data or access directly if borrowing allows
    let entries = tab.entries.clone();
    let selected_index = tab.selected_index;
    let selected_entries = tab.selected_entries.clone();
    let sort_column = tab.sort_column.clone();
    let sort_order = tab.sort_order.clone();

    let mut path_to_navigate = None;
    let mut entry_to_rename = None;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);

        // Draw table header with sorting
        let mut header_params = TableHeaderParams {
            colors: &app.colors,
            sort_column: &sort_column,
            sort_order: &sort_order,
            on_sort: &mut |column| {
                let tab = app.tab_manager.current_tab(); // Get mutable tab again
                tab.toggle_sort(column);
                tab.sort_entries();

                // Save sort preferences to config with override support
                let mut config = config::load_config_with_override(app.config_dir_override.as_ref());
                config.sort_preference = Some(SortPreference {
                    column: tab.sort_column.clone(), // Use tab's current state
                    order: tab.sort_order.clone(),
                });
                if let Err(e) = config::save_config_with_override(&config, app.config_dir_override.as_ref()) {
                    eprintln!("Failed to save sort preferences: {}", e);
                }
            },
        };
        file_list::draw_table_header(ui, &mut header_params);

        // Calculate available height for scroll area
        let available_height = height - ROW_HEIGHT;

        egui::ScrollArea::vertical()
            .id_salt("current_list_scroll")
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show(ui, |ui| {
                // Set the width of the content area
                let scrollbar_width = 6.0;
                ui.set_min_width(width - scrollbar_width);
                ui.set_max_width(width - scrollbar_width);

                // Draw entries
                for (i, entry) in entries.iter().enumerate() {
                    let is_selected = i == selected_index;
                    let is_in_selection = selected_entries.contains(&entry.path);

                    let response = file_list::draw_entry_row(
                        ui,
                        file_list::EntryRowParams {
                            entry,
                            is_selected,
                            colors: &app.colors,
                            rename_mode: app.rename_mode && is_selected,
                            new_name: &mut app.new_name, // Pass directly from app
                            rename_focus: app.rename_focus && is_selected,
                            is_marked: is_in_selection,
                            is_bookmarked: app.bookmarks.contains(&entry.path),
                        },
                    );

                    // Check for clicks/activation after drawing
                    if response {
                        if app.rename_mode && is_selected { // Check rename_mode state from app
                            // Store rename info, handle outside the draw loop
                            entry_to_rename = Some((entry.path.clone(), app.new_name.clone()));
                        } else {
                            // Store navigation info, handle outside the draw loop
                            path_to_navigate = Some(entry.path.clone());
                        }
                    }
                }

                // Handle scrolling to selected item
                if app.ensure_selected_visible && !entries.is_empty() {
                    let selected_pos = selected_index as f32 * ROW_HEIGHT;
                    ui.scroll_to_rect(
                        egui::Rect::from_min_size(
                            egui::pos2(0.0, selected_pos),
                            egui::vec2(width, ROW_HEIGHT), // Use full width for scroll target
                        ),
                        Some(egui::Align::Center),
                    );
                }
            });
    });

    // --- Handle actions after UI drawing ---

    // Handle navigation
    if let Some(path) = path_to_navigate {
        app.navigate_to(path);
    }

    // Handle rename completion
    if let Some((old_path, new_name)) = entry_to_rename {
        if let Some(parent) = old_path.parent() {
            let new_path = parent.join(new_name);
            if let Err(e) = std::fs::rename(&old_path, &new_path) {
                eprintln!("Failed to rename: {e}");
            } else {
                // Refresh entries only on successful rename
                app.refresh_entries();
            }
        }
        // Reset rename state regardless of success/failure
        app.rename_mode = false;
        app.new_name.clear();
        app.rename_focus = false;
    }
}
