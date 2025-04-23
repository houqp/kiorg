use egui::Ui;
use std::path::PathBuf;

use crate::app::Kiorg;
use crate::config;
use crate::config::SortPreference;
use crate::ui::file_list::{self, TableHeaderParams, ROW_HEIGHT};

// TODO: make this configurable
const PADDING_ROWS: usize = 3;

/// Handles clipboard paste operations (copy/cut)
/// Returns true if any operation was performed
pub fn handle_clipboard_operations(
    clipboard: &mut Option<(Vec<PathBuf>, bool)>,
    current_path: &std::path::Path,
) -> bool {
    let (paths, is_cut) = match clipboard.take() {
        Some((paths, is_cut)) => (paths, is_cut),
        _ => return false, // No clipboard operation to perform
    };

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
}

fn scroll_by_filtered_index(
    ui: &mut Ui,
    mut scroll_area: egui::ScrollArea,
    filtered_index: usize,
    scroll_range: &Option<std::ops::Range<usize>>,
) -> egui::ScrollArea {
    // Return early if scroll_range is None
    let scroll_range = match scroll_range {
        Some(range) => range,
        None => return scroll_area,
    };

    // scroll_area will always be lagging one cycle behind, i.e. it shows the view port before
    // current action has been processed
    let rows = scroll_range.end - scroll_range.start;

    // NOTE: for some reason, the range provided by show_rows has 2 more rows than what's visible
    // on the viewport
    let rows_offset = 2;

    // where there are not enough entries to fill the viewport, just start from 0
    if filtered_index + rows_offset < rows || rows_offset > rows {
        return scroll_area.vertical_scroll_offset(0.0);
    }

    let ui_spacing = ui.spacing().item_spacing.y;

    if filtered_index < scroll_range.start + PADDING_ROWS {
        // reached start of view port + row padding
        let entry_y = filtered_index as f32 * (ROW_HEIGHT + ui_spacing); // y for selected row
        let scroll_y = entry_y - (ROW_HEIGHT + ui_spacing) * 3.0; // 3 rows before the selected row
        scroll_area = scroll_area.vertical_scroll_offset(scroll_y.max(0.0));
    } else if filtered_index + PADDING_ROWS + rows_offset >= scroll_range.end {
        // reached end of view port + row padding
        let entry_y = filtered_index as f32 * (ROW_HEIGHT + ui_spacing); // y for selected row
        let scroll_y = entry_y
            // adjust by 3 rows after the selected row
            + (ROW_HEIGHT + ui_spacing) * 3.0
            // add a little bitmore spacing so the text is not literally touching the bottom edge
            + (ui_spacing * 3.0 )
            // find y for first row in the viewport
            - ((rows-rows_offset) as f32 * (ROW_HEIGHT + ui_spacing));
        scroll_area = scroll_area.vertical_scroll_offset(scroll_y.max(0.0));
    }

    scroll_area
}

/// Enum to represent actions triggered by the context menu.
#[derive(Clone, Copy, PartialEq)]
enum ContextMenuAction {
    None,
    Add,
    Paste,
    Rename,
    Delete,
    Copy,
    Cut,
}

/// Helper function to build the context menu items and return the chosen action.
/// Takes a boolean indicating if pasting is possible and if a file is selected.
fn show_context_menu(ui: &mut Ui, can_paste: bool, has_selection: bool) -> ContextMenuAction {
    let mut action = ContextMenuAction::None;

    if ui.button("Add new file/directory (a)").clicked() {
        action = ContextMenuAction::Add;
        ui.close_menu();
    }

    // File operations - only enabled when a file is selected
    ui.separator();

    if ui
        .add_enabled(has_selection, egui::Button::new("Rename (r)"))
        .clicked()
    {
        action = ContextMenuAction::Rename;
        ui.close_menu();
    }

    if ui
        .add_enabled(has_selection, egui::Button::new("Delete (d)"))
        .clicked()
    {
        action = ContextMenuAction::Delete;
        ui.close_menu();
    }

    ui.separator();

    if ui
        .add_enabled(has_selection, egui::Button::new("Copy (yy)"))
        .clicked()
    {
        action = ContextMenuAction::Copy;
        ui.close_menu();
    }

    if ui
        .add_enabled(has_selection, egui::Button::new("Cut (x)"))
        .clicked()
    {
        action = ContextMenuAction::Cut;
        ui.close_menu();
    }

    // Use the passed boolean directly
    if ui
        .add_enabled(can_paste, egui::Button::new("Paste (p)"))
        .clicked()
    {
        action = ContextMenuAction::Paste;
        ui.close_menu();
    }

    action
}

