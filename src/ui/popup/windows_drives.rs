use egui::Context;
use std::io;
use std::path::PathBuf;

use super::PopupType;
use super::window_utils::show_center_popup_window;
use crate::app::Kiorg;
use crate::config::shortcuts::ShortcutAction;

use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;

pub enum DriveAction {
    Navigate(PathBuf),
    None,
}

/// Get list of available drives on Windows
fn get_available_drives() -> Result<Vec<PathBuf>, std::io::Error> {
    // Get logical drive strings using Windows API
    let drives_mask = unsafe { GetLogicalDrives() };

    if drives_mask == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get logical drives",
        ));
    }

    let mut drives = Vec::new();
    for i in 0..26 {
        if (drives_mask & (1 << i)) != 0 {
            let drive_letter = (b'A' + i) as char;
            let drive_path = format!("{}:\\", drive_letter);
            drives.push(PathBuf::from(drive_path));
        }
    }

    Ok(drives)
}

/// Helper function to display drives in a grid layout
fn display_drives_grid(
    ui: &mut egui::Ui,
    drives: &[PathBuf],
    selected_index: usize,
    colors: &crate::config::colors::AppColors,
) -> Option<PathBuf> {
    let mut navigate_to_path = None;
    let bg_selected = colors.bg_selected;

    egui::Grid::new("drives_grid")
        .num_columns(1)
        .spacing([20.0, 2.0]) // 20px horizontal spacing, 2px vertical spacing
        .with_row_color(move |i, _| {
            if i == selected_index {
                Some(bg_selected)
            } else {
                None
            }
        })
        .show(ui, |ui| {
            for drive in drives.iter() {
                let drive_path = drive.to_string_lossy().to_string();

                // Drive path
                let drive_response = ui.colored_label(colors.fg_folder, &drive_path);

                ui.end_row();

                // Show clickable hand cursor on hover and handle clicks
                let drive_response = if drive_response.hovered() {
                    drive_response.on_hover_cursor(egui::CursorIcon::PointingHand)
                } else {
                    drive_response
                };

                // Handle row click for navigation
                if drive_response.clicked() {
                    navigate_to_path = Some(drive.clone());
                }
            }
        });

    navigate_to_path
}

pub fn show_drives_popup(ctx: &Context, app: &mut Kiorg) -> DriveAction {
    // Extract the current selected index from the popup type, or return early if not showing drives
    let current_index = match &app.show_popup {
        Some(PopupType::WindowsDrives(index)) => *index,
        _ => return DriveAction::None,
    };

    // Get current drives
    let drives = match get_available_drives() {
        Ok(drives) => drives,
        Err(e) => {
            app.notify_error(format!("Failed to read available drives: {}", e));
            app.show_popup = None;
            return DriveAction::None;
        }
    };

    let mut current_index = if drives.is_empty() {
        0
    } else {
        current_index.min(drives.len() - 1)
    };

    let action = app.get_shortcut_action_from_input(ctx);
    // Handle keyboard navigation using shortcuts
    // Check for shortcut actions based on input
    if let Some(ShortcutAction::Exit) = action {
        app.show_popup = None;
        return DriveAction::None;
    }

    let mut navigate_to_path = None;

    // Create a temporary boolean for the window's open state
    let mut window_open = true;

    if let Some(response) =
        show_center_popup_window("Available Drives", ctx, &mut window_open, |ui| {
            if drives.is_empty() {
                ui.label("No drives found");
                return;
            }

            // Handle keyboard navigation
            if let Some(action) = action {
                match action {
                    ShortcutAction::MoveDown => {
                        if !drives.is_empty() {
                            current_index = (current_index + 1).min(drives.len() - 1);
                        }
                    }
                    ShortcutAction::MoveUp => {
                        current_index = current_index.saturating_sub(1);
                    }
                    ShortcutAction::OpenDirectoryOrFile | ShortcutAction::OpenDirectory => {
                        if !drives.is_empty() {
                            navigate_to_path = Some(drives[current_index].clone());
                        }
                    }
                    _ => {} // Other actions already handled above
                }
            }

            // Display drives in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                let click_navigate = display_drives_grid(ui, &drives, current_index, &app.colors);
                if let Some(path) = click_navigate {
                    navigate_to_path = Some(path);
                }
            });
        })
    {
        // Return appropriate action based on what happened
        let mut action = DriveAction::None;

        // If we need to navigate, return the path
        if let Some(path) = navigate_to_path {
            action = DriveAction::Navigate(path);
            app.show_popup = None; // Close popup when navigating
        } else {
            // Update the popup state with the current index
            if window_open && !response.response.clicked_elsewhere() {
                app.show_popup = Some(PopupType::WindowsDrives(current_index));
            } else {
                app.show_popup = None;
            }
        }

        action
    } else {
        // Window was closed
        app.show_popup = None;
        DriveAction::None
    }
}
