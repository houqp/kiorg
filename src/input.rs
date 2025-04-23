use crate::ui::terminal;
use crate::ui::{bookmark_popup, center_panel};
use egui::{Key, Modifiers};

use super::app::Kiorg;

const DOUBLE_KEY_PRESS_THRESHOLD_MS: u64 = 500;

#[inline]
fn is_cancel_keys(key: Key) -> bool {
    key == Key::Escape || key == Key::Q
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

    if app.show_exit_confirm {
        if key == Key::Enter {
            app.shutdown_requested = true;
        } else if is_cancel_keys(key) {
            app.show_exit_confirm = false;
        }
        return;
    }

    if app.show_delete_confirm {
        if key == Key::Enter {
            app.confirm_delete();
        } else if is_cancel_keys(key) {
            app.cancel_delete();
        }
        return;
    }

    if app.rename_mode {
        if key == Key::Enter {
            let tab = app.state.tab_manager.current_tab();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                let parent = entry.path.parent().unwrap_or(&tab.current_path);
                let new_path = parent.join(&app.new_name);

                if let Err(e) = std::fs::rename(&entry.path, &new_path) {
                    eprintln!("Failed to rename: {e}");
                } else {
                    app.refresh_entries();
                }
            }
            app.rename_mode = false;
            app.new_name.clear();
        } else if key == Key::Escape {
            app.rename_mode = false;
            app.new_name.clear();
        }
        return;
    }

    if app.show_help {
        if is_cancel_keys(key) || key == Key::Enter || key == Key::Questionmark {
            app.show_help = false;
        }
        return;
    }

    match (key, modifiers.shift) {
        (Key::Questionmark, true) => {
            app.show_help = !app.show_help;
        }
        (Key::Q, false) => {
            app.show_exit_confirm = true;
        }
        (Key::R, false) => {
            app.rename_selected_entry();
        }
        (Key::D, false) => {
            app.delete_selected_entry();
        }
        // Handle copy/cut/paste
        (Key::Y, false) => {
            app.copy_selected_entries();
        }
        (Key::X, false) => {
            app.cut_selected_entries();
        }
        (Key::P, false) => {
            let tab = app.state.tab_manager.current_tab();
            if center_panel::handle_clipboard_operations(&mut app.clipboard, &tab.current_path) {
                app.refresh_entries();
            }
        }
        // Handle tab creation and switching
        (Key::T, false) => {
            let current_path = app.state.tab_manager.current_tab_ref().current_path.clone();
            app.state.tab_manager.add_tab(current_path);
            app.refresh_entries();
        }
        // T for terminal popup
        (Key::T, true) => {
            let path = app.state.tab_manager.current_tab().current_path.clone();
            app.terminal_ctx = Some(terminal::TerminalContext::new(ctx, path));
        }
        // Handle tab switching with number keys
        (Key::Num1, false) => {
            app.state.tab_manager.switch_to_tab(0);
            app.refresh_entries();
        }
        (Key::Num2, false) => {
            app.state.tab_manager.switch_to_tab(1);
            app.refresh_entries();
        }
        (Key::Num3, false) => {
            app.state.tab_manager.switch_to_tab(2);
            app.refresh_entries();
        }
        (Key::Num4, false) => {
            app.state.tab_manager.switch_to_tab(3);
            app.refresh_entries();
        }
        (Key::Num5, false) => {
            app.state.tab_manager.switch_to_tab(4);
            app.refresh_entries();
        }
        (Key::Num6, false) => {
            app.state.tab_manager.switch_to_tab(5);
            app.refresh_entries();
        }
        (Key::Num7, false) => {
            app.state.tab_manager.switch_to_tab(6);
            app.refresh_entries();
        }
        (Key::Num8, false) => {
            app.state.tab_manager.switch_to_tab(7);
            app.refresh_entries();
        }
        (Key::Num9, false) => {
            app.state.tab_manager.switch_to_tab(8);
            app.refresh_entries();
        }
        // Handle navigation in current panel
        (Key::J, false) | (Key::ArrowDown, false) => {
            app.move_selection(1);
        }
        (Key::K, false) | (Key::ArrowUp, false) => {
            app.move_selection(-1);
        }
        (Key::H, false) | (Key::ArrowLeft, false) => {
            let parent_path = app
                .state
                .tab_manager
                .current_tab_ref()
                .current_path
                .parent()
                .map(|p| p.to_path_buf());
            if let Some(parent) = parent_path {
                app.navigate_to_dir(parent);
            }
        }
        (Key::L, false) | (Key::ArrowRight, false) => {
            let tab = app.state.tab_manager.current_tab_ref();
            // Get the entry corresponding to the current `selected_index`.
            // This index always refers to the original `entries` list.
            if let Some(selected_entry) = tab.entries.get(tab.selected_index) {
                let path = selected_entry.path.clone();
                if path.is_dir() {
                    app.navigate_to_dir(path);
                }
            }
        }
        (Key::Enter, false) => {
            let tab = app.state.tab_manager.current_tab_ref();
            if let Some(selected_entry) = tab.entries.get(tab.selected_index) {
                let path = selected_entry.path.clone();
                if path.is_dir() {
                    app.navigate_to_dir(path);
                } else if path.is_file() {
                    // only open file on enter
                    app.open_file(path);
                }
            }
        }
        (Key::G, false) => {
            let tab = app.state.tab_manager.current_tab();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let last = app.last_lowercase_g_pressed_ms;
            if last > 0 && now - last < DOUBLE_KEY_PRESS_THRESHOLD_MS {
                tab.update_selection(0);
                app.ensure_selected_visible = true;
                app.selection_changed = true;
                // Reset the timestamp after double g presses has been detected
                app.last_lowercase_g_pressed_ms = 0;
            } else {
                app.last_lowercase_g_pressed_ms = now;
            }
        }
        (Key::G, true) => {
            let tab = app.state.tab_manager.current_tab();
            if !tab.entries.is_empty() {
                tab.update_selection(tab.entries.len() - 1);
                app.ensure_selected_visible = true;
                app.selection_changed = true;
            }
            app.last_lowercase_g_pressed_ms = 0;
        }
        (Key::Space, false) => {
            let tab = app.state.tab_manager.current_tab();
            if let Some(entry) = tab.entries.get(tab.selected_index) {
                if tab.selected_entries.contains(&entry.path) {
                    tab.selected_entries.remove(&entry.path);
                } else {
                    tab.selected_entries.insert(entry.path.clone());
                }
            }
        }
        (Key::B, false) => {
            bookmark_popup::toggle_bookmark(app);
        }
        (Key::B, true) => {
            // Toggle bookmark popup visibility
            app.show_bookmarks = !app.show_bookmarks;
        }
        (Key::Slash, false) => {
            app.search_bar.activate();
        }
        (Key::A, false) => {
            app.add_mode = true;
            app.add_focus = true; // Request focus for the input field
            app.new_entry_name.clear();
        }
        // Close current tab
        (Key::C, false) if modifiers.ctrl => {
            if app.state.tab_manager.close_current_tab() {
                // Refresh entries in case the active tab changed
                app.refresh_entries();
            }
        }
        _ => {}
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
