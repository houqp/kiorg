use egui::{Align2, Context, RichText};

use crate::config::colors::AppColors;

/// Dialog windows for confirmations and interactions
pub struct Dialogs;

impl Dialogs {
    /// Show exit confirmation dialog (refactored from app.rs)
    pub fn show_exit_dialog(ctx: &Context, show_exit_confirm: &mut bool, colors: &AppColors) {
        if !*show_exit_confirm {
            return;
        }

        let mut keep_open = *show_exit_confirm; // Use a temporary variable for the open state

        let response = egui::Window::new("Exit Confirmation")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut keep_open) // Control window visibility
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    if ui
                        .link(RichText::new("Press Enter to exit").color(colors.yellow))
                        .clicked()
                    {
                        std::process::exit(0);
                    }
                    if ui
                        .link(RichText::new("Press Esc or q to cancel").color(colors.gray))
                        .clicked()
                    {
                        *show_exit_confirm = false; // Update the original state variable
                    }
                    ui.add_space(10.0);
                });
            });

        // Update the state based on window interaction
        if let Some(response) = response {
            if response.response.clicked_elsewhere() {
                *show_exit_confirm = false;
            } else {
                // If the window was closed by clicking the 'x' or similar, update the state
                // Note: The .open() binding handles this implicitly if keep_open was false after the show call.
                // We explicitly set it false if the links were clicked above.
                // If the window remains open (no interaction closed it), keep_open reflects that.
                *show_exit_confirm = keep_open;
            }
        } else {
            // If the window wasn't shown (e.g., closed via 'x' button before this frame), ensure state is false
            *show_exit_confirm = false;
        }
    }
}
