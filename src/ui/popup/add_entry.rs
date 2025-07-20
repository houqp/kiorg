use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use egui::{Context, Frame, Key, TextEdit};
use std::fs;

use super::window_utils::new_center_popup_window;

pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    // Early return if not in add mode
    if let Some(PopupType::AddEntry(entry_name)) = &mut app.show_popup {
        let mut keep_open: bool = true;

        // Use Area instead of Window for a more lightweight appearance like search_bar
        new_center_popup_window("Add file/directory")
            .open(&mut keep_open)
            .show(ctx, |ui| {
                // Create a frame with styling similar to search bar
                Frame::default()
                    .fill(app.colors.bg_extreme)
                    .inner_margin(5.0)
                    .show(ui, |ui| {
                        ui.set_max_width(400.0); // Limit width

                        // Horizontal layout for input and close button
                        ui.horizontal(|ui| {
                            // Text input field
                            let text_edit = TextEdit::singleline(entry_name)
                                .hint_text("Enter name (append '/' at the end for directory)...")
                                .desired_width(f32::INFINITY) // Take available width
                                .frame(false); // No frame, like search bar

                            let response = ui.add(text_edit);

                            // Always request focus when the popup is shown
                            response.request_focus();
                        });
                    });
            });

        if !keep_open {
            app.show_popup = None;
        }
    }
}

/// Handles input specifically when the add entry popup is active.
/// Returns `true` if the input was handled (consumed), `false` otherwise.
pub(crate) fn handle_key_press(ctx: &Context, app: &mut Kiorg) -> bool {
    // Early return if not in add mode
    let entry_name = match &app.show_popup {
        Some(PopupType::AddEntry(name)) => name.clone(),
        _ => return false, // Not in add mode, let other handlers run
    };

    // Handle cancellation
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        app.show_popup = None;
        return true; // Input handled
    }

    // Handle confirmation
    if ctx.input(|i| i.key_pressed(Key::Enter)) {
        if !entry_name.is_empty() {
            // Get the current path and create the new path
            let current_path = app.tab_manager.current_tab_ref().current_path.clone();
            let new_path = current_path.join(&entry_name);

            // Check if a file or directory with the same name already exists
            if new_path.exists() {
                // Show error message and keep the popup open
                app.notify_error(format!(
                    "Cannot create '{}': Entry with the same name already exists",
                    entry_name.escape_default()
                ));
                // Don't close the popup so the user can modify the name
                return true; // Input handled
            }

            let result = if entry_name.ends_with('/') {
                // Create directory
                // Ensure parent directories exist before creating the final one
                let parent = new_path.parent().unwrap_or(&current_path);
                fs::create_dir_all(parent).and_then(|_| fs::create_dir(&new_path))
            } else {
                // Create file
                // Ensure parent directories exist before creating the file
                if let Some(parent) = new_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        app.notify_error(format!(
                            "Failed to create parent directories for '{}': {}",
                            entry_name.escape_default(),
                            e
                        ));
                        // Decide how to handle this error, maybe return early?
                        // For now, we'll proceed and let File::create handle the final error.
                    }
                }
                fs::File::create(&new_path).map(|_| ()) // Discard the File handle
            };

            match result {
                Err(e) => {
                    app.notify_error(format!(
                        "Failed to create '{}': {}",
                        entry_name.escape_default(),
                        e
                    ));
                    // Keep the popup open on error so the user can try again
                    return true; // Input handled
                }
                Ok(_) => {
                    // --- Start: Preserve Selection After Creation ---
                    // Store the path of the newly created entry
                    let created_path = current_path.join(&entry_name);
                    app.prev_path = Some(created_path); // Use prev_path to select the new entry
                    // --- End: Preserve Selection After Creation ---
                    app.refresh_entries();
                }
            }
        }
        app.show_popup = None;
        return true; // Input handled
    }

    // Handle text input (delegated to the popup drawing logic via focus)
    // We just need to ensure other keys don't interfere while the popup is active.
    // Check for any text input event to consume it.
    if !ctx.input(|i| i.events.is_empty()) {
        // Check specifically for text input or backspace/delete if needed,
        // but for now, consuming any event might be sufficient to block others.
        // A more robust check might look for `event.key == Key::Backspace` etc.
        // or text events specifically.
        // For simplicity, we return true to indicate the mode handled the input.
        return true;
    }

    // If we reach here, it means it was some other key press while in add mode,
    // which we want to block.
    true // Input handled (blocked)
}
