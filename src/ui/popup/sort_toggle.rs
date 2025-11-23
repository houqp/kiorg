//! Sort toggle popup module for toggling sort order of file manager columns

use crate::app::Kiorg;
use crate::models::tab::SortColumn;
use crate::ui::popup::PopupType;
use crate::ui::popup::window_utils::new_center_popup_window;
use egui::{Align2, Color32, Key, RichText};

/// Show the sort toggle popup
pub fn show_sort_toggle_popup(app: &mut Kiorg, ctx: &egui::Context) {
    // Check if the popup should be shown based on the show_popup field
    if app.show_popup != Some(PopupType::SortToggle) {
        return;
    }

    let mut keep_open = true; // Use a temporary variable for the open state

    let response = new_center_popup_window("Sort Toggle")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut keep_open) // Control window visibility
        .show(ctx, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                // Simple shortcut hints displayed horizontally
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("[n]").color(Color32::LIGHT_BLUE).strong());
                    ui.label("Name");

                    ui.add_space(20.0);

                    ui.label(RichText::new("[s]").color(Color32::LIGHT_BLUE).strong());
                    ui.label("Size");

                    ui.add_space(20.0);

                    ui.label(RichText::new("[m]").color(Color32::LIGHT_BLUE).strong());
                    ui.label("Modified");
                    ui.add_space(10.0);
                });
            });
            ui.add_space(10.0);
        });

    // Update the state based on window interaction
    if response.is_some() {
        if !keep_open {
            app.show_popup = None;
        }
    } else {
        app.show_popup = None;
    }
}

/// Handle key input when the sort toggle popup is active
pub fn handle_sort_toggle_key(app: &mut Kiorg, key: Key) {
    match key {
        Key::N => {
            app.tab_manager.toggle_sort(SortColumn::Name);
        }
        Key::S => {
            app.tab_manager.toggle_sort(SortColumn::Size);
        }
        Key::M => {
            app.tab_manager.toggle_sort(SortColumn::Modified);
        }
        _ => {}
    }
}
