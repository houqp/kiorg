use egui::{Context, RichText};
use std::path::{Path, PathBuf};

use super::popup_utils::{show_confirm_popup, ConfirmResult};
use crate::config::colors::AppColors;

/// Confirmation state for the delete popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteConfirmState {
    /// Initial confirmation for any file or directory
    Initial,
    /// Second confirmation specifically for directories with recursive deletion
    RecursiveConfirm,
}

/// Result of the delete confirmation dialog
pub type DeleteConfirmResult = ConfirmResult;

/// Handle the delete confirmation process and show the popup
pub fn handle_delete_confirmation(
    ctx: &Context,
    show_delete_confirm: &mut bool,
    entries_to_delete: &[PathBuf],
    colors: &AppColors,
    state: &mut DeleteConfirmState,
) -> DeleteConfirmResult {
    if !*show_delete_confirm || entries_to_delete.is_empty() {
        return DeleteConfirmResult::None;
    }

    // Check if we're deleting a single entry or multiple entries
    let is_bulk_delete = entries_to_delete.len() > 1;

    // For single entry deletion, use the existing logic
    if !is_bulk_delete {
        let path = &entries_to_delete[0];

        match *state {
            DeleteConfirmState::Initial => {
                // Initial confirmation for any file or directory
                show_confirm_popup(
                    ctx,
                    "Delete Confirmation",
                    show_delete_confirm,
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(path.display().to_string());
                        });
                    },
                    "Delete (Enter)",
                    "Cancel (Esc)",
                )
            }
            DeleteConfirmState::RecursiveConfirm => {
                // Second confirmation specifically for directories
                show_confirm_popup(
                    ctx,
                    "Delete Confirmation",
                    show_delete_confirm,
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Are you SURE you want to delete");

                            // Highlight the filename with a background
                            ui.label(RichText::new(format!("{}", path.display())).strong());

                            ui.label("and ALL its contents recursively?");

                            ui.label(
                                RichText::new("This action cannot be undone!").color(colors.error),
                            );
                        });
                    },
                    "Delete (Enter)",
                    "Cancel (Esc)",
                )
            }
        }
    } else {
        // For bulk deletion, show a different confirmation dialog
        let has_directories = entries_to_delete.iter().any(|path| path.is_dir());

        match *state {
            DeleteConfirmState::Initial => {
                // Initial confirmation for bulk deletion
                show_confirm_popup(
                    ctx,
                    "Bulk Delete Confirmation",
                    show_delete_confirm,
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(format!(
                                "Delete {} selected items?",
                                entries_to_delete.len()
                            ));

                            // Show the first few entries as examples
                            let max_to_show = 5.min(entries_to_delete.len());
                            for path in entries_to_delete.iter().take(max_to_show) {
                                let name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path.display().to_string());

                                ui.label(name);
                            }

                            // If there are more entries than we're showing
                            if entries_to_delete.len() > max_to_show {
                                ui.label(format!(
                                    "...and {} more",
                                    entries_to_delete.len() - max_to_show
                                ));
                            }
                        });
                    },
                    "Delete (Enter)",
                    "Cancel (Esc)",
                )
            }
            DeleteConfirmState::RecursiveConfirm => {
                // Second confirmation specifically for bulk deletion with directories
                show_confirm_popup(
                    ctx,
                    "Bulk Delete Confirmation",
                    show_delete_confirm,
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Are you SURE you want to delete these items?");

                            if has_directories {
                                ui.label("Some selected items are directories and will be deleted recursively.");
                            }

                            ui.label(
                                RichText::new("This action cannot be undone!").color(colors.error),
                            );
                        });
                    },
                    "Delete (Enter)",
                    "Cancel (Esc)",
                )
            }
        }
    }
}

/// Helper function to perform the actual deletion
pub fn perform_delete(path: &Path) -> Result<(), String> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };

    result.map_err(|e| format!("Failed to delete: {e}"))
}

/// Handle the confirmation of deletion
pub fn confirm_delete(app: &mut crate::app::Kiorg) {
    if app.entries_to_delete.is_empty() {
        return;
    }

    // For bulk deletion (multiple entries)
    if app.entries_to_delete.len() > 1 {
        // Check if we're in the initial state and any of the entries is a directory
        if app.delete_popup_state == DeleteConfirmState::Initial {
            // For bulk deletion with directories, move to second confirmation
            app.delete_popup_state = DeleteConfirmState::RecursiveConfirm;
            return; // Return early without performing deletion
        }

        // Clone the entries to avoid borrowing issues
        let entries_to_delete = app.entries_to_delete.clone();

        // Delete each entry
        for path in entries_to_delete {
            if let Err(error) = perform_delete(&path) {
                app.notify_error(format!("Failed to delete {}: {}", path.display(), error));
            }
        }

        app.tab_manager.current_tab_mut().marked_entries.clear();
        app.refresh_entries();

        cancel_delete(app);
        return;
    }

    let path = &app.entries_to_delete[0];
    // Check if we're in the initial state and dealing with a directory
    if app.delete_popup_state == DeleteConfirmState::Initial && path.is_dir() {
        // For directories in initial state, move to second confirmation
        app.delete_popup_state = DeleteConfirmState::RecursiveConfirm;
        return; // Return early without performing deletion
    }

    // For files or directories in second confirmation state, proceed with deletion
    match perform_delete(path) {
        Ok(_) => {
            app.refresh_entries();
        }
        Err(error) => {
            app.notify_error(error);
        }
    }

    cancel_delete(app);
}

pub fn cancel_delete(app: &mut crate::app::Kiorg) {
    app.show_popup = None;
    app.entries_to_delete.clear(); // Clear the entries to delete
    app.delete_popup_state = DeleteConfirmState::Initial; // Reset delete popup state
}
