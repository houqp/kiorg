use crate::app::Kiorg;
use crate::models::action_history::{ActionType, RenameOperation};
use crate::ui::popup::PopupType;
use egui::{Context, Frame, TextEdit};

use super::window_utils::new_center_popup_window;

/// Find the position before the file extension in a filename
/// Returns the position (in characters) where the extension starts, or the end of the string if no extension
fn find_extension_position(filename: &str) -> usize {
    // Find the last dot in the filename
    if let Some(dot_pos) = filename.rfind('.') {
        // Make sure it's not a hidden file (starting with a dot)
        if dot_pos > 0 {
            // Convert byte position to character position
            return filename[..dot_pos].chars().count();
        }
    }
    // No extension found, return the end of the string (in characters)
    filename.chars().count()
}

const RENAME_POPUP_INITIALIZED: &str = "rename_popup_initialized";

/// Helper function to handle rename confirmation
pub fn handle_rename_confirmation(app: &mut Kiorg, ctx: &Context) {
    // Extract the new name from the popup
    if let Some(PopupType::Rename(new_name)) = &app.show_popup {
        let tab = app.tab_manager.current_tab_mut();
        if let Some(entry) = tab.entries.get(tab.selected_index) {
            let parent = entry.path.parent().unwrap_or(&tab.current_path);
            let new_path = parent.join(new_name);

            if let Err(e) = std::fs::rename(&entry.path, &new_path) {
                app.notify_error(format!("Failed to rename: {e}"));
            } else {
                // Record rename action in history
                let old_path = entry.path.clone();
                tab.action_history.add_action(ActionType::Rename {
                    operations: vec![RenameOperation { old_path, new_path }],
                });

                app.refresh_entries();
            }
        }
    }
    close_rename_popup(app, ctx);
}

/// Helper function to handle rename cancellation
pub fn close_rename_popup(app: &mut Kiorg, ctx: &Context) {
    // Just clean up without performing the rename
    app.show_popup = None;
    clear_popup_initialization_flag(ctx);
}

/// Draw the rename popup dialog
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    if let Some(PopupType::Rename(new_name)) = &mut app.show_popup {
        let mut keep_open: bool = true;

        // Create a centered popup window
        new_center_popup_window("Rename")
            .open(&mut keep_open)
            .show(ctx, |ui| {
                // Create a frame with styling similar to other popups
                Frame::default()
                    .fill(app.colors.bg_extreme)
                    .inner_margin(5.0)
                    .show(ui, |ui| {
                        ui.set_max_width(400.0); // Limit width

                        // Horizontal layout for input and close button
                        ui.horizontal(|ui| {
                            // Store the original name to detect if this is the first frame
                            let is_first_frame = ui.memory(|mem| {
                                !mem.data
                                    .get_temp::<bool>(egui::Id::new(RENAME_POPUP_INITIALIZED))
                                    .unwrap_or(false)
                            });

                            // Create a TextEdit widget with custom cursor position
                            let text_edit = TextEdit::singleline(new_name)
                                .hint_text("Enter new name...")
                                .desired_width(f32::INFINITY) // Take available width
                                .frame(false); // No frame, like search bar
                            let response = ui.add(text_edit);

                            // Always request focus when the popup is shown
                            response.request_focus();

                            // If this is the first frame, set the cursor position using the stored value
                            if is_first_frame {
                                if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id)
                                {
                                    // Find the position before the file extension
                                    let cursor_selection_range = egui::text::CCursorRange::two(
                                        egui::text::CCursor::new(0),
                                        egui::text::CCursor::new(find_extension_position(new_name)),
                                    );
                                    state.cursor.set_char_range(Some(cursor_selection_range));
                                    state.store(ui.ctx(), response.id);
                                }
                                // Mark that we've initialized the popup
                                ui.memory_mut(|mem| {
                                    mem.data
                                        .insert_temp(egui::Id::new(RENAME_POPUP_INITIALIZED), true);
                                });
                            }
                        });
                    });
            });

        if !keep_open {
            close_rename_popup(app, ctx); // Cancel if window is closed
        }
    }
}

pub fn clear_popup_initialization_flag(ctx: &Context) {
    ctx.memory_mut(|mem| {
        mem.data
            .insert_temp::<bool>(egui::Id::new(RENAME_POPUP_INITIALIZED), false);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_extension_position() {
        // Test with ASCII characters
        assert_eq!(find_extension_position("file.txt"), 4);
        assert_eq!(find_extension_position("file.name.txt"), 9);
        assert_eq!(find_extension_position("file"), 4);
        assert_eq!(find_extension_position(".hidden"), 7);

        // Test with Chinese characters
        assert_eq!(find_extension_position("文件.txt"), 2);
        assert_eq!(find_extension_position("文件名称.doc"), 4);
        assert_eq!(find_extension_position("文件名称"), 4);

        // Test with mixed characters
        assert_eq!(find_extension_position("file名称.txt"), 6);
        assert_eq!(find_extension_position("文件name.doc"), 6);
    }
}
