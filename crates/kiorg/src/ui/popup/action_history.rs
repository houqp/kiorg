use crate::app::Kiorg;
use crate::models::action_history::{ActionType, HistoryAction};
use crate::ui::popup::PopupType;
use crate::utils::file_operations;
use crate::utils::rollback::RollbackManager;
use egui::{Context, Frame, RichText, ScrollArea};

use super::window_utils::new_center_popup_window;

/// Draw the action history popup with rollback options
pub fn draw(ctx: &Context, app: &mut Kiorg) {
    if Some(PopupType::ActionHistory) != app.show_popup {
        return;
    }

    let mut should_undo_last = false;
    let mut should_redo_last = false;

    // Extract the data we need before entering the UI closure
    let history = &app.tab_manager.current_tab_ref().action_history;
    let active_actions = history.get_active_actions();
    let rolled_back_actions = history.get_rolled_back_actions();

    // Create a temporary boolean for the window's open state
    let mut window_open = true;

    if let Some(_response) = new_center_popup_window("Action History")
        .default_size([600.0, 400.0])
        .open(&mut window_open)
        .show(ctx, |ui| {
            Frame::default()
                .fill(app.colors.bg_extreme)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        if rolled_back_actions.is_empty() && active_actions.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("No file operations recorded yet");
                            });
                            return;
                        }

                        render_action_history_content(
                            ui,
                            app,
                            active_actions,
                            rolled_back_actions,
                            &mut should_undo_last,
                            &mut should_redo_last,
                        )
                    });
                });
        })
    {
        // Handle window close button
        if !window_open {
            app.show_popup = None;
        }
    } else {
        // Window was closed
        app.show_popup = None;
    }

    // Handle actions after the UI closure
    if should_undo_last {
        undo_last_action(app);
    }

    if should_redo_last {
        redo_last_action(app);
    }
}

/// Render the main content of the action history popup
fn render_action_history_content(
    ui: &mut egui::Ui,
    app: &Kiorg,
    active_actions: &[HistoryAction],
    rolled_back_actions: &[HistoryAction],
    should_undo_last: &mut bool,
    should_redo_last: &mut bool,
) {
    // Scrollable list of actions in timeline format
    ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
        // Helper function to render an action with its status
        let mut render_action = |action: &HistoryAction, is_active: bool| {
            ui.horizontal(|ui| {
                // Timestamp
                let timestamp_color = if is_active {
                    app.colors.fg_light
                } else {
                    app.colors.fg_light.gamma_multiply(0.6) // More dimmed
                };

                ui.label(
                    RichText::new(action.timestamp.format("%H:%M:%S").to_string())
                        .size(10.0)
                        .color(timestamp_color)
                        .family(egui::FontFamily::Monospace), // Monospace for better alignment
                );

                ui.add_space(8.0);

                // Action description with appropriate styling
                let description = action.get_description();

                if is_active {
                    // Normal styling for active actions
                    ui.label(RichText::new(&description));
                } else {
                    // Struck through and dimmed for rolled back actions with prefix
                    ui.label(
                        RichText::new(format!("{} (rolled back)", description))
                            .strikethrough()
                            .color(app.colors.fg_light.gamma_multiply(0.6))
                            .italics(),
                    );
                }
            });

            ui.separator();
        };

        // Show rolled back actions first (in reverse order - newest first)
        for action in rolled_back_actions.iter().rev() {
            render_action(action, false);
        }

        // Show active actions (in reverse order - newest first)
        for action in active_actions.iter().rev() {
            render_action(action, true);
        }
    });

    ui.separator();

    // Footer with undo, redo and close buttons
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Redo Last Action button
            let has_redoable_action = app
                .tab_manager
                .current_tab_ref()
                .action_history
                .get_last_redoable_action()
                .is_some();
            ui.add_enabled_ui(has_redoable_action, |ui| {
                if ui.button("Redo Last Action").clicked() {
                    *should_redo_last = true;
                }
            });

            // Undo Last Action button
            let has_undoable_action = app
                .tab_manager
                .current_tab_ref()
                .action_history
                .get_last_rollbackable_action()
                .is_some();
            ui.add_enabled_ui(has_undoable_action, |ui| {
                if ui.button("Undo Last Action").clicked() {
                    *should_undo_last = true;
                }
            });
        });
    });
}

