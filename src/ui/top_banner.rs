use crate::app::Kiorg;
use crate::ui::popup::PopupType;
use crate::ui::{path_nav, update};
use egui::{RichText, Ui};

pub fn draw(app: &mut Kiorg, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            let tab_indexes = app.tab_manager.tab_indexes();

            // Path navigation on the left
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                if let Some(message) = path_nav::draw_path_navigation(
                    ui,
                    &app.tab_manager.current_tab_mut().current_path,
                    &app.colors,
                    tab_indexes.len(),
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
                        app.show_popup = Some(PopupType::Bookmarks(0));
                        ui.close();
                    }

                    #[cfg(target_os = "macos")]
                    if ui.button("Volumes").clicked() {
                        app.show_popup = Some(PopupType::Volumes(0));
                        ui.close();
                    }

                    if ui.button("Themes").clicked() {
                        // Use current theme key or default to dark_kiorg
                        let current_theme_key = app
                            .config
                            .theme
                            .clone()
                            .unwrap_or_else(|| "dark_kiorg".to_string());
                        app.show_popup = Some(PopupType::Themes(current_theme_key));
                        ui.close();
                    }

                    if ui.button("Check for update").clicked() {
                        update::check_for_updates(app);
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Help").clicked() {
                        app.show_popup = Some(PopupType::Help);
                        ui.close();
                    }

                    if ui.button("About").clicked() {
                        app.show_popup = Some(PopupType::About);
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        app.show_popup = Some(PopupType::Exit);
                        ui.close();
                    }
                });

                // Add some spacing between menu and tabs
                ui.add_space(5.0);

                // Tab numbers
                for (i, is_current) in tab_indexes.into_iter().rev() {
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
