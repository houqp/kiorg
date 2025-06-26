use crate::config::shortcuts::{shortcuts_helpers, ShortcutAction};
use crate::ui::terminal;
use crate::ui::{add_entry_popup, bookmark_popup, center_panel, preview_popup};
use egui::{Key, Modifiers};

use super::app::{Kiorg, PopupType};

const DOUBLE_KEY_PRESS_THRESHOLD_MS: u128 = 500;

#[inline]
fn is_cancel_keys(key: Key) -> bool {
    key == Key::Escape || key == Key::Q
}

// Helper function to handle a shortcut action
#[allow(clippy::too_many_lines)]
fn handle_shortcut_action(app: &mut Kiorg, ctx: &egui::Context, action: ShortcutAction) {
    match action {
        ShortcutAction::ShowFilePreview => preview_popup::handle_show_file_preview(app, ctx),
        ShortcutAction::MoveDown => app.move_selection(1),
        ShortcutAction::MoveUp => app.move_selection(-1),
        ShortcutAction::GoToParentDirectory => {
            let parent_path = app
                .tab_manager
                .current_tab_ref()
                .current_path
                .parent()
                .map(std::path::Path::to_path_buf);
            if let Some(parent) = parent_path {
                app.navigate_to_dir(parent);
            }
        }
        ShortcutAction::OpenDirectory => {
            let tab = app.tab_manager.current_tab_ref();
            if let Some(selected_entry) = tab.entries.get(tab.selected_index) {
                let path = selected_entry.path.clone();
                if path.is_dir() {
                    app.navigate_to_dir(path);
                }
            }
        }
        ShortcutAction::OpenDirectoryOrFile => {
            let tab = app.tab_manager.current_tab_ref();
            if let Some(selected_entry) = tab.entries.get(tab.selected_index) {
                let path = selected_entry.path.clone();
                if path.is_dir() {
                    app.navigate_to_dir(path);
                } else if path.is_file() {
                    // TODO: write a test for this
                    // only open file on enter
                    app.open_file(path);
                }
            }
        }
        ShortcutAction::GoToFirstEntry => {
            let tab = app.tab_manager.current_tab_mut();
            if !tab.entries.is_empty() {
                // Get the first filtered entry with its original index
                let first_filtered_index = tab
                    .get_cached_filtered_entries()
                    .first()
                    .map(|(_, index)| *index);
                if let Some(index) = first_filtered_index {
                    tab.update_selection(index);
                    app.ensure_selected_visible = true;
                    app.selection_changed = true;
                }
            }
            app.last_lowercase_g_pressed_ms = 0;
        }
        ShortcutAction::GoToLastEntry => {
            let tab = app.tab_manager.current_tab_mut();
            if !tab.entries.is_empty() {
                // Get the last filtered entry with its original index
                let last_filtered_index = tab
                    .get_cached_filtered_entries()
                    .last()
                    .map(|(_, index)| *index);
                if let Some(index) = last_filtered_index {
                    tab.update_selection(index);
                    app.ensure_selected_visible = true;
                    app.selection_changed = true;
                }
            }
            app.last_lowercase_g_pressed_ms = 0;
        }
        ShortcutAction::DeleteEntry => app.delete_selected_entry(),
        ShortcutAction::RenameEntry => app.rename_selected_entry(),
        ShortcutAction::AddEntry => app.show_popup = Some(PopupType::AddEntry(String::new())),
        ShortcutAction::SelectEntry => {
            let tab = app.tab_manager.current_tab_mut();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                let path = &entry.path;
                if tab.marked_entries.contains(path) {
                    // Unmark the entry
                    tab.marked_entries.remove(path);

                    // If this entry is in the clipboard as a cut or copy operation, remove it
                    match &mut app.clipboard {
                        Some(
                            crate::app::Clipboard::Cut(paths) | crate::app::Clipboard::Copy(paths),
                        ) => {
                            // Remove the path from the clipboard's paths list
                            paths.retain(|p| p != path);

                            // If the clipboard's paths list becomes empty, set the clipboard to None
                            if paths.is_empty() {
                                app.clipboard = None;
                            }
                        }
                        None => {}
                    }
                } else {
                    // Mark the entry
                    tab.marked_entries.insert(path.clone());
                }
            }
        }
        ShortcutAction::CopyEntry => app.copy_selected_entries(),
        ShortcutAction::CutEntry => app.cut_selected_entries(),
        ShortcutAction::PasteEntry => {
            let tab = app.tab_manager.current_tab_mut();
            if center_panel::handle_clipboard_operations(
                &mut app.clipboard,
                &tab.current_path,
                &mut app.toasts,
            ) {
                app.refresh_entries();
            }
        }
        ShortcutAction::CreateTab => {
            let current_path = app.tab_manager.current_tab_ref().current_path.clone();
            app.tab_manager.add_tab(current_path);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab1 => {
            app.tab_manager.switch_to_tab(0);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab2 => {
            app.tab_manager.switch_to_tab(1);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab3 => {
            app.tab_manager.switch_to_tab(2);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab4 => {
            app.tab_manager.switch_to_tab(3);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab5 => {
            app.tab_manager.switch_to_tab(4);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab6 => {
            app.tab_manager.switch_to_tab(5);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab7 => {
            app.tab_manager.switch_to_tab(6);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab8 => {
            app.tab_manager.switch_to_tab(7);
            app.refresh_entries();
        }
        ShortcutAction::SwitchToTab9 => {
            app.tab_manager.switch_to_tab(8);
            app.refresh_entries();
        }
        ShortcutAction::CloseCurrentTab => {
            if app.tab_manager.close_current_tab() {
                // Refresh entries in case the active tab changed
                app.refresh_entries();
            }
        }
        ShortcutAction::ToggleBookmark => bookmark_popup::toggle_bookmark(app),
        ShortcutAction::ShowBookmarks => app.show_popup = Some(PopupType::Bookmarks(0)),
        ShortcutAction::OpenTerminal => {
            let path = app.tab_manager.current_tab_mut().current_path.clone();
            match terminal::TerminalContext::new(ctx, path) {
                Ok(terminal_ctx) => {
                    app.terminal_ctx = Some(terminal_ctx);
                }
                Err(error) => {
                    tracing::error!(err = ?error, "error creating terminal");
                    app.notify_error(error);
                }
            }
        }
        ShortcutAction::ShowHelp => {
            // Toggle help popup
            if app.show_popup == Some(PopupType::Help) {
                app.show_popup = None;
            } else {
                app.show_popup = Some(PopupType::Help);
            }
        }
        ShortcutAction::Exit => app.show_popup = Some(PopupType::Exit),
        ShortcutAction::ActivateSearch => app.search_bar.activate(),
        ShortcutAction::GoBackInHistory => app.navigate_history_back(),
        ShortcutAction::GoForwardInHistory => app.navigate_history_forward(),
        ShortcutAction::SwitchToNextTab => {
            let current_index = app.tab_manager.get_current_tab_index();
            let total_tabs = app.tab_manager.get_tab_count();
            if total_tabs > 1 {
                let next_index = (current_index + 1) % total_tabs;
                app.tab_manager.switch_to_tab(next_index);
                app.refresh_entries();
            }
        }
        ShortcutAction::SwitchToPreviousTab => {
            let current_index = app.tab_manager.get_current_tab_index();
            let total_tabs = app.tab_manager.get_tab_count();
            if total_tabs > 1 {
                let prev_index = (current_index + total_tabs - 1) % total_tabs;
                app.tab_manager.switch_to_tab(prev_index);
                app.refresh_entries();
            }
        }
        ShortcutAction::OpenWithCommand => {
            let tab = app.tab_manager.current_tab_ref();
            if let Some(_selected_entry) = tab.selected_entry() {
                // Show the open with popup with an empty command string
                // Now works for both files and directories
                app.show_popup = Some(PopupType::OpenWith(String::new()));
            }
        }
        ShortcutAction::PageUp => app.move_selection_by_page(-1),
        ShortcutAction::PageDown => app.move_selection_by_page(1),
        ShortcutAction::SelectAllEntries => app.select_all_entries(),
    }
}

#[allow(clippy::too_many_lines)]
fn process_key(
    app: &mut Kiorg,
    ctx: &egui::Context,
    key: Key,
    modifiers: Modifiers,
    pressed: bool,
) {
    if !pressed {
        return;
    }

    // Handle special modal states first based on the show_popup field
    match &app.show_popup {
        Some(PopupType::Preview) => {
            if is_cancel_keys(key) {
                app.show_popup = None;
                return;
            }

            // Handle preview popup input (PDF page navigation, etc.)
            match &mut app.preview_content {
                Some(crate::models::preview_content::PreviewContent::Pdf(pdf_meta)) => {
                    preview_popup::doc::handle_preview_popup_input_pdf(
                        pdf_meta, key, modifiers, ctx,
                    );
                }
                Some(crate::models::preview_content::PreviewContent::Epub(_epub_meta)) => {
                    // EPUB documents don't have page navigation in preview popup
                    // Only handle ESC to close popup which is already handled above
                }
                _ => {}
            }
            return;
        }
        Some(PopupType::Exit) => {
            if key == Key::Enter {
                crate::ui::exit_popup::confirm_exit(app);
            } else if is_cancel_keys(key) {
                crate::ui::exit_popup::cancel_exit(app);
            }
            return;
        }
        Some(PopupType::Delete(_)) => {
            if key == Key::Enter {
                crate::ui::delete_popup::confirm_delete(app);
            } else if is_cancel_keys(key) {
                crate::ui::delete_popup::cancel_delete(app);
            }
            return;
        }
        Some(PopupType::Rename(_)) => {
            if key == Key::Enter {
                crate::ui::rename_popup::handle_rename_confirmation(app, ctx);
            } else if key == Key::Escape {
                crate::ui::rename_popup::close_rename_popup(app, ctx);
            }
            return;
        }
        Some(PopupType::OpenWith(cmd)) => {
            if key == Key::Enter {
                crate::ui::open_with_popup::confirm_open_with(app, cmd.clone());
            } else if key == Key::Escape {
                crate::ui::open_with_popup::close_popup(app);
            }
            return;
        }
        Some(PopupType::Help) => {
            if is_cancel_keys(key) || key == Key::Enter || key == Key::Questionmark {
                app.show_popup = None;
            }
            return;
        }
        Some(PopupType::About) => {
            if is_cancel_keys(key) {
                app.show_popup = None;
            }
            return;
        }
        Some(PopupType::AddEntry(_)) => {
            if add_entry_popup::handle_key_press(ctx, app) {
                return;
            }
        }
        Some(PopupType::Themes(_) | PopupType::Bookmarks(_)) => {
            // Theme popup input is handled in the popup itself
            // Bookmark popup input is handled in show_bookmark_popup
            return;
        }
        Some(PopupType::DeleteProgress(_)) => {
            // Delete progress popup doesn't handle input - just return
            return;
        }
        None => {}
    }

    // Handle ESC key to clear search filter when search is active but not focused
    if key == Key::Escape && app.search_bar.query.is_some() && !app.search_bar.focus {
        app.search_bar.close();
        // Reset filter when closing search bar
        let tab = app.tab_manager.current_tab_mut();
        tab.update_filtered_cache(&None, false);
        return;
    }

    // Special case for g key to handle namespace
    let mut namespace = false;
    if key == Key::G && !modifiers.shift {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let last = app.last_lowercase_g_pressed_ms;
        if last > 0 && now - last < DOUBLE_KEY_PRESS_THRESHOLD_MS {
            // Double-g press detected, use the namespace system
            namespace = true;
        } else {
            // First g press, set the timestamp and namespace flag
            app.last_lowercase_g_pressed_ms = now;
            // consume the key
            return;
        }
    } else if app.last_lowercase_g_pressed_ms > 0 {
        // Any other key press after g, check if within threshold
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        if now - app.last_lowercase_g_pressed_ms < DOUBLE_KEY_PRESS_THRESHOLD_MS {
            namespace = true;
        }

        // Reset the g timestamp after any other key is pressed
        app.last_lowercase_g_pressed_ms = 0;
    }

    // Find and handle the action for this key combination
    let shortcuts = app.get_shortcuts();
    if let Some(action) = shortcuts_helpers::find_action(shortcuts, key, modifiers, namespace) {
        handle_shortcut_action(app, ctx, action);
    }
}

pub(crate) fn process_input_events(app: &mut Kiorg, ctx: &egui::Context) {
    let events = ctx.input(|i| i.events.clone());
    for event in events {
        if let egui::Event::Key {
            key,
            modifiers,
            pressed,
            ..
        } = event
        {
            process_key(app, ctx, key, modifiers, pressed);
        }
    }
}