/// Draws the center panel content.
pub fn draw(app: &mut Kiorg, ui: &mut Ui, width: f32, height: f32) {
    let tab_ref = app.tab_manager.current_tab_ref(); // Use a different name to avoid confusion
    let current_search_query = &app.search_bar.query;

    // Get filtered entries - needs tab_ref and search query
    // TODO: store filtered entries in tab_ref to avoid re-filtering on every draw
    let filtered_entries = tab_ref.get_filtered_entries(current_search_query);

    // --- State variables to capture changes from UI closures ---
    let mut new_selected_index = None; // For selection changes captured from the row click
    let mut sort_requested = None; // For sort changes captured from the header click
    let mut file_list_response = None; // To store the response for the background context menu
    let mut context_menu_action = ContextMenuAction::None; // To store the action from any context menu
    let mut double_clicked_path: Option<PathBuf> = None; // To store the path of a double-clicked entry

    ui.vertical(|ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        ui.set_min_height(height);
        ui.set_max_height(height);

        let mut header_params = TableHeaderParams {
            colors: &app.colors,
            sort_column: &app.tab_manager.sort_column,
            sort_order: &app.tab_manager.sort_order,
            on_sort: &mut |column| {
                sort_requested = Some(column);
            },
        };
        let header_resp = file_list::draw_table_header(ui, &mut header_params);

        // --- Draw Scrollable File List within its own container for context menu ---
        let available_height = height - header_resp.rect.height();
        let scroll_area_id = ui.id().with("center_panel_list_scroll");

        // Use a containing layout for the scroll area to capture interactions
        let inner_response = ui
            .vertical(|ui| {
                ui.set_min_height(available_height);
                ui.set_max_height(available_height); // Constrain the inner vertical area

                let mut scroll_area = egui::ScrollArea::vertical()
                    .id_salt(scroll_area_id) // Use id_salt for stable ID
                    .auto_shrink([false; 2])
                    .max_height(available_height); // Use available_height

                let total_rows = filtered_entries.len();

                if app.ensure_selected_visible {
                    if let Some(selected_entry) = tab_ref.selected_entry() {
                        let filtered_index = filtered_entries
                            .iter()
                            .enumerate()
                            .find(|(_, entry)| entry.path == selected_entry.path)
                            .expect("selected entry not in filtered list")
                            .0;

                        println!(
                            "scrolling range: {:?}, filtered_index: {:?}",
                            app.scroll_range, filtered_index
                        );
                        scroll_area = scroll_by_filtered_index(
                            ui,
                            scroll_area,
                            filtered_index,
                            &app.scroll_range,
                        );
                    }
                    app.ensure_selected_visible = false;
                }

                // Draw the rows within the scroll area
                scroll_area.show_rows(ui, ROW_HEIGHT, total_rows, |scroll_ui, row_range| {
                    // Calculate width considering potential scrollbar
                    // Use available_width which accounts for parent layouts and scrollbars automatically
                    let available_width = scroll_ui.available_width();
                    scroll_ui.set_min_width(available_width);

                    if filtered_entries.is_empty() {
                        scroll_ui.label("No matching entries found.");
                        return;
                    }
                    app.scroll_range = Some(row_range.clone());

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

                        // Draw the row and get its response and potential new name
                        // Destructure the tuple returned by draw_entry_row
                        let row_response = file_list::draw_entry_row(
                            scroll_ui,
                            file_list::EntryRowParams {
                                entry,
                                is_selected,
                                colors: &app.colors,
                                rename_mode: app.rename_mode && is_selected,
                                new_name: &mut app.new_name,
                                is_marked: is_in_selection,
                                is_bookmarked: app.bookmarks.contains(&entry.path),
                                search_query: current_search_query,
                            },
                        );

                        // Check for clicks to update selection state (captured outside)
                        if row_response.clicked() {
                            new_selected_index = Some(original_index);
                        }

                        // Check for double-clicks to navigate or open files
                        if row_response.double_clicked() {
                            double_clicked_path = Some(entry.path.clone());
                        }

                        // --- Add Context Menu for Rows ---
                        row_response.context_menu(|menu_ui| {
                            new_selected_index = Some(original_index);
                            // Capture the action, don't perform it yet
                            // Pass only the necessary booleans, not the whole app
                            context_menu_action =
                                show_context_menu(menu_ui, app.clipboard.is_some(), true);
                        });
                    } // End row loop
                }); // End show_rows
            })
            .response; // End inner ui.vertical and get its response

        // Store the response of the inner container for context menu handling outside
        file_list_response = Some(inner_response);
    }); // End of main ui.vertical closure. All borrows of `app` inside are released here.

    // --- Context Menu Logic for Background Area (using the stored response) ---
    if let Some(response) = file_list_response {
        response.context_menu(|menu_ui| {
            // Capture the action, don't perform it yet
            // Pass only the necessary booleans, not the whole app
            // For background context menu, no file is selected
            context_menu_action = show_context_menu(menu_ui, app.clipboard.is_some(), false);
        });
    }

    // --- Apply state changes captured from the UI closures AFTER drawing ---

    // Handle selection change captured from the row closure
    // NOTE: important to update the index before handle the context menu action
    // so it's acting on the current selected entry
    if let Some(index) = new_selected_index {
        app.set_selection(index);
    }

    // Handle double-click navigation or file opening
    if let Some(path) = double_clicked_path {
        if path.is_dir() {
            app.navigate_to_dir(path);
        } else if path.is_file() {
            app.open_file(path);
        }
    }

    // Handle context menu action captured from closures
    match context_menu_action {
        ContextMenuAction::Add => {
            app.add_mode = true;
            app.add_focus = true;
        }
        ContextMenuAction::Paste => {
            let current_path = &app.tab_manager.current_tab_ref().current_path;
            if handle_clipboard_operations(&mut app.clipboard, current_path) {
                app.refresh_entries();
            }
        }
        ContextMenuAction::Rename => {
            app.rename_selected_entry();
        }
        ContextMenuAction::Delete => {
            app.delete_selected_entry();
        }
        ContextMenuAction::Copy => {
            app.copy_selected_entries();
        }
        ContextMenuAction::Cut => {
            app.cut_selected_entries();
        }
        ContextMenuAction::None => {} // Do nothing
    }

    // Handle sort request captured from the header closure
    if let Some(column) = sort_requested {
        // Borrow app mutably here - should be fine as UI closure is finished
        app.tab_manager.toggle_sort(column);

        // Save sort preferences - requires immutable borrows followed by mutable config load/save
        let config_dir_override = app.config_dir_override.as_ref(); // Borrow immutably
        let mut config = config::load_config_with_override(config_dir_override);
        config.sort_preference = Some(SortPreference {
            column: app.tab_manager.sort_column,
            order: app.tab_manager.sort_order,
        });
        // Re-borrow immutably for save path
        if let Err(e) = config::save_config_with_override(&config, app.config_dir_override.as_ref())
        {
            eprintln!("Failed to save sort preferences: {}", e);
        }
    }
}
