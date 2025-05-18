use crate::app::{Kiorg, PopupType};
use egui::{Context, Frame, Key, TextEdit};
use std::path::PathBuf;

use super::window_utils::new_center_popup_window;

/// Draw the open with popup dialog
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    // Early return if not in open with mode
    if app.show_popup != Some(PopupType::OpenWith) {
        return;
    }

    let mut keep_open: bool = true;

    // Create a centered popup window
    new_center_popup_window("Open with")
        .open(&mut keep_open)
        .show(ctx, |ui| {
            // Set background color to match search bar
            ui.visuals_mut().widgets.noninteractive.bg_fill = app.colors.bg_light;

            // Create a frame with styling similar to other popups
            Frame::default()
                .fill(app.colors.bg_light)
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.set_max_width(400.0); // Limit width

                    // Horizontal layout for input and close button
                    ui.horizontal(|ui| {
                        // Text input field
                        let text_edit = TextEdit::singleline(&mut app.open_with_command)
                            .hint_text("Enter command to open file...")
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

/// Helper function to handle open with confirmation
pub fn confirm_open_with(app: &mut Kiorg) {
    // Get the path and command before calling other functions to avoid borrow issues
    let path_to_open = {
        let tab = app.tab_manager.current_tab_ref();
        tab.selected_entry()
            .filter(|entry| !entry.is_dir)
            .map(|entry| entry.path.clone())
    };

    let command = app.open_with_command.clone();

    // Only open the file if we have a valid path
    if let Some(path) = path_to_open {
        open_file_with_command(app, path, &command);
    }

    close_popup(app);
}

/// Helper function to handle open with cancellation
pub fn close_popup(app: &mut Kiorg) {
    app.show_popup = None;
    app.open_with_command.clear();
}

/// Open a file with a custom command
fn open_file_with_command(app: &mut Kiorg, path: PathBuf, command: &str) {
    if command.is_empty() {
        return;
    }

    // Clone the path for the thread
    let path_clone = path.clone();
    let command_clone = command.to_string();

    // Clone the error sender for the thread
    let error_sender = app.error_sender.clone();

    // Spawn a thread to open the file asynchronously
    std::thread::spawn(move || match open::with(&path_clone, &command_clone) {
        Ok(_) => {}
        Err(e) => {
            // Send the error message back to the main thread
            let _ = error_sender.send(format!(
                "Failed to open file with '{}': {}",
                command_clone, e
            ));
        }
    });
}

/// Handles input specifically when the open with popup is active.
/// Returns `true` if the input was handled (consumed), `false` otherwise.
pub(crate) fn handle_key_press(ctx: &Context, app: &mut Kiorg) -> bool {
    // Early return if not in open with mode
    if app.show_popup != Some(PopupType::OpenWith) {
        return false; // Not in open with mode, let other handlers run
    }

    // Handle cancellation
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        close_popup(app);
        return true; // Input handled
    }

    // Handle confirmation
    if ctx.input(|i| i.key_pressed(Key::Enter)) {
        confirm_open_with(app);
        return true; // Input handled
    }

    false // Let other handlers run
}
