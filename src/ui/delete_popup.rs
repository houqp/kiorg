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
    entry_to_delete: &Option<PathBuf>,
    colors: &AppColors,
    state: &mut DeleteConfirmState,
) -> DeleteConfirmResult {
    if !*show_delete_confirm || entry_to_delete.is_none() {
        return DeleteConfirmResult::None;
    }

    let path = entry_to_delete.as_ref().unwrap();

    match *state {
        DeleteConfirmState::Initial => {
            // Initial confirmation for any file or directory
            show_confirm_popup(
                ctx,
                "Delete Confirmation",
                show_delete_confirm,
                colors,
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
                colors,
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

        cancel_delete(app);
    }
}

/// Cancel the deletion process
pub fn cancel_delete(app: &mut crate::app::Kiorg) {
    app.show_popup = None;
    app.entry_to_delete = None;
    app.delete_popup_state = DeleteConfirmState::Initial; // Reset delete popup state
}
