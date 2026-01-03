use egui::Context;

use super::utils::{ConfirmResult, show_confirm_popup};
use crate::app::Kiorg;
use crate::ui::popup::PopupType;

/// Handle exit confirmation
pub fn confirm_exit(app: &mut Kiorg) {
    // Set shutdown_requested flag for graceful shutdown
    // The actual shutdown will be handled in the app's update loop
    app.shutdown_requested = true;
    app.show_popup = None;
}

/// Handle exit cancellation
pub fn cancel_exit(app: &mut Kiorg) {
    app.show_popup = None;
}

/// Draw the exit confirmation popup
pub fn draw(ctx: &Context, app: &mut Kiorg) {
    // Early return if not in exit mode
    if !matches!(app.show_popup, Some(PopupType::Exit)) {
        return;
    }

    let mut keep_open = true;

    let result = show_confirm_popup(
        ctx,
        "Exit Confirmation",
        &mut keep_open,
        |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Are you sure you want to exit?");
            });
        },
        "Exit (Enter)",
        "Cancel (Esc)",
    );

    // Handle the result
    match result {
        ConfirmResult::Confirm => confirm_exit(app),
        ConfirmResult::Cancel => cancel_exit(app),
        ConfirmResult::None => {
            if !keep_open {
                cancel_exit(app);
            }
        }
    }
}