/// Undo the most recent rollbackable action
pub fn undo_last_action(app: &mut Kiorg) {
    let tab = app.tab_manager.current_tab_mut();
    if let Some(action) = tab.action_history.undo_last_action() {
        // Perform the rollback
        let rollback_manager = RollbackManager::new();
        match rollback_manager.rollback_action(&action.action_type) {
            Ok(message) => {
                // Show success message and refresh
                app.toasts
                    .success(format!("Rollback successful: {}", message));
                app.refresh_entries();
            }
            Err(error) => {
                app.toasts.error(format!("Rollback failed: {}", error));
            }
        }
    } else {
        app.toasts.info("No actions available to undo");
    }
}

/// Redo the most recently rolled back action
pub fn redo_last_action(app: &mut Kiorg) {
    let tab = app.tab_manager.current_tab_mut();

    let action = match tab.action_history.redo_last_action() {
        None => {
            app.toasts.info("No actions available to redo");
            return;
        }
        Some(act) => act,
    };

    // Re-perform the action
    match &action.action_type {
        ActionType::Create { operations } => {
            for op in operations {
                let result = if op.is_dir {
                    std::fs::create_dir_all(&op.path)
                } else {
                    if let Some(parent) = op.path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::File::create(&op.path).map(|_| ())
                };

                match result {
                    Ok(_) => {
                        app.toasts
                            .success(format!("Redone: Created '{}'", op.path.display()));
                    }
                    Err(e) => {
                        app.toasts.error(format!(
                            "Failed to redo creation of '{}': {}",
                            op.path.display(),
                            e
                        ));
                    }
                }
            }
        }
        ActionType::Rename { operations } => {
            for op in operations {
                if let Some(parent) = op.new_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                match std::fs::rename(&op.old_path, &op.new_path) {
                    Ok(_) => {
                        app.toasts.success(format!(
                            "Redone: Renamed '{}' to '{}'",
                            op.old_path.display(),
                            op.new_path.display()
                        ));
                    }
                    Err(e) => {
                        app.toasts.error(format!(
                            "Failed to redo rename of '{}' to '{}': {}",
                            op.old_path.display(),
                            op.new_path.display(),
                            e
                        ));
                    }
                }
            }
        }
        ActionType::Copy { operations } => {
            for op in operations {
                let result = if op.source_path.is_dir() {
                    file_operations::copy_dir_recursively(&op.source_path, &op.target_path)
                } else {
                    if let Some(parent) = op.target_path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::copy(&op.source_path, &op.target_path).map(|_| ())
                };

                match result {
                    Ok(_) => {
                        app.toasts.success(format!(
                            "Redone: Copied '{}' to '{}'",
                            op.source_path.display(),
                            op.target_path.display()
                        ));
                    }
                    Err(e) => {
                        app.toasts.error(format!(
                            "Failed to redo copy of '{}' to '{}': {}",
                            op.source_path.display(),
                            op.target_path.display(),
                            e
                        ));
                    }
                }
            }
        }
        ActionType::Move { operations } => {
            for op in operations {
                if let Some(parent) = op.target_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }

                match std::fs::rename(&op.source_path, &op.target_path) {
                    Ok(_) => {
                        app.toasts.success(format!(
                            "Redone: Moved '{}' to '{}'",
                            op.source_path.display(),
                            op.target_path.display()
                        ));
                    }
                    Err(e) => {
                        app.toasts.error(format!(
                            "Failed to redo move of '{}' to '{}': {}",
                            op.source_path.display(),
                            op.target_path.display(),
                            e
                        ));
                    }
                }
            }
        }
    }
}
