use crate::config::colors::AppColors;
use egui::{self, Align2, RichText};

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
                    ("D", "Delete selected file/directory"),
                    ("r", "Rename selected file/directory"),
                    ("space", "Select/deselect entry"),
                    ("y", "Copy selected entry"),
                    ("x", "Cut selected entry"),
                    ("p", "Paste copied/cut entries"),
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

            ui.heading(RichText::new("Bookmarks").color(colors.yellow));

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

            ui.separator();

            ui.heading(RichText::new("Tabs").color(colors.yellow));

            let table = egui::Grid::new("tab_help_grid");
            table.show(ui, |ui| {
                let shortcuts = [("t", "Create new tab"), ("1-9", "Switch to tab number")];

                for (key, description) in shortcuts {
                    ui.label(RichText::new(key).color(colors.yellow));
                    ui.label(description);
                    ui.end_row();
                }
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                if ui
                    .link(RichText::new("Press Enter to close").color(colors.yellow))
                    .clicked()
                {
                    *show_help = false;
                }
                if ui
                    .link(RichText::new("Press ? to close").color(colors.gray))
                    .clicked()
                {
                    *show_help = false;
                }
                ui.add_space(10.0);
            });
        });
}
