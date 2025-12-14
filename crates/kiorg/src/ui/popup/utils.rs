use egui::{Context, RichText, Ui};

use super::window_utils::{POPUP_MARGIN, new_center_popup_window};

/// Result of a confirmation popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmResult {
    /// User confirmed the action
    Confirm,
    /// User canceled the action
    Cancel,
    /// No action taken yet
    None,
}

/// Display a confirmation popup with customizable title, content, and button text
///
/// # Arguments
/// * `ctx` - The egui context
/// * `title` - The title of the popup window
/// * `show_popup` - Mutable reference to control popup visibility
/// * `colors` - Application colors for styling
/// * `content_fn` - Function to render the content of the popup
/// * `confirm_text` - Text for the confirm button (e.g., "Delete (Enter)")
/// * `cancel_text` - Text for the cancel button (e.g., "Cancel (Esc)")
///
/// # Returns
/// A `ConfirmResult` indicating the user's choice
pub fn show_confirm_popup<F>(
    ctx: &Context,
    title: &str,
    show_popup: &mut bool,
    content_fn: F,
    confirm_text: &str,
    cancel_text: &str,
) -> ConfirmResult
where
    F: FnOnce(&mut Ui),
{
    if !*show_popup {
        return ConfirmResult::None;
    }

    let mut result = ConfirmResult::None;

    let popup_response = new_center_popup_window(title)
        .open(show_popup)
        .max_width(450.0) // Restrict maximum width
        .show(ctx, |ui| {
            egui::Frame::new()
                .inner_margin(POPUP_MARGIN)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        content_fn(ui);

                        ui.add_space(20.0); // Space before buttons

                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let confirm_rich_text = RichText::new(confirm_text);
                                    let confirm_clicked = ui.button(confirm_rich_text).clicked();
                                    if confirm_clicked {
                                        result = ConfirmResult::Confirm;
                                    }
                                },
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let cancel_clicked =
                                        ui.button(RichText::new(cancel_text)).clicked();
                                    if cancel_clicked {
                                        result = ConfirmResult::Cancel;
                                    }
                                },
                            );
                        });
                    });
                });
        });
    if let Some(response) = popup_response {
        if (
            // If banner close button is clicked
            !*show_popup
            // If clicked outside of the window, treat as cancel
             || response.response.clicked_elsewhere()
        ) && result == ConfirmResult::None
        {
            result = ConfirmResult::Cancel;
        }
    } else {
        *show_popup = false;
    }

    result
}
