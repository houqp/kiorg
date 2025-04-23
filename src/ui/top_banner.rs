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
                    &app.state.tab_manager.current_tab().current_path,
                    &app.state.colors,
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
                for (i, is_current) in app.state.tab_manager.tab_indexes().into_iter().rev() {
                    let text = format!("{}", i + 1);
                    let color = if is_current {
                        app.state.colors.yellow
                    } else {
                        app.state.colors.gray
                    };
                    if ui.link(RichText::new(text).color(color)).clicked() {
                        app.state.tab_manager.switch_to_tab(i);
                    }
                }
            });
        });
        ui.separator();
    });
}
