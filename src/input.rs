use crate::config::shortcuts::{self, shortcuts_helpers, ShortcutAction};
use crate::ui::terminal;
use crate::ui::{bookmark_popup, center_panel};
use egui::{Key, Modifiers};

use super::app::{Kiorg, PopupType};

const DOUBLE_KEY_PRESS_THRESHOLD_MS: u64 = 500;

#[inline]
fn is_cancel_keys(key: Key) -> bool {
    key == Key::Escape || key == Key::Q
}

// Helper function to handle a shortcut action
fn handle_shortcut_action(app: &mut Kiorg, ctx: &egui::Context, action: ShortcutAction) {
    match action {
        ShortcutAction::MoveDown => {
            app.move_selection(1);
        }
        ShortcutAction::MoveUp => {
            app.move_selection(-1);
        }
        ShortcutAction::GoToParentDirectory => {
            let parent_path = app
                .tab_manager
                .current_tab_ref()
                .current_path
                .parent()
                .map(|p| p.to_path_buf());
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
            let tab = app.tab_manager.current_tab();
            if !tab.entries.is_empty() {
                tab.update_selection(0);
                app.ensure_selected_visible = true;
                app.selection_changed = true;
            }
            app.last_lowercase_g_pressed_ms = 0;
        }
        ShortcutAction::GoToLastEntry => {
            let tab = app.tab_manager.current_tab();
            if !tab.entries.is_empty() {
                tab.update_selection(tab.entries.len() - 1);
                app.ensure_selected_visible = true;
                app.selection_changed = true;
            }
            app.last_lowercase_g_pressed_ms = 0;
        }
        ShortcutAction::DeleteEntry => {
            app.delete_selected_entry();
        }
        ShortcutAction::RenameEntry => {
            app.rename_selected_entry();
        }
        ShortcutAction::AddEntry => {
            app.add_mode = true;
            app.add_focus = true; // Request focus for the input field
            app.new_entry_name.clear();
        }
        ShortcutAction::SelectEntry => {
            let tab = app.tab_manager.current_tab();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                if tab.selected_entries.contains(&entry.path) {
                    tab.selected_entries.remove(&entry.path);
                } else {
                    tab.selected_entries.insert(entry.path.clone());
                }
            }
        }
        ShortcutAction::CopyEntry => {
            app.copy_selected_entries();
        }
        ShortcutAction::CutEntry => {
            app.cut_selected_entries();
        }
        ShortcutAction::PasteEntry => {
            let tab = app.tab_manager.current_tab();
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
        ShortcutAction::ToggleBookmark => {
            bookmark_popup::toggle_bookmark(app);
        }
        ShortcutAction::ShowBookmarks => {
            // Toggle bookmark popup visibility
            app.show_bookmarks = !app.show_bookmarks;
        }
        ShortcutAction::OpenTerminal => {
            let path = app.tab_manager.current_tab().current_path.clone();
            app.terminal_ctx = Some(terminal::TerminalContext::new(ctx, path));
        }
        ShortcutAction::ShowHelp => {
            // Toggle help popup
            if app.show_popup == Some(PopupType::Help) {
                app.show_popup = None;
            } else {
                app.show_popup = Some(PopupType::Help);
            }
        }
        ShortcutAction::Exit => {
            app.show_popup = Some(PopupType::Exit);
        }
        ShortcutAction::ActivateSearch => {
            app.search_bar.activate();
        }
    }
}

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
    match app.show_popup {
        Some(PopupType::Exit) => {
            if key == Key::Enter {
                app.shutdown_requested = true;
            } else if is_cancel_keys(key) {
                app.show_popup = None;
            }
            return;
        }
        Some(PopupType::Delete) => {
            if key == Key::Enter {
                app.confirm_delete();
            } else if is_cancel_keys(key) {
                app.cancel_delete();
            }
            return;
        }
        Some(PopupType::Rename) => {
            if key == Key::Enter {
                let tab = app.tab_manager.current_tab();
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
            } else if key == Key::Escape {
                app.show_popup = None;
                app.new_name.clear();
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
        None => {}
    }

    // Get shortcuts from config or use defaults
    let shortcuts = match &app.config.shortcuts {
        Some(shortcuts) => shortcuts,
        None => {
            // If no shortcuts are configured, use the default ones
            shortcuts::get_default_shortcuts()
        }
    };

    // Special case for g key to handle namespace
    let mut namespace = false;
    if key == Key::G && !modifiers.shift {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

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
            .as_millis() as u64;

        if now - app.last_lowercase_g_pressed_ms < DOUBLE_KEY_PRESS_THRESHOLD_MS {
            namespace = true;
        }

        // Reset the g timestamp after any other key is pressed
        app.last_lowercase_g_pressed_ms = 0;
    }

    // Find and handle the action for this key combination
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
