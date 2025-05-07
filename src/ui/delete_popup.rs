use egui::{Context, RichText};
use std::path::{Path, PathBuf};

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

/// Result of the delete confirmation dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteConfirmResult {
    /// User confirmed the deletion
    Confirm,
    /// User canceled the deletion
    Cancel,
    /// No action taken yet
    None,
}

/// Handle the delete confirmation process and show the popup
pub fn handle_delete_confirmation(
    ctx: &Context,
    show_delete_confirm: &mut bool,
    entry_to_delete: &Option<PathBuf>,
    colors: &AppColors,
    state: &mut DeleteConfirmState,
) -> DeleteConfirmResult {
    if !*show_delete_confirm || entry_to_delete.is_none() {
        return DeleteConfirmResult::None;
    }

    let path = entry_to_delete.as_ref().unwrap();
    let mut show_popup = *show_delete_confirm;
    let mut result = DeleteConfirmResult::None;

    if let Some(response) = new_center_popup_window("Delete Confirmation")
        .open(&mut show_popup)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                let (confirm_clicked, cancel_clicked) = match *state {
                    DeleteConfirmState::Initial => {
                        // First confirmation for any file or directory
                        ui.label(format!("Delete {}?", path.display()));

                        let confirm_clicked = ui
                            .link(
                                RichText::new("Press Enter to confirm").color(colors.highlight),
                            )
                            .clicked();
                        let cancel_clicked = ui
                            .link(RichText::new("Press Esc to cancel").color(colors.fg_light))
                            .clicked();

                        (confirm_clicked, cancel_clicked)
                    }
                    DeleteConfirmState::RecursiveConfirm => {
                        // Second confirmation specifically for directories
                        ui.label(
                            RichText::new(format!(
                                "Are you SURE you want to delete {} and ALL its contents recursively?",
                                path.display()
                            ))
                            .strong(),
                        );
                        ui.label("This action cannot be undone!");

                        let confirm_clicked = ui
                            .link(
                                RichText::new("Press Enter to confirm recursive deletion")
                                    .color(colors.highlight),
                            )
                            .clicked();
                        let cancel_clicked = ui
                            .link(RichText::new("Press Esc to cancel").color(colors.fg_light))
                            .clicked();

                        (confirm_clicked, cancel_clicked)
                    }
                };

                if confirm_clicked {
                    result = DeleteConfirmResult::Confirm;
                } else if cancel_clicked {
                    result = DeleteConfirmResult::Cancel;
                }

                ui.add_space(10.0);
            });
        })
    {
        *show_delete_confirm = !response.response.clicked_elsewhere();

        // If clicked elsewhere, treat as cancel
        if !*show_delete_confirm && result == DeleteConfirmResult::None {
            result = DeleteConfirmResult::Cancel;
        }
    }

    result
}

/// Helper function to perform the actual deletion
pub fn perform_delete(path: &Path, on_success: impl FnOnce()) -> Result<(), String> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };

    match result {
        Ok(_) => {
            on_success();
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete: {e}")),
    }
}

/// Handle the confirmation of deletion
pub fn confirm_delete(app: &mut crate::app::Kiorg) {
    if let Some(path) = app.entry_to_delete.clone() {
        // Check if we're in the initial state and dealing with a directory
        if app.delete_popup_state == DeleteConfirmState::Initial && path.is_dir() {
            // For directories in initial state, move to second confirmation
            app.delete_popup_state = DeleteConfirmState::RecursiveConfirm;
            return; // Return early without performing deletion
        }

        // For files or directories in second confirmation state, proceed with deletion
        if let Err(error) = perform_delete(&path, || {
            app.refresh_entries();
        }) {
            app.toasts.error(error);
        }

        // Reset state after deletion
        app.show_popup = None;
        app.entry_to_delete = None;
        app.delete_popup_state = DeleteConfirmState::Initial;
    }
}

/// Cancel the deletion process
pub fn cancel_delete(app: &mut crate::app::Kiorg) {
    app.show_popup = None;
    app.entry_to_delete = None;
    app.delete_popup_state = DeleteConfirmState::Initial; // Reset delete popup state
}
