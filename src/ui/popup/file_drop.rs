use egui::{Context, Frame, ScrollArea};
use std::path::PathBuf;

use super::window_utils::new_center_popup_window;
use crate::app::Clipboard;
use crate::app::Kiorg;
use crate::config::shortcuts::ShortcutAction;
use crate::ui::center_panel::handle_clipboard_operations;
use crate::ui::popup::PopupType;

/// File drop operation types
#[derive(Clone, Copy, PartialEq)]
pub enum FileDropAction {
    None,
    Copy,
    Move,
    Cancel,
}

/// Draw the file drop popup dialog
pub fn draw(ctx: &Context, app: &mut Kiorg) {
    // Early return if not in file drop mode
    if let Some(PopupType::FileDrop(dropped_files)) = &app.show_popup {
        let dropped_files = dropped_files.clone(); // Clone to avoid borrow issues
        let mut keep_open = true;
        let mut action = FileDropAction::None;

        // Create a centered popup window with file count
        let title = format!("Files Dropped ({})", dropped_files.len());
        new_center_popup_window(&title)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                // Create a frame with styling similar to other popups
                Frame::default().inner_margin(5.0).show(ui, |ui| {
                    ui.set_max_width(450.0); // Slightly wider than other popups to show file paths
                    ui.set_min_width(400.0);

                    // Scrollable list of dropped files
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (i, file_path) in dropped_files.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // File icon (simple text for now)
                                let icon = if file_path.is_dir() { "📁" } else { "📄" };
                                ui.label(icon);

                                // File name and path
                                ui.vertical(|ui| {
                                    if let Some(name) = file_path.file_name() {
                                        ui.label(
                                            egui::RichText::new(name.to_string_lossy()).strong(),
                                        );
                                    }
                                    ui.label(
                                        egui::RichText::new(file_path.to_string_lossy())
                                            .small()
                                            .color(app.colors.fg_light),
                                    );
                                });
                            });

                            if i < dropped_files.len() - 1 {
                                ui.separator();
                            }
                        }
                    });

                    ui.add_space(10.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Copy").clicked() {
                            action = FileDropAction::Copy;
                        }

                        if ui.button("Move/Cut").clicked() {
                            action = FileDropAction::Move;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                action = FileDropAction::Cancel;
                            }
                        });
                    });
                });
            });

        // Handle the action
        match action {
            FileDropAction::Copy => {
                app.clipboard = Some(Clipboard::Copy(dropped_files));

                let current_tab = app.tab_manager.current_tab_mut();
                if handle_clipboard_operations(
                    &mut app.clipboard,
                    &current_tab.current_path,
                    &mut current_tab.action_history,
                    &mut app.toasts,
                ) {
                    app.refresh_entries();
                    app.toasts.success("Files copied successfully!");
                }

                app.show_popup = None;
            }
            FileDropAction::Move => {
                app.clipboard = Some(Clipboard::Cut(dropped_files));

                let current_tab = app.tab_manager.current_tab_mut();
                if handle_clipboard_operations(
                    &mut app.clipboard,
                    &current_tab.current_path,
                    &mut current_tab.action_history,
                    &mut app.toasts,
                ) {
                    app.refresh_entries();
                    app.toasts.success("Files moved successfully!");
                }

                app.show_popup = None;
            }
            FileDropAction::Cancel => {
                app.show_popup = None;
            }
            FileDropAction::None => {
                // Keep the popup open
                if !keep_open {
                    app.show_popup = None;
                }
            }
        }
    }
}

pub(crate) fn handle_key_press(
    ctx: &Context,
    app: &mut Kiorg,
    dropped_files: Vec<PathBuf>,
) -> bool {
    let action = app.get_shortcut_action_from_input(ctx, false);
    if let Some(action) = action {
        match action {
            ShortcutAction::Exit => {
                app.show_popup = None;
                return true; // Input handled
            }
            ShortcutAction::CopyEntry => {
                app.clipboard = Some(Clipboard::Copy(dropped_files));

                let current_tab = app.tab_manager.current_tab_mut();
                if handle_clipboard_operations(
                    &mut app.clipboard,
                    &current_tab.current_path,
                    &mut current_tab.action_history,
                    &mut app.toasts,
                ) {
                    app.refresh_entries();
                    app.toasts.success("Files copied successfully!");
                }

                app.show_popup = None;
                return true; // Input handled
            }
            ShortcutAction::CutEntry => {
                app.clipboard = Some(Clipboard::Cut(dropped_files));

                let current_tab = app.tab_manager.current_tab_mut();
                if handle_clipboard_operations(
                    &mut app.clipboard,
                    &current_tab.current_path,
                    &mut current_tab.action_history,
                    &mut app.toasts,
                ) {
                    app.refresh_entries();
                    app.toasts.success("Files moved successfully!");
                }

                app.show_popup = None;
                return true; // Input handled
            }
            _ => {}
        }
    }

    false // Input not handled
}
