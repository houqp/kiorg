use egui::Ui;
use std::path::PathBuf;

use crate::app::Clipboard;
use crate::app::Kiorg;
use crate::config;
use crate::config::SortPreference;
use crate::ui::file_list::{self, ROW_HEIGHT, TableHeaderParams};
use crate::ui::popup::PopupType;
use crate::utils::file_operations;

// TODO: make this configurable
const PADDING_ROWS: usize = 3;

fn new_unique_path_name_for_paste(
    path: &std::path::Path,
    current_path: &std::path::Path,
) -> PathBuf {
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
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        new_path = current_path.join(format!("{stem}_{counter}{ext}"));
        counter += 1;
    }

    new_path
}

/// Handles clipboard paste operations (copy/cut)
/// Returns true if any operation was performed
pub fn handle_clipboard_operations(
    clipboard: &mut Option<Clipboard>,
    current_path: &std::path::Path,
    action_history: &mut crate::models::action_history::TabActionHistory,
    toasts: &mut crate::ui::egui_notify::Toasts,
) -> bool {
    match clipboard.take() {
        Some(Clipboard::Copy(paths)) => {
            let mut copy_operations = Vec::new();

            paths.iter().for_each(|path| {
                let new_path = new_unique_path_name_for_paste(path, current_path);
                // Handle copying differently based on whether it's a file or directory
                if path.is_dir() {
                    if let Err(e) = file_operations::copy_dir_recursively(path, &new_path) {
                        toasts.error(format!(
                            "Failed to copy directory {} to {}: {e}",
                            path.to_string_lossy(),
                            new_path.to_string_lossy()
                        ));
                    } else {
                        // Record successful copy operation
                        copy_operations.push(crate::models::action_history::CopyOperation {
                            source_path: path.clone(),
                            target_path: new_path,
                        });
                    }
                    return;
                }

                if let Err(e) = std::fs::copy(path, &new_path) {
                    toasts.error(format!(
                        "Failed to copy file {} to {}: {e}",
                        path.to_string_lossy(),
                        new_path.to_string_lossy()
                    ));
                } else {
                    // Record successful copy operation
                    copy_operations.push(crate::models::action_history::CopyOperation {
                        source_path: path.clone(),
                        target_path: new_path,
                    });
                }
            });

            // Record operations if any operations succeeded
            if !copy_operations.is_empty() {
                action_history.add_action(crate::models::action_history::ActionType::Copy {
                    operations: copy_operations,
                });
            }
        }
        Some(Clipboard::Cut(paths)) => {
            let mut move_operations = Vec::new();

            paths.iter().for_each(|path| {
                let new_path = new_unique_path_name_for_paste(path, current_path);
                if let Err(e) = std::fs::rename(path, &new_path) {
                    toasts.error(format!(
                        "Failed to move {} to {}: {e}",
                        path.to_string_lossy(),
                        new_path.to_string_lossy()
                    ));
                } else {
                    // Record successful move operation
                    move_operations.push(crate::models::action_history::MoveOperation {
                        source_path: path.clone(),
                        target_path: new_path,
                    });
                }
            });

            // Record operations if any operations succeeded
            if !move_operations.is_empty() {
                action_history.add_action(crate::models::action_history::ActionType::Move {
                    operations: move_operations,
                });
            }
        }
        _ => return false, // No clipboard operation to perform
    }

    true
}

