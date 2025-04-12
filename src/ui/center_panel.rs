use egui::Ui;
use std::path::PathBuf;

use crate::app::Kiorg;
use crate::config;
use crate::config::SortPreference;
use crate::ui::file_list::{self, TableHeaderParams, ROW_HEIGHT};

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
    let tab_index = app.tab_manager.current_tab_index;

    // Get filtered entries and other data before any closures
    let filtered_entries =
        app.tab_manager.tabs[tab_index].get_filtered_entries(&app.search_bar.query);
    let sort_column = app.tab_manager.tabs[tab_index].sort_column.clone();
    let sort_order = app.tab_manager.tabs[tab_index].sort_order.clone();
    let selected_entries = app.tab_manager.tabs[tab_index].selected_entries.clone();
    let entries_clone = app.tab_manager.tabs[tab_index].entries.clone();
    let selected_index = app.tab_manager.tabs[tab_index].selected_index;
    let colors = app.colors.clone();
    let config_dir_override = app.config_dir_override.clone();
    let bookmarks = app.bookmarks.clone();
    let rename_mode = app.rename_mode;
    let rename_focus = app.rename_focus;
    let mut new_name = app.new_name.clone();
    let ensure_selected_visible = app.ensure_selected_visible;

    let mut path_to_navigate = None;
    let mut entry_to_rename = None;
    let mut sort_requested = None;

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);

        // Draw table header
        let mut header_params = TableHeaderParams {
            colors: &colors,
            sort_column: &sort_column,
            sort_order: &sort_order,
            on_sort: &mut |column| {
                sort_requested = Some(column);
            },
        };
        // Draw the header
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

                // --- Draw Filtered Entries ---
                if filtered_entries.is_empty() {
                    ui.label("No matching entries found.");
                } else {
                    for entry in filtered_entries.iter() {
                        // Find the original index in the full `entries` list for selection checks
                        let original_index = entries_clone
                            .iter()
                            .position(|e| e.path == entry.path)
                            .unwrap_or(usize::MAX); // Should always find it

                        let is_selected = original_index == selected_index;
                        let is_in_selection = selected_entries.contains(&entry.path);

                        let response = file_list::draw_entry_row(
                            ui,
                            file_list::EntryRowParams {
                                entry, // Use the filtered entry
                                is_selected,
                                colors: &colors,
                                rename_mode: rename_mode && is_selected,
                                new_name: &mut new_name,
                                rename_focus: rename_focus && is_selected,
                                is_marked: is_in_selection,
                                is_bookmarked: bookmarks.contains(&entry.path),
                                search_query: &app.search_bar.query,
                            },
                        );

                        // Check for clicks/activation after drawing
                        if response {
                            if rename_mode && is_selected {
                                entry_to_rename = Some((entry.path.clone(), new_name.clone()));
                            } else {
                                path_to_navigate = Some(entry.path.clone());
                            }
                        }
                    }
                }

                // Handle scrolling to selected item
                if ensure_selected_visible && !filtered_entries.is_empty() {
                    // Find the position of the selected item *within the filtered list*
                    if let Some(filtered_selected_index) = filtered_entries.iter().position(|e| {
                        entries_clone
                            .get(selected_index)
                            .is_some_and(|selected_entry| selected_entry.path == e.path)
                    }) {
                        let selected_pos = filtered_selected_index as f32 * ROW_HEIGHT;
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(
                                egui::pos2(0.0, selected_pos),
                                egui::vec2(width, ROW_HEIGHT),
                            ),
                            Some(egui::Align::Center),
                        );
                        app.ensure_selected_visible = false; // Reset flag after scrolling
                    }
                }
                // --- End Filtered Entries ---
            });
    });

    // Update app state with any changes
    app.new_name = new_name;

    // Handle sort request after UI is drawn
    if let Some(column) = sort_requested {
        let tab = app.tab_manager.current_tab();
        tab.toggle_sort(column);
        tab.sort_entries();

        // Save sort preferences after sorting
        let mut config = config::load_config_with_override(config_dir_override.as_ref());
        config.sort_preference = Some(SortPreference {
            column: tab.sort_column,
            order: tab.sort_order,
        });
        if let Err(e) = config::save_config_with_override(&config, config_dir_override.as_ref()) {
            eprintln!("Failed to save sort preferences: {}", e);
        }
    }

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
