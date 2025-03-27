use egui::{self, Align2, RichText};
use crate::config::colors::AppColors;

pub fn show_help_window(ctx: &egui::Context, show_help: &mut bool, colors: &AppColors) {
    if !*show_help {
        return;
    }

    egui::Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 6.0);
            
            ui.heading(RichText::new("Navigation").color(colors.yellow));
            
            let table = egui::Grid::new("help_grid");
            table.show(ui, |ui| {
                let shortcuts = [
                    ("j / ↓", "Move down"),
                    ("k / ↑", "Move up"),
                    ("h / ←", "Go to parent directory"),
                    ("l / → / Enter", "Open directory or file"),
                    ("gg", "Go to first entry"),
                    ("G", "Go to last entry"),
                    ("?", "Toggle this help window"),
                    ("q", "Exit application"),
                ];

                for (key, description) in shortcuts {
                    ui.label(RichText::new(key).color(colors.yellow));
                    ui.label(description);
                    ui.end_row();
                }
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("Close").color(colors.fg)).clicked() {
                        *show_help = false;
                    }
                });
            });
        });
} 