fn scroll_by_filtered_index(
    mut scroll_area: egui::ScrollArea,
    filtered_index: usize,
    scroll_range: Option<&std::ops::Range<usize>>,
    spaced_row_height: f32,
    total_rows: usize,
) -> egui::ScrollArea {
    // Return early if scroll_range is None
    let Some(scroll_range) = scroll_range else {
        return scroll_area;
    };

    // scroll_area will always be lagging one cycle behind, i.e. it shows the view port before
    // current action has been processed
    // range end is exclusive not inclusive, so subtract 1
    let rows = scroll_range.end - scroll_range.start - 1;

    // NOTE: for some reason, the range provided by show_rows has 2 more rows than what's visible
    // on the viewport
    let rows_offset = 2;

    // where there are not enough entries to fill the viewport, just start from 0
    // if filtered_index + rows_offset < rows || rows_offset > rows {
    if filtered_index + PADDING_ROWS < rows || rows_offset > rows {
        return scroll_area.vertical_scroll_offset(0.0);
    }

    let scroll_page_end_index = filtered_index + PADDING_ROWS + rows_offset;
    // TODO: y offset is off for the last few rows, this is workaround to avoid
    // excessive scroll when we reach the end
    //
    // note that we also need to check for scroll_range.end so jumping to the
    // last page still works.
    if filtered_index < scroll_range.end && scroll_page_end_index >= total_rows {
        return scroll_area;
    }

    if filtered_index <= scroll_range.start + PADDING_ROWS {
        // scrolling up, reached start of view port + row padding
        // y for selected row
        let entry_y = filtered_index as f32 * spaced_row_height;
        // 3 rows before the selected row
        let scroll_y = spaced_row_height.mul_add(-(PADDING_ROWS as f32), entry_y);
        scroll_area = scroll_area.vertical_scroll_offset(scroll_y.max(0.0));
    } else if scroll_page_end_index >= scroll_range.end {
        // scrolling down, reached end of view port + row padding
        let entry_y = filtered_index as f32 * spaced_row_height; // y for selected row
        let scroll_y = entry_y
            // adjust by 3 rows after the selected row
            + spaced_row_height * PADDING_ROWS as f32
            // find y for first row in the viewport
            - ((rows - 1) as f32 * spaced_row_height);
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
    BulkDelete, // New action for bulk deletion
    OpenWith,   // New action for opening with custom command
}

/// Helper function to build the context menu items and return the chosen action.
/// Takes a boolean indicating if pasting is possible, if a file is selected, and if there are marked entries.
fn show_context_menu(
    ui: &mut Ui,
    can_paste: bool,
    has_selection: bool,
    has_marked_entries: bool,
) -> ContextMenuAction {
    let mut action = ContextMenuAction::None;

    if ui.button("Add new file/directory").clicked() {
        action = ContextMenuAction::Add;
        ui.close();
    }

    // File operations - only enabled when a file is selected
    ui.separator();

    if ui
        .add_enabled(has_selection, egui::Button::new("Rename"))
        .clicked()
    {
        action = ContextMenuAction::Rename;
        ui.close();
    }

    // Show bulk delete option when there are marked entries
    if has_marked_entries {
        // TODO: do we need to add enabled
        if ui.button("Delete all marked items").clicked() {
            action = ContextMenuAction::BulkDelete;
            ui.close();
        }
    } else if ui
        .add_enabled(has_selection, egui::Button::new("Delete"))
        .clicked()
    {
        action = ContextMenuAction::Delete;
        ui.close();
    }

    // Add "Open with" option - enabled for both files and directories
    if ui
        .add_enabled(has_selection, egui::Button::new("Open with..."))
        .clicked()
    {
        action = ContextMenuAction::OpenWith;
        ui.close();
    }

    ui.separator();

    if ui
        .add_enabled(has_selection, egui::Button::new("Copy"))
        .clicked()
    {
        action = ContextMenuAction::Copy;
        ui.close();
    }

    if ui
        .add_enabled(has_selection, egui::Button::new("Cut"))
        .clicked()
    {
        action = ContextMenuAction::Cut;
        ui.close();
    }

    // Use the passed boolean directly
    if ui
        .add_enabled(can_paste, egui::Button::new("Paste"))
        .clicked()
    {
        action = ContextMenuAction::Paste;
        ui.close();
    }

    action
}

/// Draws the center panel content.
pub fn draw(app: &mut Kiorg, ui: &mut Ui, width: f32, height: f32) {
    handle_file_drop(ui.ctx(), app);

    // --- State variables to capture changes from UI closures ---
    let mut new_selected_index = None; // For selection changes captured from the row click
    let mut sort_requested = None; // For sort changes captured from the header click
    let mut file_list_response = None; // To store the response for the background context menu
    let mut context_menu_action = ContextMenuAction::None; // To store the action from any context menu
    let mut double_clicked_path: Option<PathBuf> = None; // To store the path of a double-clicked entry
    let mut drag_started_source: Option<PathBuf> = None; // To store an item (file or directory) that started being dragged
    let mut drop_target_folder: Option<PathBuf> = None; // To store the folder where a file was dropped

    // prepare drag and drop state
    let is_drag_active = app.is_dragging();
    let primary_pointer_released = ui.ctx().input(|i| i.pointer.primary_released());

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
                // Get cached filtered entries or compute them if cache is empty
                // Get the current tab reference for reading
                let tab_ref = app.tab_manager.current_tab_ref();
                let filtered_entries = tab_ref.get_cached_filtered_entries();

                ui.set_min_height(available_height);
                ui.set_max_height(available_height); // Constrain the inner vertical area

                let mut scroll_area = egui::ScrollArea::vertical()
                    .id_salt(scroll_area_id) // Use id_salt for stable ID
                    .auto_shrink([false; 2])
                    .max_height(available_height); // Use available_height

                let total_rows = filtered_entries.len();

                let ui_spacing = ui.spacing().item_spacing.y;
                let spaced_row_height = ROW_HEIGHT + ui_spacing;

                if app.ensure_selected_visible {
                    if let Some(selected_entry) = tab_ref.selected_entry() {
                        // Find the position of the selected entry in the filtered list
                        if let Some(filtered_index) = filtered_entries
                            .iter()
                            .position(|(entry, _)| entry.meta.path == selected_entry.meta.path)
                        {
                            scroll_area = scroll_by_filtered_index(
                                scroll_area,
                                filtered_index,
                                app.scroll_range.as_ref(),
                                spaced_row_height,
                                total_rows,
                            );
                        }
                    }
                    app.ensure_selected_visible = false;
                }

                let current_dragged_file = app.get_dragged_file().cloned();
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
                    let selection_range = tab_ref.get_range_selection_range();

                    for row_index in row_range {
                        // Get the entry and original index for the current visible row from the filtered list
                        let (entry, original_index) = &filtered_entries[row_index];

                        let is_selected = *original_index == tab_ref.selected_index;

                        // Check if this entry is in the range selection range
                        let is_in_range_selection = if let Some((start, end)) = selection_range {
                            *original_index >= start && *original_index <= end
                        } else {
                            false
                        };
                        let is_marked = tab_ref.marked_entries.contains(&entry.meta.path)
                            || is_in_range_selection;

                        let being_opened = match app.files_being_opened.get(&entry.meta.path) {
                            Some(signal) => {
                                if signal.load(std::sync::atomic::Ordering::Relaxed) {
                                    true
                                } else {
                                    // trim hashmap to keep it lean
                                    app.files_being_opened.remove(&entry.meta.path);
                                    false
                                }
                            }
                            None => false,
                        };

                        // Check if this entry is in the clipboard as a cut or copy operation
                        let (is_in_cut_clipboard, is_in_copy_clipboard) = match &app.clipboard {
                            Some(Clipboard::Cut(paths)) => {
                                if paths.contains(&entry.meta.path) {
                                    (true, false)
                                } else {
                                    (false, false)
                                }
                            }
                            Some(Clipboard::Copy(paths)) => {
                                if paths.contains(&entry.meta.path) {
                                    (false, true)
                                } else {
                                    (false, false)
                                }
                            }
                            None => (false, false),
                        };

                        // Check if this entry is being dragged or is a drag target
                        let is_drag_source = is_drag_active
                            && current_dragged_file
                                .as_ref()
                                .map(|dragged| dragged == &entry.meta.path)
                                .unwrap_or(false);

                        // Draw the row and get its response
                        let row_response = file_list::draw_entry_row(
                            scroll_ui,
                            file_list::EntryRowParams {
                                entry,
                                is_selected,
                                colors: &app.colors,
                                is_marked,
                                is_bookmarked: app.bookmarks.contains(&entry.meta.path),
                                is_being_opened: being_opened,
                                is_in_cut_clipboard,
                                is_in_copy_clipboard,
                                is_drag_source,
                                is_drag_active,
                            },
                        );

                        // Check for clicks to update selection state (captured outside)
                        if row_response.clicked() {
                            new_selected_index = Some(*original_index);
                        }
                        // double_clicked() and clicked() return true at the same time
                        if row_response.double_clicked() {
                            // Check for double-clicks to navigate or open files
                            double_clicked_path = Some(entry.meta.path.clone());
                        } else if row_response.drag_started() {
                            // Start dragging files or directories
                            drag_started_source = Some(entry.meta.path.clone());
                        } else if is_drag_active
                            && !is_drag_source
                            && entry.is_dir
                            && primary_pointer_released
                            // Handle drop onto folders - check if mouse was released over this entry
                            && row_response.hovered()
                        {
                            drop_target_folder = Some(entry.meta.path.clone());
                        }

                        // --- Add Context Menu for Rows ---
                        row_response.context_menu(|menu_ui| {
                            new_selected_index = Some(*original_index);
                            // Capture the action, don't perform it yet
                            // Pass only the necessary booleans, not the whole app
                            let has_marked_entries = !tab_ref.marked_entries.is_empty();
                            context_menu_action = show_context_menu(
                                menu_ui,
                                app.clipboard.is_some(),
                                true,
                                has_marked_entries,
                            );
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
            let has_marked_entries = !app.tab_manager.current_tab_ref().marked_entries.is_empty();
            context_menu_action = show_context_menu(
                menu_ui,
                app.clipboard.is_some(),
                false, // No file is selected in background context menu
                has_marked_entries,
            );
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

    if let Some(item_path) = drag_started_source {
        app.start_drag(item_path);
    }

    if primary_pointer_released {
        if let Some(target_folder) = drop_target_folder {
            app.move_dragged_item_to_folder(target_folder);
        }
        app.end_drag();
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
    }

    if app.is_dragging() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    }

    // Handle context menu action captured from closures
    match context_menu_action {
        ContextMenuAction::Add => {
            app.show_popup = Some(PopupType::AddEntry(String::new()));
        }
        ContextMenuAction::Paste => {
            let current_tab = app.tab_manager.current_tab_mut();
            if handle_clipboard_operations(
                &mut app.clipboard,
                &current_tab.current_path,
                &mut current_tab.action_history,
                &mut app.toasts,
            ) {
                // Clear marked entries after successful paste operation
                app.tab_manager.current_tab_mut().marked_entries.clear();
                app.refresh_entries();
            }
        }
        ContextMenuAction::Rename => {
            app.rename_selected_entry();
        }
        ContextMenuAction::Delete => {
            app.delete_selected_entry();
        }
        ContextMenuAction::BulkDelete => {
            // Same as Delete, but explicitly for marked entries
            app.delete_selected_entry();
        }
        ContextMenuAction::Copy => {
            app.copy_selected_entries();
        }
        ContextMenuAction::Cut => {
            app.cut_selected_entries();
        }
        ContextMenuAction::OpenWith => {
            // Show the open with popup with an empty command string
            let tab = app.tab_manager.current_tab_ref();
            if let Some(_selected_entry) = tab.selected_entry() {
                // Now works for both files and directories
                app.show_popup = Some(PopupType::OpenWith);
            }
        }
        ContextMenuAction::None => {} // Do nothing
    }

    // Handle sort request captured from the header closure
    if let Some(column) = sort_requested {
        // Borrow app mutably here - should be fine as UI closure is finished
        app.tab_manager.toggle_sort(column);
        // Save sort preferences - requires immutable borrows followed by mutable config load/save
        app.config.sort_preference = Some(SortPreference {
            column: app.tab_manager.sort_column,
            order: app.tab_manager.sort_order,
        });
        // Re-borrow immutably for save path
        if let Err(e) =
            config::save_config_with_override(&app.config, app.config_dir_override.as_ref())
        {
            app.toasts
                .error(format!("Failed to save sort preferences: {e}"));
        }
    }
}

fn handle_file_drop(ctx: &egui::Context, app: &mut Kiorg) {
    ctx.input(|i| {
        if !i.raw.dropped_files.is_empty() {
            let mut dropped_paths = Vec::new();

            for dropped_file in &i.raw.dropped_files {
                let file_path = if let Some(path) = &dropped_file.path {
                    path
                } else {
                    continue;
                };
                dropped_paths.push(file_path.clone());
            }

            if !dropped_paths.is_empty() {
                app.show_popup = Some(PopupType::FileDrop(dropped_paths));
            }
        }
    });
}
