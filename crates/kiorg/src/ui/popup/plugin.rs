use crate::app::Kiorg;
use crate::config::shortcuts::ShortcutAction;
use crate::plugins::manager::{FailedPlugin, LoadedPlugin};
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use super::window_utils::show_center_popup_window;

/// Helper function to display plugins in a table layout
fn display_plugins_table<'a>(
    ui: &mut egui::Ui,
    plugins: impl Iterator<Item = (&'a String, &'a Arc<LoadedPlugin>)>,
    colors: &crate::config::colors::AppColors,
) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().resizable(true))
        .column(Column::auto().resizable(true))
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.colored_label(colors.fg_light, "Name");
            });
            header.col(|ui| {
                ui.colored_label(colors.fg_light, "Version");
            });
            header.col(|ui| {
                ui.colored_label(colors.fg_light, "Load Time");
            });
            header.col(|ui| {
                ui.colored_label(colors.fg_light, "Description");
            });
        })
        .body(|mut body| {
            for (plugin_name, plugin) in plugins {
                body.row(18.0, |mut row| {
                    let (display_name, description, desc_color) =
                        if let Some(error_msg) = &plugin.state.lock().unwrap().error {
                            if error_msg.contains("Incompatible protocol version") {
                                (
                                    format!("🚨 {}", plugin_name),
                                    format!("WARN: {}", error_msg),
                                    colors.warn,
                                )
                            } else {
                                (
                                    format!("❌ {}", plugin_name),
                                    format!("ERROR: {}", error_msg),
                                    colors.error,
                                )
                            }
                        } else {
                            (
                                plugin_name.to_string(),
                                plugin.metadata.description.clone(),
                                colors.fg,
                            )
                        };

                    // Name
                    row.col(|ui| {
                        ui.label(display_name);
                    });

                    // Version
                    row.col(|ui| {
                        ui.label(&plugin.metadata.version);
                    });

                    // Load Time
                    row.col(|ui| {
                        let time_text = format!("{:.2}ms", plugin.load_time.as_secs_f64() * 1000.0);
                        ui.label(time_text);
                    });

                    // Description
                    row.col(|ui| {
                        ui.colored_label(desc_color, description);
                    });
                });
            }
        });
}

/// Helper function to display failed plugins in a grid layout
fn display_failed_plugins_grid<'a>(
    ui: &mut egui::Ui,
    grid_id: &str,
    plugins: impl Iterator<Item = &'a FailedPlugin>,
    colors: &crate::config::colors::AppColors,
) {
    egui::Grid::new(grid_id)
        .num_columns(2)
        .max_col_width(400.0)
        .spacing([20.0, 2.0])
        .show(ui, |ui| {
            for failed_plugin in plugins {
                ui.label(failed_plugin.path.to_string_lossy());
                ui.colored_label(colors.error, &failed_plugin.error);
                ui.end_row();
            }
        });
}

pub fn draw(app: &mut Kiorg, ctx: &egui::Context) {
    let mut keep_open = true;

    // Check for shortcut actions based on input
    let action = app.get_shortcut_action_from_input(ctx);
    if let Some(ShortcutAction::Exit) = action {
        app.show_popup = None;
        return;
    }

    let loaded_plugins_map = app.plugin_manager.list_loaded();
    let failed_plugins_map = app.plugin_manager.list_failed();
    let _ = show_center_popup_window("Plugins", ctx, &mut keep_open, |ui| {
        if loaded_plugins_map.is_empty() && failed_plugins_map.is_empty() {
            ui.label("No plugins found");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if !loaded_plugins_map.is_empty() {
                    display_plugins_table(ui, loaded_plugins_map.iter(), &app.colors);
                }

                if !failed_plugins_map.is_empty() {
                    if !loaded_plugins_map.is_empty() {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                    }
                    ui.colored_label(app.colors.fg_light, "Failed to load plugins");
                    display_failed_plugins_grid(
                        ui,
                        "failed_plugins_list_grid",
                        failed_plugins_map.iter(),
                        &app.colors,
                    );
                }
            });
        }
    });

    if !keep_open {
        app.show_popup = None;
    }
}
