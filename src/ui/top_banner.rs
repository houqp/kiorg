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
                    &app.tab_manager.current_tab_mut().current_path,
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
                // Menu button with popup
                ui.menu_button(RichText::new("☰").color(app.colors.fg_light), |ui| {
                    ui.set_min_width(150.0);

                    if ui.button("Bookmarks").clicked() {
                        app.show_popup = Some(crate::app::PopupType::Bookmarks(0));
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Help").clicked() {
                        app.show_popup = Some(crate::app::PopupType::Help);
                        ui.close_menu();
                    }

                    if ui.button("About").clicked() {
                        app.show_popup = Some(crate::app::PopupType::About);
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        app.show_popup = Some(crate::app::PopupType::Exit);
                        ui.close_menu();
                    }
                });

                // Add some spacing between menu and tabs
                ui.add_space(5.0);

                // Tab numbers
                for (i, is_current) in app.tab_manager.tab_indexes().into_iter().rev() {
                    let text = format!("{}", i + 1);
                    let color = if is_current {
                        app.colors.highlight
                    } else {
                        app.colors.link_text
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
