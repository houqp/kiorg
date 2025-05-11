use egui::Context;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf; // Removed unused Path

use super::window_utils::new_center_popup_window;
use crate::app::Kiorg;
use crate::config::get_kiorg_config_dir;

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
) -> (Option<PathBuf>, Option<PathBuf>) {
    let mut navigate_to_path = None;
    let mut remove_bookmark_path = None;

    egui::Grid::new("bookmarks_grid")
        .num_columns(2)
        .spacing([20.0, 6.0]) // Space between columns and rows
        .striped(true) // Alternate row background for better readability
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

                // First column: Folder name with selectable label
                let folder_response =
                    ui.add(egui::SelectableLabel::new(i == selected_index, folder_name));

                // Second column: Parent path
                let path_color = if i == selected_index {
                    ui.visuals().strong_text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                let path_response = ui.colored_label(path_color, parent_path);
                ui.end_row();

                // Combined response for click handling
                let response = folder_response.union(path_response);

                if response.clicked() {
                    navigate_to_path = Some(bookmark.clone());
                }

                // Right-click context menu for removing bookmarks
                response.context_menu(|ui| {
                    if ui.button("Remove bookmark").clicked() {
                        remove_bookmark_path = Some(bookmark.clone());
                        ui.close_menu();
                    }
                });
            }
        });

    (navigate_to_path, remove_bookmark_path)
}

pub fn show_bookmark_popup(
    ctx: &Context,
    show_bookmarks: &mut bool,
    bookmarks: &mut Vec<PathBuf>,
    selected_index: &mut usize,
) -> BookmarkAction {
    if !*show_bookmarks {
        return BookmarkAction::None;
    }

    let mut show_popup = *show_bookmarks;
    let mut current_index = *selected_index;

    // Ensure index is valid
    if !bookmarks.is_empty() {
        current_index = current_index.min(bookmarks.len() - 1);
    } else {
        current_index = 0;
    }

    // Handle keyboard navigation for closing the popup
    if ctx.input(|i| i.key_pressed(egui::Key::Q) || i.key_pressed(egui::Key::Escape)) {
        show_popup = false;
    }

    // Handle keyboard shortcut for deleting bookmarks
    let mut remove_bookmark_path = None;
    if ctx.input(|i| i.key_pressed(egui::Key::D)) && !bookmarks.is_empty() {
        remove_bookmark_path = Some(bookmarks[current_index].clone());
    }

    let mut navigate_to_path = None;

    if let Some(response) = new_center_popup_window("Bookmarks")
        .default_pos(ctx.screen_rect().center()) // Position at screen center
        .open(&mut show_popup)
        .show(ctx, |ui| {
            if bookmarks.is_empty() {
                ui.label("No bookmarks yet. Use 'b' to bookmark folders.");
                return;
            }

            // Handle keyboard navigation
            if ctx.input(|i| i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown)) {
                if !bookmarks.is_empty() {
                    current_index = (current_index + 1).min(bookmarks.len() - 1);
                }
            } else if ctx
                .input(|i| i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp))
            {
                current_index = current_index.saturating_sub(1);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::L))
                && !bookmarks.is_empty()
            {
                navigate_to_path = Some(bookmarks[current_index].clone());
            }

            // Display bookmarks in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                let (click_navigate, context_menu_remove) =
                    display_bookmarks_grid(ui, bookmarks, current_index);
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
            show_popup = false;
        }

        // If we need to remove a bookmark, do it now
        if let Some(path) = remove_bookmark_path {
            bookmarks.retain(|p| p != &path);
            action = BookmarkAction::SaveBookmarks;
        }

        // Update the selected_index with our current_index
        *selected_index = current_index;

        *show_bookmarks = show_popup && !response.response.clicked_elsewhere();
        action
    } else {
        *show_bookmarks = show_popup;
        *selected_index = current_index;
        BookmarkAction::None
    }
}

/// Toggle bookmark status for the given path
///
/// Returns true if the bookmark was added, false if it was removed
pub fn toggle_bookmark(app: &mut Kiorg) {
    let bookmarks = &mut app.bookmarks;
    let tab = app.tab_manager.current_tab_mut();
    let selected_entry = match tab.selected_entry() {
        Some(entry) => entry,
        None => return, // No selected entry
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
            app.notify_error(format!("Failed to save bookmarks: {}", e));
        }
    }
}
