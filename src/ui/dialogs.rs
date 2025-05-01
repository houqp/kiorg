use egui::{Context, RichText};

use super::window_utils::new_center_popup_window;
use crate::config::colors::AppColors;

/// Show exit confirmation dialog (refactored from app.rs)
pub fn show_exit_dialog(ctx: &Context, keep_open: &mut bool, colors: &AppColors) {
    if !*keep_open {
        return;
    }

    let response = new_center_popup_window("Exit Confirmation")
        .open(keep_open) // Control window visibility
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                if ui
                    .link(RichText::new("Press Enter to exit").color(colors.highlight))
                    .clicked()
                {
                    std::process::exit(0);
                }
                let cancel_clicked = ui
                    .link(RichText::new("Press Esc or q to cancel").color(colors.fg_light))
                    .clicked();
                if cancel_clicked {
                    // We'll update keep_open after the window is shown
                }
                ui.add_space(10.0);
            });
        });

    // If the window wasn't shown (e.g., closed via 'x' button before this frame), ensure state is false
    if response.is_none() {
        *keep_open = false;
    }
}
