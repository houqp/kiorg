use egui::{Context, ProgressBar, RichText};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use super::popup_utils::{ConfirmResult, show_confirm_popup};
use super::window_utils::new_center_popup_window;
use crate::config::colors::AppColors;

/// Confirmation state for the delete popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteConfirmState {
    /// Initial confirmation for any file or directory
    Initial,
    /// Second confirmation specifically for directories with recursive deletion
    RecursiveConfirm,
}

/// Progress state for delete operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteProgressState {
    pub total_files: usize,
    pub current_file: usize,
    pub current_path: String,
    pub completed: bool,
    pub error: Option<String>,
}

/// Progress data containing state and receiver
pub struct DeleteProgressData {
    pub state: DeleteProgressState,
    pub receiver: std::sync::mpsc::Receiver<DeleteProgressUpdate>,
}

impl std::fmt::Debug for DeleteProgressData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeleteProgressData")
            .field("state", &self.state)
            .field("receiver", &"<receiver>")
            .finish()
    }
}

impl Clone for DeleteProgressData {
    fn clone(&self) -> Self {
        // This should never be called in practice since we don't need to clone the receiver
        panic!("DeleteProgressData cannot be cloned")
    }
}

impl PartialEq for DeleteProgressData {
    fn eq(&self, other: &Self) -> bool {
        // Compare only the state, not the receiver
        self.state == other.state
    }
}

impl Eq for DeleteProgressData {}

/// Progress update message sent from background thread
#[derive(Debug, Clone)]
pub enum DeleteProgressUpdate {
    Progress {
        current: usize,
        total: usize,
        current_path: String,
    },
    Completed,
    Error(String),
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
    if is_bulk_delete {
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
                                let name = path.file_name().map_or_else(
                                    || path.display().to_string(),
                                    |n| n.to_string_lossy().to_string(),
                                );

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
    } else {
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
    }
}

/// Helper function to perform the actual deletion
///
/// # Errors
///
/// Returns an error string if the deletion fails, either due to permission issues,
/// file system errors, or if the path doesn't exist.
pub fn perform_delete(path: &Path) -> Result<(), String> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };

    result.map_err(|e| format!("Failed to delete: {e}"))
}

/// Count total files to delete (for progress tracking)
fn count_files_to_delete(paths: &[PathBuf]) -> usize {
    let mut count = 0;
    for path in paths {
        if path.is_dir() {
            count += count_files_in_dir(path);
        } else {
            count += 1;
        }
    }
    count
}

/// Recursively count files in a directory
fn count_files_in_dir(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                count += count_files_in_dir(&entry.path());
            } else {
                count += 1;
            }
        }
        // Count the directory itself
        count += 1;
    }
    count
}

/// Delete directory recursively with progress updates
fn delete_dir_with_progress(
    dir: &Path,
    progress_sender: &mpsc::Sender<DeleteProgressUpdate>,
    current_file: &mut usize,
    total_files: usize,
) -> Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        delete_dir_with_progress(
                            &path,
                            progress_sender,
                            current_file,
                            total_files,
                        )?;
                    } else {
                        *current_file += 1;
                        let _ = progress_sender.send(DeleteProgressUpdate::Progress {
                            current: *current_file,
                            total: total_files,
                            current_path: path.display().to_string(),
                        });
                        std::fs::remove_file(&path).map_err(|e| {
                            format!("Failed to delete file {}: {e}", path.display())
                        })?;
                    }
                }
                Err(e) => return Err(format!("Failed to read directory entry: {e}")),
            }
        }
    }

    // Delete the directory itself
    *current_file += 1;
    let _ = progress_sender.send(DeleteProgressUpdate::Progress {
        current: *current_file,
        total: total_files,
        current_path: dir.display().to_string(),
    });

    std::fs::remove_dir(dir)
        .map_err(|e| format!("Failed to delete directory {}: {e}", dir.display()))
}

