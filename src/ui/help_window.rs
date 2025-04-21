use crate::config::colors::AppColors;
use egui::{self, RichText};

use super::window_utils::new_center_popup_window;

pub fn show_help_window(ctx: &egui::Context, show_help: &mut bool, colors: &AppColors) {
    let mut keep_open = *show_help; // Use a temporary variable for the open state
                                    //
    let response = new_center_popup_window("Help")
        .open(&mut keep_open)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 6.0);

            ui.horizontal(|ui| {
                // Column 1: Navigation
                ui.vertical(|ui| {
                    ui.heading(RichText::new("Navigation").color(colors.gray));
                    let table = egui::Grid::new("help_grid");
                    table.show(ui, |ui| {
                        let shortcuts = [
                            ("j / ⬇", "Move down"),
                            ("k / ⬆", "Move up"),
                            ("h / ⬅", "Go to parent directory"),
                            ("l / ➡ / Enter", "Open directory or file"),
                            ("gg", "Go to first entry"),
                            ("G", "Go to last entry"),
                        ];

                        for (key, description) in shortcuts {
                            ui.label(RichText::new(key).color(colors.yellow));
                            ui.label(description);
                            ui.end_row();
                        }
                    });

                    ui.add_space(10.0); // Space between sections

                    // Section: File Operations
                    ui.heading(RichText::new("File Operations").color(colors.gray));
                    let table = egui::Grid::new("file_op_help_grid");
                    table.show(ui, |ui| {
                        let shortcuts = [
                            ("D", "Delete selected file/directory"),
                            ("r", "Rename selected file/directory"),
                            ("a", "Add file/directory"),
                            ("space", "Select/deselect entry"),
                            ("y", "Copy selected entry"),
                            ("x", "Cut selected entry"),
                            ("p", "Paste copied/cut entries"),
                        ];
                        for (key, description) in shortcuts {
                            ui.label(RichText::new(key).color(colors.yellow));
                            ui.label(description);
                            ui.end_row();
                        }
                    });
                });

                ui.separator(); // Vertical separator between columns

                // Column 2: Bookmarks & Tabs
                ui.vertical(|ui| {
                    ui.heading(RichText::new("Bookmarks").color(colors.gray));
                    let table = egui::Grid::new("bookmark_help_grid");
                    table.show(ui, |ui| {
                        let shortcuts = [
                            ("b", "Add/remove bookmark for current directory"),
                            ("B (shift+b)", "Show bookmark popup"),
                            ("d", "Delete selected bookmark (in popup)"),
                        ];

                        for (key, description) in shortcuts {
                            ui.label(RichText::new(key).color(colors.yellow));
                            ui.label(description);
                            ui.end_row();
                        }
                    });

                    ui.add_space(10.0); // Space between sections

                    ui.heading(RichText::new("Tabs").color(colors.gray));
                    let table = egui::Grid::new("tab_help_grid");
                    table.show(ui, |ui| {
                        let shortcuts = [("t", "Create new tab"), ("1-9", "Switch to tab number")];

                        for (key, description) in shortcuts {
                            ui.label(RichText::new(key).color(colors.yellow));
                            ui.label(description);
                            ui.end_row();
                        }
                    });

                    ui.add_space(10.0); // Space between sections

                    // Section: Search
                    ui.heading(RichText::new("Search").color(colors.gray));
                    let table = egui::Grid::new("search_help_grid");
                    table.show(ui, |ui| {
                        let shortcuts = [
                            ("/", "Activate search filter"),
                            ("Enter (in search)", "Apply filter"),
                            ("Esc (in search)", "Clear filter"),
                        ];
                        for (key, description) in shortcuts {
                            ui.label(RichText::new(key).color(colors.yellow));
                            ui.label(description);
                            ui.end_row();
                        }
                    });

                    ui.add_space(10.0); // Space between sections

                    // Section: Utils
                    ui.heading(RichText::new("Utils").color(colors.gray));
                    let table = egui::Grid::new("utils_help_grid");
                    table.show(ui, |ui| {
                        let shortcuts = [
                            ("T (shift+t)", "Open terminal panel at current directory"),
                            ("q", "Exit application"),
                            ("?", "Toggle this help window"),
                        ];
                        for (key, description) in shortcuts {
                            ui.label(RichText::new(key).color(colors.yellow));
                            ui.label(description);
                            ui.end_row();
                        }
                    });
                });
            });

            ui.separator(); // Horizontal separator below columns

            ui.vertical_centered(|ui| {
                if ui
                    .link(RichText::new("Press ? or Enter to close").color(colors.gray))
                    .clicked()
                {
                    *show_help = false;
                }
            });
        });

    if response.is_some() {
        *show_help = keep_open;
    }
}
