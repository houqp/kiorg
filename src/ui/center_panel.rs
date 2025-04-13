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
    let tab_ref = app.tab_manager.current_tab_ref(); // Use a different name to avoid confusion
    let current_search_query = &app.search_bar.query;

    // Get filtered entries - needs tab_ref and search query
    let filtered_entries = tab_ref.get_filtered_entries(current_search_query);

    // --- State variables to capture changes from UI closures ---
    let mut new_selected_index = None; // For selection changes captured from the row click
    let mut sort_requested = None; // For sort changes captured from the header click
    let mut scrolling_performed = false; // Flag to defer scroll state update

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);

        let mut header_params = TableHeaderParams {
            colors: &app.colors,
            sort_column: &tab_ref.sort_column,
            sort_order: &tab_ref.sort_order,
            on_sort: &mut |column| {
                sort_requested = Some(column);
            },
        };
        file_list::draw_table_header(ui, &mut header_params);

        // --- Draw Scrollable File List ---
        let available_height = height - ROW_HEIGHT;
        egui::ScrollArea::vertical()
            .id_salt("center_panel_list_scroll") // Use id_source for stable ID
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show_rows(
                ui,
                ROW_HEIGHT,
                filtered_entries.len(),
                |scroll_ui, row_range| {
                    let scrollbar_width = 6.0;
                    scroll_ui.set_min_width(width - scrollbar_width);
                    scroll_ui.set_max_width(width - scrollbar_width);

                    if filtered_entries.is_empty() {
                        scroll_ui.label("No matching entries found.");
                        return;
                    }

                    for row_index in row_range {
                        // Get the entry for the current visible row from the filtered list
                        let entry = &filtered_entries[row_index];

                        // Find the original index in the full `entries` list for selection state
                        let original_index = tab_ref
                            .entries
                            .iter()
                            .position(|e| e.path == entry.path)
                            .unwrap_or(usize::MAX); // Should always find

                        let is_selected = original_index == tab_ref.selected_index;
                        let is_in_selection = tab_ref.selected_entries.contains(&entry.path);

                        // Draw the row and get its response
                        let row_response = file_list::draw_entry_row(
                            scroll_ui,
                            file_list::EntryRowParams {
                                entry,
                                is_selected,
                                colors: &app.colors,
                                rename_mode: app.rename_mode && is_selected,
                                new_name: &mut app.new_name,
                                rename_focus: app.rename_focus && is_selected,
                                is_marked: is_in_selection,
                                is_bookmarked: app.bookmarks.contains(&entry.path),
                                search_query: current_search_query,
                            },
                        );

                        // Check for clicks to update selection state (captured outside)
                        if row_response.clicked() {
                            new_selected_index = Some(original_index);
                        }

                        // Handle scrolling to the selected item if needed for this row
                        if app.ensure_selected_visible && is_selected {
                            scroll_ui.scroll_to_rect(row_response.rect, Some(egui::Align::Center));
                            scrolling_performed = true;
                        }
                    }
                },
            );
    }); // End of ui.vertical closure. All borrows of `app` inside are released here.

    // --- Apply state changes captured from the UI closures AFTER drawing ---

    // Reset the scroll flag in app state only if scrolling actually happened
    if scrolling_performed {
        app.ensure_selected_visible = false;
    }

    // Handle sort request captured from the header closure
    if let Some(column) = sort_requested {
        // Borrow app mutably here - should be fine as UI closure is finished
        let tab = app.tab_manager.current_tab();
        tab.toggle_sort(column);
        tab.sort_entries();

        // Save sort preferences - requires immutable borrows followed by mutable config load/save
        let config_dir_override = app.config_dir_override.as_ref(); // Borrow immutably
        let mut config = config::load_config_with_override(config_dir_override);
        config.sort_preference = Some(SortPreference {
            column: tab.sort_column,
            order: tab.sort_order,
        });
        // Re-borrow immutably for save path
        if let Err(e) = config::save_config_with_override(&config, app.config_dir_override.as_ref())
        {
            eprintln!("Failed to save sort preferences: {}", e);
        }
    }

    // Handle selection change captured from the row closure
    if let Some(index) = new_selected_index {
        app.set_selection(index);
    }
}
