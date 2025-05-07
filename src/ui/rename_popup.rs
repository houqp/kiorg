use crate::app::Kiorg;
use crate::app::PopupType;
use egui::{Context, Frame, Key, TextEdit};

use super::window_utils::new_center_popup_window;

/// Draw the rename popup dialog
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    // Early return if not in rename mode
    if app.show_popup != Some(PopupType::Rename) {
        return;
    }

    let mut keep_open: bool = true;

    // Create a centered popup window
    new_center_popup_window("Rename")
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
                        let text_edit = TextEdit::singleline(&mut app.new_name)
                            .hint_text("Enter new name...")
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
        app.new_name.clear();
    }
}

/// Handle key presses for the rename popup
/// Note: This is already handled in input.rs, but we keep this here for completeness
pub fn handle_key_press(ctx: &Context, app: &mut Kiorg) -> bool {
    if app.show_popup != Some(PopupType::Rename) {
        return false;
    }

    // Check for Enter key to confirm rename
    if ctx.input(|i| i.key_pressed(Key::Enter)) {
        let tab = app.tab_manager.current_tab_mut();
        if let Some(entry) = tab.entries.get(tab.selected_index) {
            let parent = entry.path.parent().unwrap_or(&tab.current_path);
            let new_path = parent.join(&app.new_name);

            if let Err(e) = std::fs::rename(&entry.path, &new_path) {
                app.toasts.error(format!("Failed to rename: {e}"));
            } else {
                app.refresh_entries();
            }
        }
        app.show_popup = None;
        app.new_name.clear();
        return true;
    }

    // Check for Escape key to cancel rename
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        app.show_popup = None;
        app.new_name.clear();
        return true;
    }

    false
}
