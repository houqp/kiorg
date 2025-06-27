use egui::Context;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf; // Removed unused Path

use super::window_utils::new_center_popup_window;
use crate::app::Kiorg;
use crate::config::get_kiorg_config_dir;
use crate::config::shortcuts::ShortcutAction;

// Get the full path to the bookmarks file
fn get_bookmarks_file_path(config_dir_override: Option<&PathBuf>) -> PathBuf {
    let mut config_dir = get_kiorg_config_dir(config_dir_override);

    if !config_dir.exists() {
        // Attempt to create the directory, ignore error if it fails
        let _ = fs::create_dir_all(&config_dir);
    }
    config_dir.push("bookmarks.txt");
    config_dir
}

// Save bookmarks to the config file
pub fn save_bookmarks(
    bookmarks: &[PathBuf],
    config_dir_override: Option<&PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let bookmarks_file = get_bookmarks_file_path(config_dir_override);
    // Ensure the directory exists before creating the file
    if let Some(parent_dir) = bookmarks_file.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }
    let mut file = fs::File::create(bookmarks_file)?;

    for bookmark in bookmarks {
        writeln!(file, "{}", bookmark.to_string_lossy())?;
    }

    Ok(())
}

// Load bookmarks from the config file
pub fn load_bookmarks(config_dir_override: Option<&PathBuf>) -> Vec<PathBuf> {
    let bookmarks_file = get_bookmarks_file_path(config_dir_override);
    if !bookmarks_file.exists() {
        return Vec::new();
    }

    match fs::File::open(&bookmarks_file) {
        Ok(file) => {
            let reader = BufReader::new(file);
            reader
                .lines()
                .map_while(Result::ok)
                .filter(|line| !line.trim().is_empty())
                .map(|line| PathBuf::from(line.trim()))
                .collect()
        }
        // Return empty vec on any error during file opening or reading
        Err(_) => Vec::new(),
    }
}

// --- End of new functions ---

pub enum BookmarkAction {
    Navigate(PathBuf),
    SaveBookmarks,
    None,
}

/// Helper function to display bookmarks in a grid layout
fn display_bookmarks_grid(
    ui: &mut egui::Ui,
    bookmarks: &[PathBuf],
    selected_index: usize,
    colors: &crate::config::colors::AppColors,
) -> (Option<PathBuf>, Option<PathBuf>) {
    let mut navigate_to_path = None;
    let mut remove_bookmark_path = None;
    let bg_selected = colors.bg_selected;

    egui::Grid::new("bookmarks_grid")
        .num_columns(2)
        .spacing([20.0, 2.0]) // 20px horizontal spacing, 2px vertical spacing
        .with_row_color(move |i, _| {
            if i == selected_index {
                Some(bg_selected)
            } else {
                None
            }
        })
        .show(ui, |ui| {
            for (i, bookmark) in bookmarks.iter().enumerate() {
                // Extract folder name and parent path
                let folder_name = bookmark
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let parent_path = bookmark
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let is_selected = i == selected_index;

                // Column 1: Folder name
                let folder_response = ui.colored_label(colors.fg_folder, &folder_name);

                // Column 2: Parent path
                let path_color = if is_selected {
                    colors.fg_selected
                } else {
                    colors.fg_light
                };

                let path_response = ui.colored_label(path_color, &parent_path);

                ui.end_row();

                // Combine responses for unified row clicking
                let combined_response = folder_response.union(path_response);

                // Show clickable hand cursor on hover and handle clicks
                let combined_response = if combined_response.hovered() {
                    combined_response.on_hover_cursor(egui::CursorIcon::PointingHand)
                } else {
                    combined_response
                };

                // Handle row click for navigation
                if combined_response.clicked() {
                    navigate_to_path = Some(bookmark.clone());
                }

                // Right-click context menu for the entire row
                combined_response.context_menu(|ui| {
                    if ui.button("Remove bookmark").clicked() {
                        remove_bookmark_path = Some(bookmark.clone());
                        ui.close_menu();
                    }
                });
            }
        });

    (navigate_to_path, remove_bookmark_path)
}

