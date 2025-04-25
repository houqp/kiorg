use crate::app::Kiorg;
use crate::ui::path_nav;
use egui::{RichText, Ui};

pub fn draw(app: &mut Kiorg, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Path navigation on the left
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                if let Some(message) = path_nav::draw_path_navigation(
                    ui,
                    &app.tab_manager.current_tab().current_path,
                    &app.colors,
                ) {
                    match message {
                        path_nav::PathNavMessage::Navigate(path) => {
                            app.navigate_to_dir(path);
                        }
                    }
                }
            });

            // Tab numbers on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                for (i, is_current) in app.tab_manager.tab_indexes().into_iter().rev() {
                    let text = format!("{}", i + 1);
                    let color = if is_current {
                        app.colors.yellow
                    } else {
                        app.colors.gray
                    };
                    if ui.link(RichText::new(text).color(color)).clicked() {
                        app.tab_manager.switch_to_tab(i);
                        app.refresh_entries();
                    }
                }
            });
        });
        ui.separator();
    });
}
