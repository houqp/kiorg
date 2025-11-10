use egui::Context;
use std::fs;
use std::io;
use std::path::PathBuf;

use super::PopupType;
use super::window_utils::new_center_popup_window;
use crate::app::Kiorg;
use crate::config::shortcuts::ShortcutAction;

pub enum VolumeAction {
    Navigate(PathBuf),
    None,
}

/// Get list of mounted volumes from /Volumes directory
fn get_mounted_volumes() -> Result<Vec<PathBuf>, io::Error> {
    let volumes_path = PathBuf::from("/Volumes");

    if !volumes_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "/Volumes directory does not exist",
        ));
    }

    let entries = fs::read_dir(&volumes_path)?;
    let volumes: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    Ok(volumes)
}

/// Helper function to display volumes in a grid layout
fn display_volumes_grid(
    ui: &mut egui::Ui,
    volumes: &[PathBuf],
    selected_index: usize,
    colors: &crate::config::colors::AppColors,
) -> Option<PathBuf> {
    let mut navigate_to_path = None;
    let bg_selected = colors.bg_selected;

    egui::Grid::new("volumes_grid")
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
            for (i, volume) in volumes.iter().enumerate() {
                // Extract volume name
                let volume_name = volume
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let volume_path = volume.to_string_lossy().to_string();

                let is_selected = i == selected_index;

                // Column 1: Volume name
                let volume_response = ui.colored_label(colors.fg_folder, &volume_name);

                // Column 2: Volume path
                let path_color = if is_selected {
                    colors.fg_selected
                } else {
                    colors.fg_light
                };

                let path_response = ui.colored_label(path_color, &volume_path);

                ui.end_row();

                // Combine responses for unified row clicking
                let combined_response = volume_response.union(path_response);

                // Show clickable hand cursor on hover and handle clicks
                let combined_response = if combined_response.hovered() {
                    combined_response.on_hover_cursor(egui::CursorIcon::PointingHand)
                } else {
                    combined_response
                };

                // Handle row click for navigation
                if combined_response.clicked() {
                    navigate_to_path = Some(volume.clone());
                }
            }
        });

    navigate_to_path
}

pub fn show_volumes_popup(ctx: &Context, app: &mut Kiorg) -> VolumeAction {
    // Extract the current selected index from the popup type, or return early if not showing volumes
    let current_index = match &app.show_popup {
        Some(PopupType::Volumes(index)) => *index,
        _ => return VolumeAction::None,
    };

    // Get current volumes
    let volumes = match get_mounted_volumes() {
        Ok(vols) => vols,
        Err(e) => {
            app.notify_error(format!("Failed to read mounted volumes: {}", e));
            app.show_popup = None;
            return VolumeAction::None;
        }
    };

    let mut current_index = if volumes.is_empty() {
        0
    } else {
        current_index.min(volumes.len() - 1)
    };

    let action = app.get_shortcut_action_from_input(ctx, false);
    // Handle keyboard navigation using shortcuts
    // Check for shortcut actions based on input
    if let Some(ShortcutAction::Exit) = action {
        app.show_popup = None;
        return VolumeAction::None;
    }

    let mut navigate_to_path = None;

    // Create a temporary boolean for the window's open state
    let mut window_open = true;

    if let Some(response) = new_center_popup_window("Mounted Volumes")
        .default_pos(ctx.content_rect().center()) // Position at screen center
        .open(&mut window_open)
        .show(ctx, |ui| {
            if volumes.is_empty() {
                ui.label("No mounted volumes found in /Volumes directory");
                return;
            }

            // Handle keyboard navigation
            if let Some(action) = action {
                match action {
                    ShortcutAction::MoveDown => {
                        if !volumes.is_empty() {
                            current_index = (current_index + 1).min(volumes.len() - 1);
                        }
                    }
                    ShortcutAction::MoveUp => {
                        current_index = current_index.saturating_sub(1);
                    }
                    ShortcutAction::OpenDirectoryOrFile | ShortcutAction::OpenDirectory => {
                        if !volumes.is_empty() {
                            navigate_to_path = Some(volumes[current_index].clone());
                        }
                    }
                    _ => {} // Other actions already handled above
                }
            }

            // Display volumes in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                let click_navigate = display_volumes_grid(ui, &volumes, current_index, &app.colors);
                if let Some(path) = click_navigate {
                    navigate_to_path = Some(path);
                }
            });
        })
    {
        // Return appropriate action based on what happened
        let mut action = VolumeAction::None;

        // If we need to navigate, return the path
        if let Some(path) = navigate_to_path {
            action = VolumeAction::Navigate(path);
            app.show_popup = None; // Close popup when navigating
        } else {
            // Update the popup state with the current index
            if window_open && !response.response.clicked_elsewhere() {
                app.show_popup = Some(PopupType::Volumes(current_index));
            } else {
                app.show_popup = None;
            }
        }

        action
    } else {
        // Window was closed
        app.show_popup = None;
        VolumeAction::None
    }
}
