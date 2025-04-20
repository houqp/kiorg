use egui::{Context, RichText};
use std::path::{Path, PathBuf};

use super::window_utils::new_center_popup_window;
use crate::config::colors::AppColors;

/// Dialog for confirming file and directory deletion
pub struct DeleteDialog;

impl DeleteDialog {
    /// Show the delete confirmation dialog
    pub fn show_delete_dialog(
        ctx: &Context,
        show_delete_confirm: &mut bool,
        entry_to_delete: &Option<PathBuf>,
        colors: &AppColors,
        on_confirm: impl FnOnce(),
        on_cancel: impl FnOnce(),
    ) {
        if ctx.input(|i| i.key_pressed(egui::Key::Q) || i.key_pressed(egui::Key::Escape)) {
            *show_delete_confirm = false;
        }

        if !*show_delete_confirm || entry_to_delete.is_none() {
            return;
        }

        let path = entry_to_delete.as_ref().unwrap();
        let mut show_popup = *show_delete_confirm;

        if let Some(response) = new_center_popup_window("Delete Confirmation")
            .open(&mut show_popup)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(format!("Delete {}?", path.display()));
                    let confirm_clicked = ui
                        .link(RichText::new("Press Enter to confirm").color(colors.yellow))
                        .clicked();
                    let cancel_clicked = ui
                        .link(RichText::new("Press Esc to cancel").color(colors.gray))
                        .clicked();
                    ui.add_space(10.0);

                    if confirm_clicked {
                        on_confirm();
                    } else if cancel_clicked {
                        on_cancel();
                    }
                });
            })
        {
            *show_delete_confirm = !response.response.clicked_elsewhere();
        }
    }

    /// Handle the delete confirmation process
    pub fn handle_delete_confirmation(
        ctx: &Context,
        show_delete_confirm: &mut bool,
        entry_to_delete: &Option<PathBuf>,
        colors: &AppColors,
        on_confirm: impl FnOnce(),
        on_cancel: impl FnOnce(),
    ) {
        if *show_delete_confirm && entry_to_delete.is_some() {
            Self::show_delete_dialog(
                ctx,
                show_delete_confirm,
                entry_to_delete,
                colors,
                on_confirm,
                on_cancel,
            );
        }
    }

    /// Helper function to perform the actual deletion
    pub fn perform_delete(path: &Path, on_success: impl FnOnce()) {
        let result = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match result {
            Ok(_) => on_success(),
            Err(e) => eprintln!("Failed to delete: {e}"),
        }
    }
}
