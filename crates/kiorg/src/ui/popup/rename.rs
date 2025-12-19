use crate::app::Kiorg;
use crate::models::action_history::{ActionType, RenameOperation};
use crate::ui::popup::PopupType;
use egui::Context;

use super::text_input_popup::{
    TextInputConfig, TextSelection, clear_init_flag, draw as draw_text_input,
};

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

const RENAME_POPUP_ID: &str = "rename_popup";

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
    clear_init_flag(ctx, RENAME_POPUP_ID);
}

/// Draw the rename popup dialog
pub fn draw(ctx: &egui::Context, app: &mut Kiorg) {
    if let Some(PopupType::Rename(new_name)) = &mut app.show_popup {
        let extension_pos = find_extension_position(new_name);

        let config = TextInputConfig {
            title: "Rename",
            hint: "Enter new name...",
            initial_selection: TextSelection::Range {
                start: 0,
                end: extension_pos,
            },
        };

        let keep_open = draw_text_input(ctx, &app.colors, &config, new_name, RENAME_POPUP_ID);

        if !keep_open {
            close_rename_popup(app, ctx);
        }
    }
}

pub fn clear_popup_initialization_flag(ctx: &Context) {
    clear_init_flag(ctx, RENAME_POPUP_ID);
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
