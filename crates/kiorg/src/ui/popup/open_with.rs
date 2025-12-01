use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use egui::{Frame, TextEdit};

use super::window_utils::new_center_popup_window;

/// Draw the open with popup dialog
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    // Early return if not in open with mode
    if let Some(PopupType::OpenWith(command)) = &mut app.show_popup {
        let mut keep_open: bool = true;

        // Create a centered popup window
        new_center_popup_window("Open with")
            .open(&mut keep_open)
            .show(ctx, |ui| {
                // Create a frame with styling similar to other popups
                Frame::default()
                    .fill(app.colors.bg_light)
                    .inner_margin(5.0)
                    .show(ui, |ui| {
                        ui.set_max_width(400.0); // Limit width

                        // Horizontal layout for input and close button
                        ui.horizontal(|ui| {
                            // Text input field
                            let text_edit = TextEdit::singleline(command)
                                .hint_text("Enter command to open...")
                                .desired_width(f32::INFINITY) // Take available width
                                .frame(false); // No frame, like search bar

                            let response = ui.add(text_edit);

                            // Always request focus when the popup is shown
                            response.request_focus();
                        });
                    });
            });

        if !keep_open {
            close_popup(app);
        }
    }
}

/// Helper function to handle open with confirmation
pub fn confirm_open_with(app: &mut Kiorg, command: String) {
    if command.is_empty() {
        app.notify_error("Cannot open: No command provided");
        return;
    }

    // Get the path and command before calling other functions to avoid borrow issues
    let path_to_open = {
        let tab = app.tab_manager.current_tab_ref();
        tab.selected_entry().map(|entry| entry.path.clone())
    };

    // Only open if we have a valid path
    if let Some(path) = path_to_open {
        app.open_file_with_command(path, command);
    }

    close_popup(app);
}

/// Helper function to handle open with cancellation
pub fn close_popup(app: &mut Kiorg) {
    app.show_popup = None;
}