pub fn show_bookmark_popup(ctx: &Context, app: &mut Kiorg) -> BookmarkAction {
    // Extract the current selected index from the popup type, or return early if not showing bookmarks
    let current_index = match &app.show_popup {
        Some(crate::app::PopupType::Bookmarks(index)) => *index,
        _ => return BookmarkAction::None,
    };

    let mut current_index = if app.bookmarks.is_empty() {
        0
    } else {
        current_index.min(app.bookmarks.len() - 1)
    };

    // Handle keyboard navigation using shortcuts

    let mut remove_bookmark_path = None;

    // Check for shortcut actions based on input
    let action = app.get_shortcut_action_from_input(ctx, false);

    if let Some(action) = action {
        match action {
            ShortcutAction::Exit => {
                app.show_popup = None;
                return BookmarkAction::None;
            }
            ShortcutAction::DeleteEntry => {
                if !app.bookmarks.is_empty() {
                    remove_bookmark_path = Some(app.bookmarks[current_index].clone());
                }
            }
            _ => {} // Other actions will be handled below in the window
        }
    }

    let mut navigate_to_path = None;

    // Create a temporary boolean for the window's open state
    let mut window_open = true;

    if let Some(response) = new_center_popup_window("Bookmarks")
        .default_pos(ctx.screen_rect().center()) // Position at screen center
        .open(&mut window_open)
        .show(ctx, |ui| {
            if app.bookmarks.is_empty() {
                ui.label("No bookmarks yet. Use 'b' to bookmark folders.");
                return;
            }

            // Handle keyboard navigation
            let action = app.get_shortcut_action_from_input(ctx, false);
            if let Some(action) = action {
                match action {
                    ShortcutAction::MoveDown => {
                        if !app.bookmarks.is_empty() {
                            current_index = (current_index + 1).min(app.bookmarks.len() - 1);
                        }
                    }
                    ShortcutAction::MoveUp => {
                        current_index = current_index.saturating_sub(1);
                    }
                    ShortcutAction::OpenDirectoryOrFile | ShortcutAction::OpenDirectory => {
                        if !app.bookmarks.is_empty() {
                            navigate_to_path = Some(app.bookmarks[current_index].clone());
                        }
                    }
                    _ => {} // Other actions already handled above
                }
            }

            // Display bookmarks in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                let (click_navigate, context_menu_remove) =
                    display_bookmarks_grid(ui, &app.bookmarks, current_index, &app.colors);
                if let Some(path) = click_navigate {
                    navigate_to_path = Some(path);
                }
                if let Some(path) = context_menu_remove {
                    remove_bookmark_path = Some(path);
                }
            });
        })
    {
        // Return appropriate action based on what happened
        let mut action = BookmarkAction::None;

        // If we need to navigate, return the path
        if let Some(path) = navigate_to_path {
            action = BookmarkAction::Navigate(path);
            app.show_popup = None; // Close popup when navigating
        } else {
            // If we need to remove a bookmark, do it now
            if let Some(path) = remove_bookmark_path {
                app.bookmarks.retain(|p| p != &path);
                action = BookmarkAction::SaveBookmarks;
            }

            // Update the popup state with the current index
            if window_open && !response.response.clicked_elsewhere() {
                app.show_popup = Some(crate::app::PopupType::Bookmarks(current_index));
            } else {
                app.show_popup = None;
            }
        }

        action
    } else {
        // Window was closed
        app.show_popup = None;
        BookmarkAction::None
    }
}

/// Toggle bookmark status for the given path
///
/// Returns true if the bookmark was added, false if it was removed
pub fn toggle_bookmark(app: &mut Kiorg) {
    let bookmarks = &mut app.bookmarks;
    let tab = app.tab_manager.current_tab_mut();
    let Some(selected_entry) = tab.selected_entry() else {
        return;
    };

    // Only allow bookmarking directories, not files
    if selected_entry.is_dir {
        let path = selected_entry.path.clone();

        // Toggle bookmark status
        if bookmarks.contains(&path) {
            bookmarks.retain(|p| p != &path);
        } else {
            bookmarks.push(path);
        }

        // Save bookmarks to config file
        if let Err(e) = save_bookmarks(bookmarks, app.config_dir_override.as_ref()) {
            app.notify_error(format!("Failed to save bookmarks: {e}"));
        }
    } else {
        app.notify_error("Bookmarks can only be applied to directories, not files".to_string());
    }
}
