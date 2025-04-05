use egui::{RichText, Ui};
use crate::app::Kiorg;
use crate::ui::path_nav;

pub struct TopBanner<'a> {
    app: &'a mut Kiorg,
}

impl<'a> TopBanner<'a> {
    pub fn new(app: &'a mut Kiorg) -> Self {
        Self { app }
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Path navigation on the left
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    if let Some(message) = path_nav::draw_path_navigation(ui, &self.app.tab_manager.current_tab().current_path, &self.app.colors) {
                        match message {
                            path_nav::PathNavMessage::Navigate(path) => {
                                self.app.navigate_to(path);
                            }
                        }
                    }
                });

                // Tab numbers on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    for i in (0..self.app.tab_manager.tabs.len()).rev() {
                        let is_current = i == self.app.tab_manager.current_tab_index;
                        let text = format!("{}", i + 1);
                        let color = if is_current {
                            self.app.colors.yellow
                        } else {
                            self.app.colors.gray
                        };
                        if ui.link(RichText::new(text).color(color)).clicked() {
                            self.app.tab_manager.switch_to_tab(i);
                        }
                    }
                });
            });
            ui.separator();
        });
    }
} 