/// Handle progress popup UI
pub fn handle_delete_progress(ctx: &Context, app: &mut crate::app::Kiorg) {
    let mut should_close = false;
    let mut error_msg = None;

    // Check for progress updates
    if let Some(crate::app::PopupType::DeleteProgress(ref mut progress_data)) = app.show_popup {
        while let Ok(update) = progress_data.receiver.try_recv() {
            match update {
                DeleteProgressUpdate::Progress {
                    current,
                    total,
                    current_path,
                } => {
                    progress_data.state.current_file = current;
                    progress_data.state.total_files = total;
                    progress_data.state.current_path = current_path;
                    // Request repaint to update progress UI
                    ctx.request_repaint();
                }
                DeleteProgressUpdate::Completed => {
                    should_close = true;
                }
                DeleteProgressUpdate::Error(error) => {
                    should_close = true;
                    error_msg = Some(error);
                }
            }
        }
    }

    // Handle cleanup outside of the borrow
    if should_close {
        app.show_popup = None;

        app.tab_manager.current_tab_mut().marked_entries.clear();
        app.refresh_entries();

        if let Some(error) = error_msg {
            app.notify_error(error);
        }

        return;
    }

    // Show progress popup
    if let Some(crate::app::PopupType::DeleteProgress(ref progress_data)) = app.show_popup {
        let state = &progress_data.state;
        new_center_popup_window("Deletion Progress").show(ctx, |ui| {
            ui.set_min_width(400.0);

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                // Progress bar
                let progress = if state.total_files > 0 {
                    state.current_file as f32 / state.total_files as f32
                } else {
                    0.0
                };

                ui.add(ProgressBar::new(progress).desired_width(350.0));

                ui.add_space(10.0);

                // Status text
                ui.label(format!(
                    "{} / {} files",
                    state.current_file, state.total_files
                ));

                ui.add_space(5.0);

                // Current file being deleted
                if !state.current_path.is_empty() {
                    ui.label(format!("Deleting: {}", state.current_path));
                }

                ui.add_space(10.0);
            });
        });
    }
}

/// Handle the confirmation of deletion
pub fn confirm_delete(app: &mut crate::app::Kiorg) {
    let (state, entries_to_delete) =
        if let Some(crate::app::PopupType::Delete(ref state, ref entries)) = app.show_popup {
            (state.clone(), entries.clone())
        } else {
            return;
        };

    if entries_to_delete.is_empty() {
        return;
    }

    // For bulk deletion (multiple entries)
    if entries_to_delete.len() > 1 {
        // Check if we're in the initial state and any of the entries is a directory
        if state == DeleteConfirmState::Initial {
            // For bulk deletion with directories, move to second confirmation
            app.show_popup = Some(crate::app::PopupType::Delete(
                DeleteConfirmState::RecursiveConfirm,
                entries_to_delete,
            ));
            return; // Return early without performing deletion
        }
    } else {
        let path = &entries_to_delete[0];
        // Check if we're in the initial state and dealing with a directory
        if state == DeleteConfirmState::Initial && path.is_dir() {
            // For directories in initial state, move to second confirmation
            app.show_popup = Some(crate::app::PopupType::Delete(
                DeleteConfirmState::RecursiveConfirm,
                entries_to_delete,
            ));
            return; // Return early without performing deletion
        }
    }
    delete_async(app, entries_to_delete);
}

/// Start the async threaded deletion process
fn delete_async(app: &mut crate::app::Kiorg, entries_to_delete: Vec<PathBuf>) {
    let (tx, rx) = mpsc::channel();

    // Set up progress state
    let total_files = count_files_to_delete(&entries_to_delete);
    let progress_state = DeleteProgressState {
        total_files,
        current_file: 0,
        current_path: String::new(),
        completed: false,
        error: None,
    };

    let progress_data = DeleteProgressData {
        state: progress_state,
        receiver: rx,
    };

    // Switch to progress popup
    app.show_popup = Some(crate::app::PopupType::DeleteProgress(progress_data));

    // Clone entries for the thread
    thread::spawn(move || {
        let total_files = count_files_to_delete(&entries_to_delete);
        let mut current_file = 0;

        for path in entries_to_delete {
            let result = if path.is_dir() {
                delete_dir_with_progress(&path, &tx, &mut current_file, total_files)
            } else {
                current_file += 1;
                let _ = tx.send(DeleteProgressUpdate::Progress {
                    current: current_file,
                    total: total_files,
                    current_path: path.display().to_string(),
                });
                std::fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {e}"))
            };

            if let Err(error) = result {
                let _ = tx.send(DeleteProgressUpdate::Error(error));
                return;
            }
        }

        let _ = tx.send(DeleteProgressUpdate::Completed);
    });
}

pub fn cancel_delete(app: &mut crate::app::Kiorg) {
    app.show_popup = None;
}
